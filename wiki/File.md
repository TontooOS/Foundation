# File

File system operations providing Apple Foundation-like FileManager, FileHandle, and Bundle for TontooOS.

## API Overview

| Type | Description |
|---|---|
| `FileManager` | Directory and file operations |
| `FileHandle` | File read/write handle |
| `Bundle` | Application bundle info |
| `FileAttributes` | File metadata |
| `FileEnumerator` | Recursive directory traversal |
| `FileManagerDirectory` | Standard directory enum |
| `FileManagerDomain` | Domain enum for directory lookup |

## FileManager

Static methods for file system operations.

```rust
pub struct FileManager;
```

### File Operations

```rust
pub fn default() -> Self
pub fn file_exists(&self, path: &Path) -> bool
pub fn is_directory(&self, path: &Path) -> bool
pub fn create_directory(&self, path: &Path) -> Result<()>
pub fn remove_item(&self, path: &Path) -> Result<()>
pub fn copy_item(&self, from: &Path, to: &Path) -> Result<()>
pub fn move_item(&self, from: &Path, to: &Path) -> Result<()>
pub fn contents_of_directory(&self, path: &Path) -> Result<Vec<PathBuf>>
pub fn attributes_of_item(&self, path: &Path) -> Result<FileAttributes>
```

### Standard Directories

```rust
pub fn home_directory() -> PathBuf
pub fn document_directory() -> PathBuf
pub fn cache_directory() -> PathBuf
pub fn temp_directory() -> PathBuf
pub fn application_support_directory() -> PathBuf
pub fn desktop_directory() -> PathBuf
pub fn downloads_directory() -> PathBuf
pub fn trash_directory() -> PathBuf
```

### Query

```rust
pub fn is_deletable_file_at_path(&self, path: &Path) -> bool
pub fn is_executable_file_at_path(&self, path: &Path) -> bool
pub fn is_readable_file_at_path(&self, path: &Path) -> bool
pub fn is_writable_file_at_path(&self, path: &Path) -> bool
pub fn file_size(&self, path: &Path) -> Option<u64>
pub fn displayName_at_path(&self, path: &Path) -> String
```

### Enumeration

```rust
pub fn enumerator_at_path(&self, path: &Path) -> Result<FileEnumerator>
pub fn urls_for_directory(&self, directory: FileManagerDirectory, domain: FileManagerDomain) -> Result<Vec<PathBuf>>
```

## FileAttributes

```rust
pub struct FileAttributes {
    pub size: u64,
    pub created: Option<std::time::SystemTime>,
    pub modified: Option<std::time::SystemTime>,
    pub is_directory: bool,
    pub is_file: bool,
    pub readonly: bool,
}
```

## FileHandle

File read/write operations.

```rust
pub fn for_reading(path: &Path) -> Result<Self>
pub fn for_writing(path: &Path) -> Self
pub fn for_updating(path: &Path) -> Self
pub fn read_to_end(&self) -> Result<Vec<u8>>
pub fn read_to_string(&self) -> Result<String>
pub fn read_line(&self) -> Result<Option<String>>
pub fn write(&self, data: &[u8]) -> Result<()>
pub fn write_string(&self, data: &str) -> Result<()>
pub fn append(&self, data: &[u8]) -> Result<()>
pub fn file_size(&self) -> Result<u64>
pub fn synchronize(&self) -> Result<()>
```

Returns `Err` from `for_reading` when the file does not exist.

## Bundle

Application bundle information reader.

```rust
pub fn main() -> Self
pub fn from_path(path: &Path) -> Self
pub fn load_info_dictionary(&mut self) -> Result<()>
pub fn bundle_path(&self) -> &Path
pub fn bundle_identifier(&self) -> Option<&str>
pub fn bundle_name(&self) -> Option<&str>
pub fn bundle_version(&self) -> Option<&str>
pub fn bundle_short_version(&self) -> Option<&str>
pub fn executable_name(&self) -> Option<&str>
pub fn path_for_resource(&self, name: &str, extension: &str) -> Option<PathBuf>
pub fn resource_path(&self) -> PathBuf
```

Returns `None` from accessor methods when `load_info_dictionary` has not been called or the key is missing.

## Usage

```rust
use tontoo_foundation::prelude::*;
use std::path::Path;

let fm = FileManager::default();

// Standard directories
let home = FileManager::home_directory();
let docs = FileManager::document_directory();
let cache = FileManager::cache_directory();

// File operations
let path = Path::new("/tmp/test.txt");
fm.create_directory(Path::new("/tmp")).unwrap();
FileHandle::for_writing(path).write_string("hello").unwrap();

let content = FileHandle::for_reading(path).read_to_string().unwrap();
assert_eq!(content, "hello");

// Directory listing
let entries = fm.contents_of_directory(&home).unwrap();
```

## Cross References

- [URL.md](URL.md) - URL-based file operations
- [Serialization.md](Serialization.md) - Reading/writing structured data
