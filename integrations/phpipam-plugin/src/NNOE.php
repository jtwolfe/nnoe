<?php
/**
 * NNOE (New Network Orchestration Engine) Plugin for phpIPAM
 * 
 * This plugin extends phpIPAM to integrate with NNOE's etcd backend,
 * providing DNS/DHCP management, threat intelligence viewing, and
 * Nebula topology visualization.
 * 
 * @package phpIPAM
 * @subpackage NNOE Plugin
 */

class NNOE {
    private $etcd_endpoints;
    private $etcd_prefix;
    private $etcd_client;
    private $etcd_tls_ca_cert;
    private $etcd_tls_cert;
    private $etcd_tls_key;
    private $etcd_tls_verify;
    private $version = "1.0.0";
    
    /**
     * Initialize NNOE plugin
     */
    public function __construct() {
        // Load configuration
        $this->loadConfig();
        
        // Initialize etcd client
        $this->initEtcdClient();
    }
    
    /**
     * Load configuration from config file
     */
    private function loadConfig() {
        $config_file = dirname(__FILE__) . '/../etc/nnoe-config.php';
        if (file_exists($config_file)) {
            require_once $config_file;
            $this->etcd_endpoints = defined('NNOE_ETCD_ENDPOINTS') ? 
                explode(',', NNOE_ETCD_ENDPOINTS) : ['http://127.0.0.1:2379'];
            $this->etcd_prefix = defined('NNOE_ETCD_PREFIX') ? NNOE_ETCD_PREFIX : '/nnoe';
            
            // TLS configuration
            $this->etcd_tls_ca_cert = defined('NNOE_ETCD_TLS_CA_CERT') ? NNOE_ETCD_TLS_CA_CERT : '';
            $this->etcd_tls_cert = defined('NNOE_ETCD_TLS_CERT') ? NNOE_ETCD_TLS_CERT : '';
            $this->etcd_tls_key = defined('NNOE_ETCD_TLS_KEY') ? NNOE_ETCD_TLS_KEY : '';
            $this->etcd_tls_verify = defined('NNOE_ETCD_TLS_VERIFY') ? NNOE_ETCD_TLS_VERIFY : true;
        } else {
            $this->etcd_endpoints = ['http://127.0.0.1:2379'];
            $this->etcd_prefix = '/nnoe';
            $this->etcd_tls_ca_cert = '';
            $this->etcd_tls_cert = '';
            $this->etcd_tls_key = '';
            $this->etcd_tls_verify = true;
        }
    }
    
    /**
     * Initialize etcd client connection with TLS and retry support
     */
    private function initEtcdClient() {
        $client_config = [
            'base_uri' => $this->etcd_endpoints[0],
            'timeout' => 5.0,
            'connect_timeout' => 2.0,
        ];
        
        // Add TLS/SSL configuration if cert paths are provided
        if (!empty($this->etcd_tls_ca_cert)) {
            $client_config['verify'] = $this->etcd_tls_ca_cert;
        } elseif (isset($this->etcd_tls_verify) && $this->etcd_tls_verify === false) {
            $client_config['verify'] = false; // Only for development/testing
        }
        
        // Add client certificate if provided
        if (!empty($this->etcd_tls_cert) && !empty($this->etcd_tls_key)) {
            $client_config['cert'] = [$this->etcd_tls_cert, $this->etcd_tls_key];
        }
        
        $this->etcd_client = new GuzzleHttp\Client($client_config);
    }
    
    /**
     * Retry HTTP request with exponential backoff
     * 
     * @param callable $request_fn Function that makes the HTTP request
     * @param int $max_retries Maximum number of retry attempts
     * @param int $initial_delay_ms Initial delay in milliseconds
     * @param int $max_delay_ms Maximum delay in milliseconds
     * @return mixed Response from request function
     * @throws Exception If all retries fail
     */
    private function retryRequest($request_fn, $max_retries = 3, $initial_delay_ms = 100, $max_delay_ms = 5000) {
        $delay = $initial_delay_ms;
        $last_exception = null;
        
        for ($attempt = 0; $attempt <= $max_retries; $attempt++) {
            try {
                return $request_fn();
            } catch (GuzzleHttp\Exception\RequestException $e) {
                $last_exception = $e;
                
                // Don't retry on client errors (4xx) except timeout/connection errors
                $status_code = $e->getResponse() ? $e->getResponse()->getStatusCode() : 0;
                if ($status_code >= 400 && $status_code < 500 && $status_code != 408 && $status_code != 429) {
                    throw $e; // Client error, don't retry
                }
                
                // Don't retry on last attempt
                if ($attempt >= $max_retries) {
                    break;
                }
                
                // Exponential backoff
                usleep($delay * 1000); // Convert ms to microseconds
                $delay = min($delay * 2, $max_delay_ms);
            } catch (Exception $e) {
                $last_exception = $e;
                
                if ($attempt >= $max_retries) {
                    break;
                }
                
                usleep($delay * 1000);
                $delay = min($delay * 2, $max_delay_ms);
            }
        }
        
        throw $last_exception ?? new Exception("Request failed after {$max_retries} retries");
    }
    
