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
        } else {
            $this->etcd_endpoints = ['http://127.0.0.1:2379'];
            $this->etcd_prefix = '/nnoe';
        }
    }
    
    /**
     * Initialize etcd client connection
     */
    private function initEtcdClient() {
        // Use HTTP client for etcd API (v3)
        $this->etcd_client = new GuzzleHttp\Client([
            'base_uri' => $this->etcd_endpoints[0],
            'timeout' => 5.0
        ]);
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
     * Put key-value pair to etcd
     * 
     * @param string $key Key
     * @param string $value Value
     * @return bool Success status
     */
    private function etcdPut($key, $value) {
        try {
            $response = $this->etcd_client->put('/v3/kv/put', [
                'json' => [
                    'key' => base64_encode($key),
                    'value' => base64_encode($value)
                ]
            ]);
            
            return $response->getStatusCode() === 200;
        } catch (Exception $e) {
            error_log("NNOE: etcd PUT failed: " . $e->getMessage());
            return false;
        }
    }
    
    /**
     * Get key-value pair from etcd
     * 
     * @param string $key Key
     * @param bool $prefix Whether to use prefix search
     * @return array|null Response data
     */
    private function etcdGet($key, $prefix = false) {
        try {
            $params = [
                'key' => base64_encode($key)
            ];
            
            if ($prefix) {
                $params['range_end'] = base64_encode($key . chr(0xFF));
            }
            
            $response = $this->etcd_client->post('/v3/kv/range', [
                'json' => $params
            ]);
            
            if ($response->getStatusCode() === 200) {
                return json_decode($response->getBody()->getContents(), true);
            }
            
            return null;
        } catch (Exception $e) {
            error_log("NNOE: etcd GET failed: " . $e->getMessage());
            return null;
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
        ?>
        <div class="widget-dashboard">
            <h4>NNOE Network Orchestration</h4>
            <div class="widget-content">
                <p>Agent Health: <span id="nnoe-agent-health">Loading...</span></p>
                <p>Active Zones: <span id="nnoe-zone-count">-</span></p>
                <p>Threat Domains: <span id="nnoe-threat-count">-</span></p>
            </div>
        </div>
        <?php
    }
}

// Register hooks
if (function_exists('register_hook')) {
    register_hook('section_display', 'nnoe_display_section');
}

