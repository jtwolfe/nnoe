# phpIPAM Extensions API

NNOE extends phpIPAM with additional API endpoints and functionality for DNS/DHCP management and threat intelligence.

## Plugin API Methods

### NNOE Class

The main `NNOE` class provides integration with etcd backend.

#### `pushDnsZone($zone_name, $records)`

Push DNS zone configuration to etcd for agent distribution.

**Parameters:**
- `$zone_name` (string): Zone name (e.g., "example.com")
- `$records` (array): Array of DNS records

**Example:**
```php
$nnoe = new NNOE();
$records = [
    ['name' => '@', 'type' => 'A', 'value' => '192.168.1.1'],
    ['name' => 'www', 'type' => 'A', 'value' => '192.168.1.2']
];
$nnoe->pushDnsZone('example.com', $records);
```

**etcd Key:** `/nnoe/dns/zones/{zone_name}`

#### `pushDhcpScope($scope_id, $scope_data)`

Push DHCP scope configuration to etcd.

**Parameters:**
- `$scope_id` (string): Scope identifier
- `$scope_data` (array): Scope configuration

**Example:**
```php
$scope = [
    'subnet' => '192.168.1.0/24',
    'pool' => ['start' => '192.168.1.100', 'end' => '192.168.1.200'],
    'gateway' => '192.168.1.1'
];
$nnoe->pushDhcpScope('scope-1', $scope);
```

**etcd Key:** `/nnoe/dhcp/scopes/{scope_id}`

#### `getAgentHealth()`

Retrieve health status of all NNOE agents.

**Returns:** Array of agent health data

**Example:**
```php
$health = $nnoe->getAgentHealth();
// Returns: ['agent-1' => ['status' => 'healthy', ...], ...]
```

**etcd Key:** `/nnoe/agents/health`

#### `getThreatFeeds()`

Get threat intelligence feeds from etcd.

**Returns:** Array of threat domains with metadata

**Example:**
```php
$threats = $nnoe->getThreatFeeds();
// Returns: ['malicious.com' => ['source' => 'MISP', 'severity' => 'high', ...], ...]
```

**etcd Keys:** `/nnoe/threats/domains/{domain}`

#### `getNebulaTopology()`

Get Nebula overlay network topology.

**Returns:** Array of topology data

**Example:**
```php
$topology = $nnoe->getNebulaTopology();
// Returns network graph data
```

**etcd Key:** `/nnoe/nebula/topology`

## REST API Endpoints

phpIPAM can expose REST endpoints for NNOE operations:

### POST `/api/nnoe/zones`

Create or update DNS zone.

**Request Body:**
```json
{
  "zone": "example.com",
  "records": [
    {"name": "@", "type": "A", "value": "192.168.1.1"}
  ]
}
```

### GET `/api/nnoe/agents/health`

Get agent health status.

### GET `/api/nnoe/threats`

Get threat intelligence feeds.

## Configuration

Edit `/var/www/html/plugins/nnoe/etc/nnoe-config.php`:

```php
define('NNOE_ETCD_ENDPOINTS', 'http://127.0.0.1:2379,http://127.0.0.2:2379');
define('NNOE_ETCD_PREFIX', '/nnoe');
```

## Integration Hooks

phpIPAM hooks available for extension:

- `nnoe_display_section($section)`: Display NNOE widgets in sections
- `nnoe_zone_created($zone)`: Called when zone is created
- `nnoe_scope_created($scope)`: Called when DHCP scope is created

