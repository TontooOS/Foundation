//! Bonjour / mDNS – NetService, NetServiceBrowser

use crate::error::{FoundationError, Result};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

/// NSNetService equivalent
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

impl NetService {
    pub fn new(domain: &str, service_type: &str, name: &str, port: i32) -> Self {
        Self {
            domain: domain.to_string(),
            service_type: service_type.to_string(),
            name: name.to_string(),
            port,
            txt_record: HashMap::new(),
            host_name: None,
            addresses: Vec::new(),
            is_publishing: false,
            is_resolving: false,
        }
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }

    pub fn service_type(&self) -> &str {
        &self.service_type
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn port(&self) -> i32 {
        self.port
    }

    pub fn host_name(&self) -> Option<&str> {
        self.host_name.as_deref()
    }

    pub fn addresses(&self) -> &[SocketAddr] {
        &self.addresses
    }

    pub fn txt_record_data(&self) -> Option<Vec<u8>> {
        if self.txt_record.is_empty() {
            None
        } else {
            let mut data = Vec::new();
            for (key, value) in &self.txt_record {
                let entry = format!("{}={}", key, value);
                data.extend_from_slice(&(entry.len() as u8).to_be_bytes());
                data.extend_from_slice(entry.as_bytes());
            }
            Some(data)
        }
    }

    pub fn set_txt_record_data(&mut self, data: Option<&[u8]>) {
        if let Some(data) = data {
            self.txt_record.clear();
            let mut i = 0;
            while i < data.len() {
                let len = data[i] as usize;
                i += 1;
                if i + len <= data.len() {
                    let entry = String::from_utf8_lossy(&data[i..i + len]);
                    if let Some(pos) = entry.find('=') {
                        let key = entry[..pos].to_string();
                        let value = entry[pos + 1..].to_string();
                        self.txt_record.insert(key, value);
                    }
                    i += len;
                }
            }
        } else {
            self.txt_record.clear();
        }
    }

    pub fn set_txt_record_value(&mut self, key: &str, value: &str) {
        self.txt_record.insert(key.to_string(), value.to_string());
    }

    pub fn remove_txt_record_value(&mut self, key: &str) {
        self.txt_record.remove(key);
    }

    pub fn includes_name_in_txt_record(&self, name: &str) -> bool {
        self.txt_record.contains_key(name)
    }

    pub fn delegate(&self) -> Option<&dyn NetServiceDelegate> {
        None
    }

    pub fn schedule_in_run_loop(&self, _mode: &str) {
        // Run loop scheduling not applicable on Linux
    }

    pub fn remove_from_run_loop(&self, _mode: &str) {
        // Run loop scheduling not applicable on Linux
    }

    pub fn publish(&mut self) -> Result<()> {
        #[cfg(feature = "mdns-sd")]
        {
            self.is_publishing = true;
            Ok(())
        }
        #[cfg(not(feature = "mdns-sd"))]
        {
            Err(FoundationError::Bonjour("mdns-sd feature not enabled".to_string()))
        }
    }

    pub fn publish_with_options(&mut self, _options: NetServicePublishOptions) -> Result<()> {
        self.publish()
    }

    pub fn stop(&mut self) {
        self.is_publishing = false;
        self.is_resolving = false;
    }

    pub fn resolve(&mut self) {
        self.is_resolving = true;
    }

    pub fn resolve_with_timeout(&mut self, _timeout: f64) {
        self.resolve()
    }

    pub fn is_publishing(&self) -> bool {
        self.is_publishing
    }

    pub fn is_resolving(&self) -> bool {
        self.is_resolving
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetServicePublishOptions {
    NoAutoRename,
    ListenForConnections,
}

/// NSNetServiceDelegate equivalent
pub trait NetServiceDelegate: Send + Sync {
    fn net_service_did_publish(&self, _service: &NetService) {}
    fn net_service_did_not_publish(&self, _service: &NetService, _error: &FoundationError) {}
    fn net_service_did_resolve(&self, _service: &NetService) {}
    fn net_service_did_not_resolve(&self, _service: &NetService, _error: &FoundationError) {}
    fn net_service_did_stop(&self, _service: &NetService) {}
    fn net_service_did_update_txt_record(&self, _service: &NetService) {}
    fn net_service_did_accept_connection(&self, _service: &NetService, _input: &[u8], _output: &mut Vec<u8>) {}
}

/// NSNetServiceBrowser equivalent
pub struct NetServiceBrowser {
    service_type: String,
    domain: String,
    is_searching: bool,
    services: Vec<NetService>,
    delegate: Option<Box<dyn NetServiceBrowserDelegate>>,
}

impl NetServiceBrowser {
    pub fn new() -> Self {
        Self {
            service_type: String::new(),
            domain: String::new(),
            is_searching: false,
            services: Vec::new(),
            delegate: None,
        }
    }

    pub fn search_for_services_of_type(&mut self, service_type: &str, domain: &str) {
        self.service_type = service_type.to_string();
        self.domain = domain.to_string();
        self.is_searching = true;
    }

    pub fn search_for_browseable_domains(&mut self) {
        self.is_searching = true;
    }

    pub fn search_for_registration_domains(&mut self) {
        self.is_searching = true;
    }

    pub fn stop(&mut self) {
        self.is_searching = false;
    }

    pub fn is_searching(&self) -> bool {
        self.is_searching
    }

    pub fn services(&self) -> &[NetService] {
        &self.services
    }

    pub fn add_service(&mut self, service: NetService) {
        self.services.push(service);
    }

    pub fn remove_service(&mut self, name: &str) {
        self.services.retain(|s| s.name() != name);
    }

    pub fn set_delegate<D: NetServiceBrowserDelegate + 'static>(&mut self, delegate: D) {
        self.delegate = Some(Box::new(delegate));
    }

    pub fn delegate(&self) -> Option<&dyn NetServiceBrowserDelegate> {
        self.delegate.as_deref()
    }

    pub fn schedule_in_run_loop(&self, _mode: &str) {
        // Run loop scheduling not applicable on Linux
    }

    pub fn remove_from_run_loop(&self, _mode: &str) {
        // Run loop scheduling not applicable on Linux
    }
}

impl Default for NetServiceBrowser {
    fn default() -> Self {
        Self::new()
    }
}

/// NSNetServiceBrowserDelegate equivalent
pub trait NetServiceBrowserDelegate: Send + Sync {
    fn net_service_browser_did_find_service(&self, _browser: &NetServiceBrowser, _service: &NetService) {}
    fn net_service_browser_did_remove_service(&self, _browser: &NetServiceBrowser, _service: &NetService) {}
    fn net_service_browser_did_not_search(&self, _browser: &NetServiceBrowser, _error: &FoundationError) {}
    fn net_service_browser_did_stop_search(&self, _browser: &NetServiceBrowser) {}
}

/// NSNetServiceDelegate equivalent for TXT record handling
pub trait NetServiceTXTRecord: Send + Sync {
    fn txt_record_data(&self) -> Option<Vec<u8>>;
    fn set_txt_record_data(&mut self, data: Option<&[u8]>);
}

/// Bonjour service types
pub mod service_types {
    pub const AFP: &str = "_afpovertcp._tcp";
    pub const FTP: &str = "_ftp._tcp";
    pub const HTTP: &str = "_http._tcp";
    pub const HTTPS: &str = "_https._tcp";
    pub const IMAP: &str = "_imap._tcp";
    pub const IRC: &str = "_irc._tcp";
    pub const LDAP: &str = "_ldap._tcp";
    pub const POP3: &str = "_pop3._tcp";
    pub const SMB: &str = "_smb._tcp";
    pub const SMTP: &str = "_smtp._tcp";
    pub const SSH: &str = "_ssh._tcp";
    pub const TELNET: &str = "_telnet._tcp";
    pub const AIRPLAY: &str = "_airplay._tcp";
    pub const AIRDROP: &str = "_adisk._tcp";
    pub const HOME_SHARING: &str = "_home-sharing._tcp";
    pub const PRINTER: &str = "_ipp._tcp";
    pub const RAOP: &str = "_raop._tcp";
    pub const REMOTE_AUDIO: &str = "_raop._tcp";
}

/// mDNS service discovery using mdns-sd crate
#[cfg(feature = "mdns-sd")]
pub mod mdns {
    use super::*;
    use mdns_sd::{ServiceDaemon, ServiceEvent};

    pub struct MDNSService {
        daemon: ServiceDaemon,
        service_fullname: String,
    }

    impl MDNSService {
        pub fn new() -> Result<Self> {
            let daemon = ServiceDaemon::new()
                .map_err(|e| FoundationError::Bonjour(e.to_string()))?;
            Ok(Self {
                daemon,
                service_fullname: String::new(),
            })
        }

        pub fn register(
            &self,
            service_type: &str,
            name: &str,
            port: u16,
            txt_records: &[(&str, &str)],
        ) -> Result<()> {
            let mut properties = Vec::new();
            for (k, v) in txt_records {
                properties.push((k.to_string(), v.to_string()));
            }

            let service_fullname = format!("{}.{}.local.", name, service_type);
            self.daemon.register(mdns_sd::ServiceInfo::new(
                service_type,
                name,
                "local.",
                (),
                port,
                &[],
            ).map_err(|e| FoundationError::Bonjour(e.to_string()))?)
            .map_err(|e| FoundationError::Bonjour(e.to_string()))?;

            Ok(())
        }

        pub fn unregister(&self) -> Result<()> {
            Ok(())
        }
    }

    pub struct MDNSBrowser {
        daemon: ServiceDaemon,
        service_type: String,
    }

    impl MDNSBrowser {
        pub fn new(service_type: &str) -> Result<Self> {
            let daemon = ServiceDaemon::new()
                .map_err(|e| FoundationError::Bonjour(e.to_string()))?;
            Ok(Self {
                daemon,
                service_type: service_type.to_string(),
            })
        }

        pub fn browse(&self) -> Result<()> {
            let receiver = self.daemon.browse(&self.service_type)
                .map_err(|e| FoundationError::Bonjour(e.to_string()))?;
            Ok(())
        }

        pub fn stop(&self) -> Result<()> {
            self.daemon.shutdown()
                .map_err(|e| FoundationError::Bonjour(e.to_string()))?;
            Ok(())
        }
    }
}
