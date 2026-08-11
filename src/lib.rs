//! TontooFoundation – Apple Foundation reimplemented in Rust for Linux
//!
//! Provides the core data types, utilities, and system services that
//! Foundation provides on Apple platforms, adapted for TontooOS on Linux.

pub mod error;
pub mod string;
pub mod collections;
pub mod date;
pub mod url;
pub mod file;
pub mod serialization;
pub mod formatting;
pub mod measurement;
pub mod notification;
pub mod userdefaults;
pub mod process;
pub mod threading;
pub mod undo;
pub mod predicate;
pub mod progress;

#[cfg(feature = "bonjour")]
pub mod bonjour;

/// Library version: (major, minor, patch).
pub const FOUNDATION_VERSION: (u32, u32, u32) = (0, 1, 0);

/// Convenience prelude that re-exports the most commonly used types.
pub mod prelude {
    pub use crate::error::{FoundationError, Result};
    pub use crate::string::{TString, Scanner, RegularExpression, DataDetector, DetectorKind};
    pub use crate::collections::{Array, Dictionary, Set};
    pub use crate::date::{Date, Calendar, CalendarIdentifier, DateFormatter, TimeZone, Locale, ISO8601DateFormatter, DateComponents, DateStyle};
    pub use crate::url::{URL, URLComponents, HTTPMethod, URLRequest};
    pub use crate::file::{FileManager, FileHandle, Bundle};
    pub use crate::serialization::{JSONSerialization, PropertyList};
    pub use crate::formatting::{
        NumberFormatter, ByteCountFormatter,
        MeasurementFormatter,
    };
    pub use crate::measurement::{Measurement, Unit, UnitType, UnitLength, UnitMass, UnitTemperature, UnitVolume};
    pub use crate::notification::{NotificationCenter, Notification};
    pub use crate::userdefaults::UserDefaults;
    pub use crate::process::{ProcessInfo, OperatingSystemVersion};
    pub use crate::threading::{Thread, OperationQueue, Lock, Mutex, Condition, RecursiveLock, ConditionLock};
    pub use crate::undo::UndoManager;
    pub use crate::predicate::{Predicate, SortDescriptor, PredicateOperator, Expression};
    pub use crate::progress::Progress;
    pub use crate::FOUNDATION_VERSION;
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    // ========== string.rs ==========
    #[test]
    fn test_tstring_new() {
        let s = TString::new();
        assert!(s.is_empty());
        assert_eq!(s.len(), 0);
    }

