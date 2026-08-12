# Bonjour

mDNS/Bonjour service discovery providing Apple Foundation-like NetService for TontooOS. Feature-gated behind `bonjour`.

## API Overview

| Type | Description |
|---|---|
| `NetService` | mDNS service registration |
| `NetServiceBrowser` | mDNS service discovery |
| `NetServiceDelegate` | Delegate trait for service events |
| `NetServiceBrowserDelegate` | Delegate trait for browser events |
| `NetServicePublishOptions` | Publish options enum |

> **Note:** This module requires the `bonjour` feature flag: `tontoo-foundation = { features = ["bonjour"] }`

## NetService

Registers and resolves mDNS network services.

```rust
pub struct NetService {
    domain: String,
    service_type: String,
    name: String,
    port: i32,
    txt_record: HashMap<String, String>,
    host_name: Option<String>,
    addresses: Vec<SocketAddr>,
    is_publishing: bool,
    is_resolving: bool,
}
```

### Constructor

```rust
pub fn new(domain: &str, service_type: &str, name: &str, port: i32) -> Self
```

### Accessors

```rust
pub fn domain(&self) -> &str
pub fn service_type(&self) -> &str
pub fn name(&self) -> &str
pub fn port(&self) -> i32
pub fn host_name(&self) -> Option<&str>
pub fn addresses(&self) -> &[SocketAddr]
pub fn is_publishing(&self) -> bool
pub fn is_resolving(&self) -> bool
```

### TXT Record

```rust
pub fn txt_record_data(&self) -> Option<Vec<u8>>
pub fn set_txt_record_data(&mut self, data: Option<&[u8]>)
pub fn set_txt_record_value(&mut self, key: &str, value: &str)
pub fn remove_txt_record_value(&mut self, key: &str)
```

### Publishing

```rust
pub fn publish(&mut self) -> Result<()>
pub fn publish_with_options(&mut self, options: NetServicePublishOptions) -> Result<()>
pub fn stop(&mut self)
pub fn resolve(&mut self)
pub fn resolve_with_timeout(&mut self, timeout: f64)
```

Returns `Err` when the `mdns-sd` feature is not enabled.

### Run Loop

```rust
pub fn schedule_in_run_loop(&self, mode: &str)
pub fn remove_from_run_loop(&self, mode: &str)
```

No-op on Linux (run loops are not applicable).

## NetServiceBrowser

Discovers mDNS services on the network.

```rust
pub fn new() -> Self
pub fn search_for_services_of_type(&mut self, service_type: &str, domain: &str)
pub fn search_for_browseable_domains(&mut self)
pub fn search_for_registration_domains(&mut self)
pub fn stop(&mut self)
pub fn is_searching(&self) -> bool
pub fn services(&self) -> &[NetService]
pub fn add_service(&mut self, service: NetService)
pub fn remove_service(&mut self, name: &str)
```

## Service Types

Pre-defined service type constants:

| Constant | Value |
|---|---|
| `AFP` | `_afpovertcp._tcp` |
| `FTP` | `_ftp._tcp` |
| `HTTP` | `_http._tcp` |
| `HTTPS` | `_https._tcp` |
| `SSH` | `_ssh._tcp` |
| `SMB` | `_smb._tcp` |
| `AIRPLAY` | `_airplay._tcp` |
| `PRINTER` | `_ipp._tcp` |
| `RAOP` | `_raop._tcp` |

## Usage

```rust
use tontoo_foundation::bonjour::*;

// Register a service
let mut service = NetService::new("local.", "_http._tcp.", "My Web Server", 8080);
service.set_txt_record_value("version", "1.0");
service.publish().unwrap();

// Browse for services
let mut browser = NetServiceBrowser::new();
browser.search_for_services_of_type("_http._tcp.", "local.");
```

## Cross References

- [URL.md](URL.md) - URL types for service endpoints