    /**
     * Push DNS zone configuration to etcd
     * 
     * @param string $zone_name Zone name (e.g., "example.com")
     * @param array $records DNS records array
     * @return bool Success status
     */
    public function pushDnsZone($zone_name, $records) {
        try {
            $zone_data = [
                'domain' => $zone_name,
                'ttl' => 3600,
                'records' => $records
            ];
            
            $key = $this->etcd_prefix . '/dns/zones/' . $zone_name;
            $value = json_encode($zone_data);
            
            return $this->etcdPut($key, $value);
        } catch (Exception $e) {
            error_log("NNOE: Failed to push DNS zone: " . $e->getMessage());
            return false;
        }
    }
    
    /**
     * Push DHCP scope configuration to etcd
     * 
     * @param string $scope_id Scope identifier
     * @param array $scope_data Scope configuration
     * @return bool Success status
     */
    public function pushDhcpScope($scope_id, $scope_data) {
        try {
            $key = $this->etcd_prefix . '/dhcp/scopes/' . $scope_id;
            $value = json_encode($scope_data);
            
            return $this->etcdPut($key, $value);
        } catch (Exception $e) {
            error_log("NNOE: Failed to push DHCP scope: " . $e->getMessage());
            return false;
        }
    }
    
    /**
     * Get agent health status from etcd
     * 
     * @return array Health status of all agents
     */
    public function getAgentHealth() {
        try {
            $key = $this->etcd_prefix . '/agents/health';
            $response = $this->etcdGet($key);
            
            if ($response && isset($response['kvs'][0]['value'])) {
                $health_data = base64_decode($response['kvs'][0]['value']);
                return json_decode($health_data, true);
            }
            
            return [];
        } catch (Exception $e) {
            error_log("NNOE: Failed to get agent health: " . $e->getMessage());
            return [];
        }
    }
    
    /**
     * Get threat intelligence feeds from etcd
     * 
     * @return array Threat domains
     */
    public function getThreatFeeds() {
        try {
            $key = $this->etcd_prefix . '/threats/domains';
            $response = $this->etcdGet($key, true); // prefix search
            
            $threats = [];
            if ($response && isset($response['kvs'])) {
                foreach ($response['kvs'] as $kv) {
                    $domain = str_replace($this->etcd_prefix . '/threats/domains/', '', $kv['key']);
                    $threat_data = json_decode(base64_decode($kv['value']), true);
                    $threats[$domain] = $threat_data;
                }
            }
            
            return $threats;
        } catch (Exception $e) {
            error_log("NNOE: Failed to get threat feeds: " . $e->getMessage());
            return [];
        }
    }
    
    /**
     * Get Nebula topology information
     * 
     * @return array Nebula network topology
     */
    public function getNebulaTopology() {
        try {
            $key = $this->etcd_prefix . '/nebula/topology';
            $response = $this->etcdGet($key);
            
            if ($response && isset($response['kvs'][0]['value'])) {
                $topology_data = base64_decode($response['kvs'][0]['value']);
                return json_decode($topology_data, true);
            }
            
            return [];
        } catch (Exception $e) {
            error_log("NNOE: Failed to get Nebula topology: " . $e->getMessage());
            return [];
        }
    }
    
    /**
     * Put key-value pair to etcd with retry logic
     * 
     * @param string $key Key
     * @param string $value Value
     * @return bool Success status
     */
    private function etcdPut($key, $value) {
        try {
            $response = $this->retryRequest(function() use ($key, $value) {
                return $this->etcd_client->put('/v3/kv/put', [
                    'json' => [
                        'key' => base64_encode($key),
                        'value' => base64_encode($value)
                    ]
                ]);
            });
            
            return $response->getStatusCode() === 200 || $response->getStatusCode() === 201;
        } catch (Exception $e) {
            error_log("NNOE: etcd PUT failed after retries (endpoint: {$this->etcd_endpoints[0]}, key: {$key}): " . $e->getMessage());
            return false;
        }
    }
    
    /**
     * Get key-value pair from etcd with retry logic
     * 
     * @param string $key Key
     * @param bool $prefix Whether to use prefix search
     * @return array|null Response data
     */
    private function etcdGet($key, $prefix = false) {
        try {
            $response = $this->retryRequest(function() use ($key, $prefix) {
                $params = [
                    'key' => base64_encode($key)
                ];
                
                if ($prefix) {
                    $params['range_end'] = base64_encode($key . chr(0xFF));
                }
                
                return $this->etcd_client->post('/v3/kv/range', [
                    'json' => $params
                ]);
            });
            
            if ($response->getStatusCode() === 200) {
                return json_decode($response->getBody()->getContents(), true);
            }
            
            return null;
        } catch (Exception $e) {
            error_log("NNOE: etcd GET failed after retries (endpoint: {$this->etcd_endpoints[0]}, key: {$key}): " . $e->getMessage());
            return null;
        }
    }
    