    #[test]
    fn test_tstring_from_str() {
        let s = TString::from_str("hello");
        assert_eq!(s.as_str(), "hello");
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn test_tstring_contains() {
        let s = TString::from_str("hello world");
        assert!(s.contains("world"));
        assert!(!s.contains("xyz"));
    }

    #[test]
    fn test_tstring_prefix_suffix() {
        let s = TString::from_str("hello world");
        assert!(s.has_prefix("hello"));
        assert!(s.has_suffix("world"));
        assert!(!s.has_prefix("world"));
    }

    #[test]
    fn test_tstring_substring() {
        let s = TString::from_str("hello world");
        let sub = s.substring(0, 5);
        assert!(sub.is_some());
        assert_eq!(sub.unwrap().as_str(), "hello");
    }

    #[test]
    fn test_tstring_replace() {
        let s = TString::from_str("hello world");
        let r = s.replace("world", "rust");
        assert_eq!(r.as_str(), "hello rust");
    }

    #[test]
    fn test_tstring_split() {
        let s = TString::from_str("a,b,c");
        let parts = s.split(",");
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].as_str(), "a");
    }

    #[test]
    fn test_tstring_case() {
        let s = TString::from_str("Hello");
        assert_eq!(s.to_lowercase().as_str(), "hello");
        assert_eq!(s.to_uppercase().as_str(), "HELLO");
    }

    #[test]
    fn test_scanner() {
        let mut scanner = Scanner::new("hello world");
        let result = scanner.scan_up_to(" ");
        assert!(result.is_some());
        assert_eq!(result.unwrap().as_str(), "hello");
        let rest = scanner.remaining();
        assert_eq!(rest, "world");
    }

    #[test]
    fn test_scanner_regex() {
        let mut scanner = Scanner::new("abc123def");
        let result = scanner.scan_regex(r"\d+");
        assert!(result.is_some());
        assert_eq!(result.unwrap().as_str(), "123");
    }

    #[test]
    fn test_regex() {
        let re = RegularExpression::new(r"\d+").unwrap();
        assert!(re.is_match("abc123"));
        assert!(!re.is_match("abc"));
        let matches = re.matches("a1b2c3");
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_regex_replace() {
        let re = RegularExpression::new(r"\d+").unwrap();
        let result = re.replace("abc123def456", "X");
        assert_eq!(result, "abcXdefX");
    }

    #[test]
    fn test_data_detector_url() {
        let detector = DataDetector::new(DetectorKind::URL).unwrap();
        let results = detector.detect("visit https://example.com today");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].value, "https://example.com");
    }

    // ========== collections.rs ==========
    #[test]
    fn test_array_new() {
        let arr: Array<i32> = Array::new();
        assert!(arr.is_empty());
        assert_eq!(arr.count(), 0);
    }

    #[test]
    fn test_array_add_get() {
        let mut arr = Array::new();
        arr.add(1);
        arr.add(2);
        arr.add(3);
        assert_eq!(arr.count(), 3);
        assert_eq!(arr.get(0), Some(&1));
        assert_eq!(arr.get(2), Some(&3));
    }

    #[test]
    fn test_array_first_last() {
        let mut arr = Array::new();
        arr.add(10);
        arr.add(20);
        assert_eq!(arr.first(), Some(&10));
        assert_eq!(arr.last(), Some(&20));
    }

    #[test]
    fn test_array_remove() {
        let mut arr = Array::new();
        arr.add(1);
        arr.add(2);
        arr.add(3);
        let removed = arr.remove(1);
        assert_eq!(removed, Some(2));
        assert_eq!(arr.count(), 2);
    }

    #[test]
    fn test_array_contains() {
        let mut arr = Array::new();
        arr.add(1);
        arr.add(2);
        assert!(arr.contains(&1));
        assert!(!arr.contains(&3));
    }

    #[test]
    fn test_array_filter() {
        let mut arr = Array::new();
        arr.add(1);
        arr.add(2);
        arr.add(3);
        arr.add(4);
        let filtered = arr.filter(|x| *x > 2);
        assert_eq!(filtered.count(), 2);
    }

    #[test]
    fn test_array_map() {
        let mut arr = Array::new();
        arr.add(1);
        arr.add(2);
        let mapped = arr.map(|x| x * 2);
        assert_eq!(mapped.get(0), Some(&2));
        assert_eq!(mapped.get(1), Some(&4));
    }

    #[test]
    fn test_dictionary() {
        let mut dict: Dictionary<String, i32> = Dictionary::new();
        dict.set("one".to_string(), 1);
        dict.set("two".to_string(), 2);
        assert_eq!(dict.get(&"one".to_string()), Some(&1));
        assert_eq!(dict.count(), 2);
    }

    #[test]
    fn test_set() {
        let mut set = Set::new();
        set.insert(1);
        set.insert(2);
        set.insert(3);
        set.insert(2);
        assert_eq!(set.count(), 3);
        assert!(set.contains(&2));
        assert!(!set.contains(&4));
    }

    // ========== date.rs ==========
    #[test]
    fn test_date_now() {
        let date = Date::now();
        assert!(date.timestamp() > 0);
    }

    #[test]
    fn test_date_from_timestamp() {
        let date = Date::from_timestamp(1000000);
        assert_eq!(date.timestamp(), 1000000);
    }

    #[test]
    fn test_date_adding() {
        let date = Date::from_timestamp(1000000);
        let later = date.adding_seconds(3600);
        assert_eq!(later.timestamp(), 1003600);
    }

    #[test]
    fn test_date_comparison() {
        let d1 = Date::from_timestamp(1000);
        let d2 = Date::from_timestamp(2000);
        assert!(d1.is_before(&d2));
        assert!(d2.is_after(&d1));
    }

    #[test]
    fn test_date_interval() {
        let d1 = Date::from_timestamp(1000);
        let d2 = Date::from_timestamp(3000);
        assert!((d2.time_interval_since(&d1) - 2000.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calendar() {
        let cal = Calendar::current();
        let date = Date::now();
        let comps = cal.components(&date);
        assert!(comps.year > 2020);
        assert!(comps.month >= 1 && comps.month <= 12);
    }

    #[test]
    fn test_date_formatter() {
        let formatter = DateFormatter::new().with_format("%Y-%m-%d");
        let date = Date::from_timestamp(1700000000);
        let formatted = formatter.format(&date);
        assert!(formatted.starts_with("2023-"));
    }

    #[test]
    fn test_iso8601_formatter() {
        let date = Date::now();
        let formatted = ISO8601DateFormatter::string_from(&date);
        assert!(formatted.contains("T"));
    }

    #[test]
    fn test_locale() {
        let locale = Locale::from_identifier("de_DE");
        assert_eq!(locale.language_code(), "de");
        assert_eq!(locale.country_code(), Some("DE"));
    }

    // ========== url.rs ==========
    #[test]
    fn test_url_parse() {
        let url = URL::from_str("https://example.com:8080/path?q=1#frag").unwrap();
        assert_eq!(url.scheme(), Some("https"));
        assert_eq!(url.host(), Some("example.com"));
        assert_eq!(url.port(), Some(8080));
        assert_eq!(url.path(), "/path");
    }

    #[test]
    fn test_url_query_items() {
        let url = URL::from_str("https://example.com?a=1&b=2").unwrap();
        let items = url.query_items();
        assert_eq!(items.get("a"), Some(&"1".to_string()));
        assert_eq!(items.get("b"), Some(&"2".to_string()));
    }

    #[test]
    fn test_url_path_components() {
        let url = URL::from_str("https://example.com/a/b/c").unwrap();
        let components = url.path_components();
        assert_eq!(components, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_url_components() {
        let mut comps = URLComponents::new();
        comps.set_scheme("https");
        comps.set_host("example.com");
        comps.set_port(Some(443));
        comps.set_path("/test");
        assert_eq!(comps.url().scheme(), Some("https"));
    }

    // ========== serialization.rs ==========
    #[test]
    fn test_json_serialization() {
        let data: std::collections::HashMap<String, String> = vec![("key".to_string(), "value".to_string())].into_iter().collect();
        let json = JSONSerialization::to_string(&data).unwrap();
        assert!(json.contains("key"));
        let parsed: std::collections::HashMap<String, String> = JSONSerialization::from_string(&json).unwrap();
        assert_eq!(parsed.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_json_valid() {
        assert!(JSONSerialization::is_valid_json(r#"{"a": 1}"#));
        assert!(!JSONSerialization::is_valid_json("not json"));
    }

    // ========== formatting.rs ==========
    #[test]
    fn test_number_formatter() {
        let fmt = NumberFormatter::new();
        let result = fmt.string_from_number(1234.56);
        assert!(result.contains("1234"));
    }

    #[test]
    fn test_number_formatter_int() {
        let fmt = NumberFormatter::new().with_grouping_separator(true);
        let result = fmt.string_from_int(1234567);
        assert!(result.contains("1,234,567"));
    }

    #[test]
    fn test_byte_count_formatter() {
        let fmt = ByteCountFormatter::new();
        let result = fmt.string_from_byte_count(1536);
        assert!(result.contains("1.5"));
        assert!(result.contains("KB"));
    }

    #[test]
    fn test_date_components_formatter() {
        use crate::date::DateComponentsFormatter;
        let fmt = DateComponentsFormatter::new();
        let comps = DateComponents {
            era: 0, year: 0, month: 0, day: 0,
            hour: 1, minute: 30, second: 0, nanosecond: 0, weekday: 0,
        };
        let result = fmt.string_from_components(&comps);
        assert!(result.contains("1h"));
        assert!(result.contains("30m"));
    }

    // ========== measurement.rs ==========
    #[test]
    fn test_measurement_new() {
        let m = Measurement::new(100.0, Box::new(UnitLength::meters.clone()));
        assert_eq!(m.value(), 100.0);
    }

    #[test]
    fn test_measurement_convert() {
        let m = Measurement::new(1.0, Box::new(UnitLength::kilometers.clone()));
        let converted = m.converted_to(&UnitLength::meters);
        assert!((converted.value() - 1000.0).abs() < 0.01);
    }

    #[test]
    fn test_measurement_add() {
        let m1 = Measurement::new(100.0, Box::new(UnitLength::meters.clone()));
        let m2 = Measurement::new(500.0, Box::new(UnitLength::meters.clone()));
        let result = m1.adding(&m2);
        assert_eq!(result.value(), 600.0);
    }

    #[test]
    fn test_measurement_subtract() {
        let m1 = Measurement::new(1000.0, Box::new(UnitLength::meters.clone()));
        let m2 = Measurement::new(500.0, Box::new(UnitLength::meters.clone()));
        let result = m1.subtracting(&m2);
        assert_eq!(result.value(), 500.0);
    }

    #[test]
    fn test_unit_symbol() {
        assert_eq!(UnitLength::meters.symbol, "m ");
        assert_eq!(UnitMass::kilograms.symbol, "kg ");
    }

    // ========== notification.rs ==========
    #[test]
    fn test_notification_center() {
        let center = NotificationCenter::new();
        let called = std::sync::Arc::new(std::sync::Mutex::new(false));
        let called_clone = called.clone();
        center.add_observer("test.event", move |_notif| {
            *called_clone.lock().unwrap() = true;
        });
        center.post("test.event", None);
        assert!(*called.lock().unwrap());
    }

    #[test]
    fn test_notification_remove_observer() {
        let center = NotificationCenter::new();
        let id = center.add_observer("test.event", |_notif| {});
        center.remove_observer(id);
        assert_eq!(center.observer_count(), 0);
    }

    // ========== userdefaults.rs ==========
    #[test]
    fn test_user_defaults_string() {
        let mut defaults = UserDefaults::standard();
        defaults.set_string("test_key", "test_value");
        assert_eq!(defaults.string("test_key"), Some("test_value"));
    }

    #[test]
    fn test_user_defaults_bool() {
        let mut defaults = UserDefaults::standard();
        defaults.set_bool("flag", true);
        assert!(defaults.bool("flag"));
    }

    #[test]
    fn test_user_defaults_int() {
        let mut defaults = UserDefaults::standard();
        defaults.set_int("count", 42);
        assert_eq!(defaults.int("count"), 42);
    }

    #[test]
    fn test_user_defaults_remove() {
        let mut defaults = UserDefaults::standard();
        defaults.set_string("temp", "data");
        defaults.remove("temp");
        assert!(defaults.string("temp").is_none());
    }

    // ========== process.rs ==========
    #[test]
    fn test_process_info() {
        let info = ProcessInfo::process_info();
        assert!(info.process_identifier() > 0);
        assert!(!info.process_name().is_empty());
    }

    // ========== threading.rs ==========
    #[test]
    fn test_lock() {
        let lock = Lock::new();
        let _guard = lock.lock();
    }

    #[test]
    fn test_mutex() {
        let mutex = Mutex::new(42);
        let val = mutex.lock();
        assert_eq!(*val, 42);
    }

    #[test]
    fn test_dispatch_group() {
        use crate::threading::DispatchGroup;
        let group = DispatchGroup::new();
        group.enter();
        group.leave();
        group.wait();
    }

    // ========== undo.rs ==========
    #[test]
    fn test_undo_manager() {
        let mut manager = UndoManager::new();
        let value = std::sync::Arc::new(std::sync::Mutex::new(0));
        let v = value.clone();
        manager.register_undo("test", move || {
            *v.lock().unwrap() = 1;
        });
        assert!(manager.can_undo());
        assert_eq!(manager.undo_count(), 1);
    }

    #[test]
    fn test_undo_redo() {
        let mut manager = UndoManager::new();
        let value = std::sync::Arc::new(std::sync::Mutex::new(0));
        let v = value.clone();
        manager.register_undo("test", move || {
            *v.lock().unwrap() += 1;
        });
        manager.undo();
        assert_eq!(*value.lock().unwrap(), 1);
        assert!(manager.can_redo());
    }

    // ========== predicate.rs ==========
    #[test]
    fn test_sort_descriptor() {
        use crate::predicate::Sortable;
        let mut data: Vec<std::collections::HashMap<String, String>> = vec![
            vec![("name".to_string(), "Charlie".to_string())].into_iter().collect(),
            vec![("name".to_string(), "Alice".to_string())].into_iter().collect(),
            vec![("name".to_string(), "Bob".to_string())].into_iter().collect(),
        ];
        let desc = SortDescriptor::new("name", true);
        data.sort_by_descriptors(&[desc]);
        assert_eq!(data[0].get("name"), Some(&"Alice".to_string()));
    }

    // ========== progress.rs ==========
    #[test]
    fn test_progress() {
        let mut progress = Progress::with_total_unit_count(100);
        progress.set_completed_unit_count(50);
        assert!((progress.fraction_completed() - 0.5).abs() < f64::EPSILON);
        assert!(!progress.is_finished());
    }

    #[test]
    fn test_progress_cancel() {
        let mut progress = Progress::with_total_unit_count(100);
        progress.cancel();
        assert!(progress.is_cancelled());
    }

    #[test]
    fn test_progress_pause_resume() {
        let mut progress = Progress::with_total_unit_count(100);
        progress.pause();
        assert!(progress.is_paused());
        progress.resume();
        assert!(!progress.is_paused());
    }

    // ========== file.rs ==========
    #[test]
    fn test_file_manager() {
        let _home = FileManager::home_directory();
        let _docs = FileManager::document_directory();
        let _cache = FileManager::cache_directory();
    }
}
