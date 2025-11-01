# NNOE Plugin for phpIPAM

This plugin extends phpIPAM to provide integration with NNOE's distributed DDI platform.

## Features

- **DNS Zone Management**: Push DNS zones and records to etcd for agent distribution
- **DHCP Scope Management**: Configure DHCP scopes and push to etcd
- **Threat Intelligence Viewer**: Display MISP threat feeds and blocked domains
- **Agent Health Monitoring**: Real-time status of NNOE agents
- **Nebula Topology Visualization**: View Nebula overlay network topology
- **Dashboard Widgets**: Integration with phpIPAM dashboard

## Installation

1. Copy the plugin files to phpIPAM's plugin directory:
   ```bash
   cp -r integrations/phpipam-plugin/src/* /var/www/html/plugins/nnoe/
   ```

2. Copy and configure the config file:
   ```bash
   cp integrations/phpipam-plugin/etc/nnoe-config.php.example \
      /var/www/html/plugins/nnoe/etc/nnoe-config.php
   # Edit nnoe-config.php with your etcd endpoints
   ```

3. Install PHP dependencies (GuzzleHTTP):
   ```bash
   composer require guzzlehttp/guzzle
   ```

4. Enable the plugin in phpIPAM admin interface

## Configuration

Edit `/var/www/html/plugins/nnoe/etc/nnoe-config.php`:

- `NNOE_ETCD_ENDPOINTS`: Comma-separated list of etcd endpoints
- `NNOE_ETCD_PREFIX`: etcd key prefix (default: `/nnoe`)
- Feature flags for DNS, DHCP, threat viewer, topology

## Usage

### DNS Zone Management

In phpIPAM's DNS section, zones can be pushed to NNOE via the plugin API:

```php
$nnoe = new NNOE();
$records = [
    ['name' => '@', 'type' => 'A', 'value' => '192.168.1.1'],
    ['name' => 'www', 'type' => 'A', 'value' => '192.168.1.2']
];
$nnoe->pushDnsZone('example.com', $records);
```

### DHCP Scope Management

DHCP scopes can be configured and pushed:

```php
$nnoe = new NNOE();
$scope = [
    'subnet' => '192.168.1.0/24',
    'pool' => ['start' => '192.168.1.100', 'end' => '192.168.1.200'],
    'gateway' => '192.168.1.1'
];
$nnoe->pushDhcpScope('scope-1', $scope);
```

## API Methods

- `pushDnsZone($zone_name, $records)` - Push DNS zone to etcd
- `pushDhcpScope($scope_id, $scope_data)` - Push DHCP scope to etcd
- `getAgentHealth()` - Get health status of all agents
- `getThreatFeeds()` - Get threat intelligence from etcd
- `getNebulaTopology()` - Get Nebula network topology

## Status

This plugin is in active development. Full integration with phpIPAM UI will be completed in Phase 3.