    /**
     * Watch etcd key for changes (long polling via HTTP)
     * 
     * @param string $key Key to watch
     * @param callable $callback Callback function called when key changes
     * @param int $revision Optional revision to watch from
     * @return bool Success status
     */
    public function watchKey($key, $callback, $revision = null) {
        try {
            $params = [
                'key' => base64_encode($key),
                'prev_kv' => true
            ];
            
            if ($revision !== null) {
                $params['start_revision'] = $revision;
            }
            
            // Use long polling for watch (etcd v3 watch via HTTP requires polling)
            // In production, consider using etcd watch stream (gRPC)
            $response = $this->retryRequest(function() use ($params) {
                return $this->etcd_client->post('/v3/watch', [
                    'json' => $params,
                    'timeout' => 30.0
                ]);
            });
            
            if ($response->getStatusCode() === 200) {
                $watch_data = json_decode($response->getBody()->getContents(), true);
                if (isset($watch_data['events']) && is_array($watch_data['events'])) {
                    foreach ($watch_data['events'] as $event) {
                        if (isset($event['kv']['value'])) {
                            $value = base64_decode($event['kv']['value']);
                            call_user_func($callback, $key, $value, $event);
                        }
                    }
                }
                return true;
            }
            
            return false;
        } catch (Exception $e) {
            error_log("NNOE: etcd watch failed for key {$key}: " . $e->getMessage());
            return false;
        }
    }
    
    /**
     * Get Prometheus metrics for Grafana embedding
     * 
     * @param string $prometheus_endpoint Prometheus endpoint URL
     * @param string $query PromQL query
     * @return array|null Metrics data
     */
    public function getPrometheusMetrics($prometheus_endpoint, $query) {
        try {
            $client = new GuzzleHttp\Client(['timeout' => 5.0]);
            $response = $client->get($prometheus_endpoint . '/api/v1/query', [
                'query' => ['query' => $query]
            ]);
            
            if ($response->getStatusCode() === 200) {
                return json_decode($response->getBody()->getContents(), true);
            }
            
            return null;
        } catch (Exception $e) {
            error_log("NNOE: Failed to fetch Prometheus metrics: " . $e->getMessage());
            return null;
        }
    }
    
    /**
     * Generate Grafana iframe embed URL
     * 
     * @param string $grafana_url Grafana base URL
     * @param int $dashboard_id Dashboard ID
     * @param array $params Optional parameters (from, to, orgId, etc.)
     * @return string Embed URL
     */
    public function getGrafanaEmbedUrl($grafana_url, $dashboard_id, $params = []) {
        $default_params = [
            'from' => 'now-1h',
            'to' => 'now',
            'kiosk' => 'tv', // TV mode for embedding
        ];
        
        $params = array_merge($default_params, $params);
        $query_string = http_build_query($params);
        
        return rtrim($grafana_url, '/') . '/d/' . $dashboard_id . '?' . $query_string;
    }
    
    /**
     * Get DNS zones count for dashboard
     * 
     * @return int Number of active zones
     */
    public function getDnsZoneCount() {
        try {
            $key = $this->etcd_prefix . '/dns/zones';
            $response = $this->etcdGet($key, true);
            
            if ($response && isset($response['kvs'])) {
                return count($response['kvs']);
            }
            
            return 0;
        } catch (Exception $e) {
            error_log("NNOE: Failed to get DNS zone count: " . $e->getMessage());
            return 0;
        }
    }
    
    /**
     * Get DHCP scope count for dashboard
     * 
     * @return int Number of active scopes
     */
    public function getDhcpScopeCount() {
        try {
            $key = $this->etcd_prefix . '/dhcp/scopes';
            $response = $this->etcdGet($key, true);
            
            if ($response && isset($response['kvs'])) {
                return count($response['kvs']);
            }
            
            return 0;
        } catch (Exception $e) {
            error_log("NNOE: Failed to get DHCP scope count: " . $e->getMessage());
            return 0;
        }
    }
    
