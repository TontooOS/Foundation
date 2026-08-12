# Process

System process information providing Apple Foundation-like ProcessInfo for TontooOS.

## API Overview

| Type | Description |
|---|---|
| `ProcessInfo` | Current process information |
| `OperatingSystemVersion` | OS version triplet |
| `ThermalState` | Device thermal state enum |
| `PowerState` | Power state enum |
| `ProcessInfoActivityOptions` | Activity options flags |

## ProcessInfo

Static methods for current process information.

```rust
pub struct ProcessInfo;
```

### Process Identity

```rust
pub fn process_info() -> Self
pub fn process_name(&self) -> String
pub fn process_identifier(&self) -> u32
pub fn globally_unique_string(&self) -> String
```

### System Information

```rust
pub fn operating_system_version(&self) -> OperatingSystemVersion
pub fn operating_system_version_string(&self) -> String
pub fn is_operating_system_at_least_version(&self, version: &OperatingSystemVersion) -> bool
pub fn physical_memory(&self) -> u64
pub fn processor_count(&self) -> usize
pub fn active_processor_count(&self) -> usize
pub fn system_uptime(&self) -> f64
pub fn host_name(&self) -> Option<String>
```

### Process State

```rust
pub fn thermal_state(&self) -> ThermalState
pub fn is_low_power_mode_enabled(&self) -> bool
pub fn is_ios_on_mac(&self) -> bool
pub fn is_macos_app(&self) -> bool
pub fn is_mac_catalyst_app(&self) -> bool
```

### Environment

```rust
pub fn arguments(&self) -> Vec<String>
pub fn environment(&self) -> HashMap<String, String>
```

## OperatingSystemVersion

```rust
pub struct OperatingSystemVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub version_string: String,
}
```

### Constructor

```rust
pub fn new(major: u32, minor: u32, patch: u32) -> Self
```

## ThermalState

| Variant | Description |
|---|---|
| `Nominal` | Normal operation |
| `Fair` | Slightly elevated temperature |
| `Serious` | High temperature, performance may be reduced |
| `Critical` | Very high temperature, significant throttling |

## Usage

```rust
use tontoo_foundation::prelude::*;

let info = ProcessInfo::process_info();

// Process identity
println!("PID: {}", info.process_identifier());
println!("Name: {}", info.process_name());

// System info
let version = info.operating_system_version();
println!("OS: {}.{}.{}", version.major, version.minor, version.patch);
println!("Memory: {} bytes", info.physical_memory());
println!("CPUs: {}", info.processor_count());

// Version check
let required = OperatingSystemVersion::new(5, 10, 0);
assert!(info.is_operating_system_at_least_version(&required));

// Environment
let args = info.arguments();
let env = info.environment();
```

## Cross References

- [Date.md](Date.md) - Date types for uptime calculations
