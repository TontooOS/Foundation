//! ProcessInfo – system information

use crate::error::Result;
use std::collections::HashMap;
use std::env;

/// NSProcessInfo equivalent
pub struct ProcessInfo;

impl ProcessInfo {
    pub fn process_info() -> Self {
        Self
    }

    pub fn process_name(&self) -> String {
        env::current_exe()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "unknown".to_string())
    }

    pub fn process_identifier(&self) -> u32 {
        std::process::id()
    }

    pub fn globally_unique_string(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }

    pub fn operating_system_version(&self) -> OperatingSystemVersion {
        let info = sys_info::os_release().unwrap_or_default();
        OperatingSystemVersion {
            major: 0,
            minor: 0,
            patch: 0,
            version_string: info,
        }
    }

    pub fn is_operating_system_at_least_version(&self, version: &OperatingSystemVersion) -> bool {
        let current = self.operating_system_version();
        current.major > version.major
            || (current.major == version.major && current.minor > version.minor)
            || (current.major == version.major && current.minor == version.minor && current.patch >= version.patch)
    }

    pub fn physical_memory(&self) -> u64 {
        sys_info::mem_info().map(|m| m.total).unwrap_or(0) * 1024
    }

    pub fn processor_count(&self) -> usize {
        num_cpus::get()
    }

    pub fn active_processor_count(&self) -> usize {
        num_cpus::get()
    }

    pub fn system_uptime(&self) -> f64 {
        #[cfg(not(windows))]
        {
            sys_info::boottime().map(|bt| bt.tv_sec as f64).unwrap_or(0.0)
        }
        #[cfg(windows)]
        {
            0.0
        }
    }

    pub fn arguments(&self) -> Vec<String> {
        env::args().collect()
    }

    pub fn environment(&self) -> HashMap<String, String> {
        env::vars().collect()
    }

    pub fn host_name(&self) -> Option<String> {
        sys_info::hostname().ok()
    }

    pub fn operating_system_version_string(&self) -> String {
        let info = sys_info::os_type().unwrap_or_default();
        let release = sys_info::os_release().unwrap_or_default();
        format!("{} {}", info, release)
    }

    pub fn is_ios_on_mac(&self) -> bool {
        false
    }

    pub fn is_macos_app_(&self) -> bool {
        false
    }

    pub fn thermal_state(&self) -> ThermalState {
        ThermalState::Nominal
    }

    pub fn is_low_power_mode_enabled(&self) -> bool {
        false
    }

    pub fn is_mac_catalyst_app(&self) -> bool {
        false
    }
}

/// NSOperatingSystemVersion equivalent
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatingSystemVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub version_string: String,
}

impl OperatingSystemVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            version_string: format!("{}.{}.{}", major, minor, patch),
        }
    }
}

impl std::fmt::Display for OperatingSystemVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.version_string)
    }
}

/// ProcessInfoThermalState equivalent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalState {
    Nominal,
    Fair,
    Serious,
    Critical,
}

/// ProcessInfoPowerState equivalent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerState {
    NotCharging,
    Charging,
    Full,
}

/// NSProcessInfoActivityOptions equivalent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessInfoActivityOptions(u32);

impl ProcessInfoActivityOptions {
    pub const IDLE_SYSTEM_SLEEP_DISABLED: Self = Self(1);
    pub const SUDDEN_TERMINATION_DISABLED: Self = Self(2);
    pub const AUTOMATIC_TERMINATION_DISABLED: Self = Self(4);
    pub const USER_INACTIVE: Self = Self(8);
    pub const LATENCY_SENSITIVE: Self = Self(16);
    pub const SENSITIVE: Self = Self(32);
    pub const BACKGROUND: Self = Self(64);
    pub const MAINTENANCE: Self = Self(128);
}