    /**
     * Get active DHCP leases count
     * 
     * @return int Number of active leases
     */
    public function getActiveLeaseCount() {
        try {
            $key = $this->etcd_prefix . '/dhcp/leases';
            $response = $this->etcdGet($key, true);
            
            if ($response && isset($response['kvs'])) {
                // Filter active leases (not expired)
                $active_count = 0;
                foreach ($response['kvs'] as $kv) {
                    $lease_data = json_decode(base64_decode($kv['value']), true);
                    if (isset($lease_data['valid_lft'])) {
                        $expires_at = $lease_data['cltt'] + $lease_data['valid_lft'];
                        if ($expires_at > time()) {
                            $active_count++;
                        }
                    }
                }
                return $active_count;
            }
            
            return 0;
        } catch (Exception $e) {
            error_log("NNOE: Failed to get active lease count: " . $e->getMessage());
            return 0;
        }
    }
}

// Hook into phpIPAM's section display
function nnoe_display_section($section) {
    global $NNOE;
    
    if (!isset($NNOE)) {
        $NNOE = new NNOE();
    }
    
    // Add NNOE dashboard widget if section is dashboard
    if ($section === 'dashboard') {
        $zone_count = $NNOE->getDnsZoneCount();
        $scope_count = $NNOE->getDhcpScopeCount();
        $lease_count = $NNOE->getActiveLeaseCount();
        $threat_count = count($NNOE->getThreatFeeds());
        $agent_health = $NNOE->getAgentHealth();
        $healthy_agents = count(array_filter($agent_health, function($agent) {
            return isset($agent['status']) && $agent['status'] === 'healthy';
        }));
        ?>
        <div class="widget-dashboard" style="border: 1px solid #ddd; padding: 15px; margin: 10px 0; background: #f9f9f9;">
            <h4>NNOE Network Orchestration</h4>
            <div class="widget-content">
                <div style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 10px;">
                    <div>
                        <strong>Agent Health:</strong> 
                        <span id="nnoe-agent-health"><?php echo $healthy_agents . ' / ' . count($agent_health); ?></span>
                    </div>
                    <div>
                        <strong>Active DNS Zones:</strong> 
                        <span id="nnoe-zone-count"><?php echo $zone_count; ?></span>
                    </div>
                    <div>
                        <strong>DHCP Scopes:</strong> 
                        <span id="nnoe-scope-count"><?php echo $scope_count; ?></span>
                    </div>
                    <div>
                        <strong>Active Leases:</strong> 
                        <span id="nnoe-lease-count"><?php echo $lease_count; ?></span>
                    </div>
                    <div>
                        <strong>Threat Domains:</strong> 
                        <span id="nnoe-threat-count"><?php echo $threat_count; ?></span>
                    </div>
                </div>
                <?php
                // Grafana embed support (if configured)
                $grafana_url = defined('NNOE_GRAFANA_URL') ? NNOE_GRAFANA_URL : '';
                $grafana_dashboard = defined('NNOE_GRAFANA_DASHBOARD') ? NNOE_GRAFANA_DASHBOARD : '';
                if (!empty($grafana_url) && !empty($grafana_dashboard)) {
                    $embed_url = $NNOE->getGrafanaEmbedUrl($grafana_url, $grafana_dashboard);
                    ?>
                    <div style="margin-top: 15px;">
                        <h5>NNOE Metrics Dashboard</h5>
                        <iframe src="<?php echo htmlspecialchars($embed_url); ?>" 
                                width="100%" 
                                height="400" 
                                frameborder="0"
                                style="border: 1px solid #ddd;"></iframe>
                    </div>
                    <?php
                }
                ?>
            </div>
        </div>
        <script>
        // Auto-refresh dashboard data
        (function() {
            var refreshInterval = <?php echo defined('NNOE_DASHBOARD_REFRESH') ? NNOE_DASHBOARD_REFRESH : 30; ?> * 1000;
            setInterval(function() {
                // Refresh counts via AJAX (implement endpoint in phpIPAM API)
                // For now, page refresh handles updates
            }, refreshInterval);
        })();
        </script>
        <?php
    }
}

// Hook into DNS section for real-time updates
function nnoe_dns_section_hook($action, $data) {
    global $NNOE;
    
    if (!isset($NNOE)) {
        $NNOE = new NNOE();
    }
    
    // Watch for DNS zone changes and update phpIPAM view
    if ($action === 'zone_updated') {
        // Trigger zone refresh in UI
        return ['refresh' => true];
    }
    
    return null;
}

// Hook into DHCP section for real-time updates
function nnoe_dhcp_section_hook($action, $data) {
    global $NNOE;
    
    if (!isset($NNOE)) {
        $NNOE = new NNOE();
    }
    
    // Watch for DHCP scope changes and update phpIPAM view
    if ($action === 'scope_updated') {
        // Trigger scope refresh in UI
        return ['refresh' => true];
    }
    
    return null;
}

// Register hooks
if (function_exists('register_hook')) {
    register_hook('section_display', 'nnoe_display_section');
    register_hook('dns_action', 'nnoe_dns_section_hook');
    register_hook('dhcp_action', 'nnoe_dhcp_section_hook');
}

