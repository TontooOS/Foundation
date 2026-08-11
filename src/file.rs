//! File Management – FileManager, FileHandle, Bundle

use crate::error::{FoundationError, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// NSFileManager equivalent
pub struct FileManager;

impl FileManager {
    pub fn default() -> Self {
        Self
    }

    pub fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    pub fn is_directory(&self, path: &Path) -> bool {
        path.is_dir()
    }

    pub fn create_directory(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path)?;
        Ok(())
    }

    pub fn create_directory_with_intermediates(&self, path: &Path) -> Result<()> {
        self.create_directory(path)
    }

    pub fn remove_item(&self, path: &Path) -> Result<()> {
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn copy_item(&self, from: &Path, to: &Path) -> Result<()> {
        if from.is_dir() {
            self.copy_dir_recursive(from, to)?;
        } else {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(from, to)?;
        }
        Ok(())
    }

    pub fn move_item(&self, from: &Path, to: &Path) -> Result<()> {
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(from, to)?;
        Ok(())
    }

    fn copy_dir_recursive(&self, from: &Path, to: &Path) -> Result<()> {
        fs::create_dir_all(to)?;
        for entry in fs::read_dir(from)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                self.copy_dir_recursive(&entry.path(), &to.join(entry.file_name()))?;
            } else {
                fs::copy(entry.path(), to.join(entry.file_name()))?;
            }
        }
        Ok(())
    }

    pub fn contents_of_directory(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(path)? {
            entries.push(entry?.path());
        }
        Ok(entries)
    }

    pub fn contents_of_directory_at_url(&self, url: &crate::url::URL) -> Result<Vec<PathBuf>> {
        self.contents_of_directory(Path::new(url.path()))
    }

    pub fn attributes_of_item(&self, path: &Path) -> Result<FileAttributes> {
        let metadata = fs::metadata(path)?;
        Ok(FileAttributes {
            size: metadata.len(),
            created: metadata.created().ok(),
            modified: metadata.modified().ok(),
            is_directory: metadata.is_dir(),
            is_file: metadata.is_file(),
            readonly: metadata.permissions().readonly(),
        })
    }

    pub fn home_directory() -> PathBuf {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
    }

    pub fn document_directory() -> PathBuf {
        dirs::document_dir().unwrap_or_else(|| {
            dirs::home_dir().unwrap_or_default().join("Documents")
        })
    }

    pub fn cache_directory() -> PathBuf {
        dirs::cache_dir().unwrap_or_else(|| {
            dirs::home_dir().unwrap_or_default().join(".cache")
        })
    }

    pub fn temp_directory() -> PathBuf {
        std::env::temp_dir()
    }

    pub fn application_support_directory() -> PathBuf {
        dirs::data_dir().unwrap_or_else(|| {
            dirs::home_dir().unwrap_or_default().join(".local/share")
        })
    }

    pub fn desktop_directory() -> PathBuf {
        dirs::desktop_dir().unwrap_or_else(|| {
            dirs::home_dir().unwrap_or_default().join("Desktop")
        })
    }

    pub fn downloads_directory() -> PathBuf {
        dirs::download_dir().unwrap_or_else(|| {
            dirs::home_dir().unwrap_or_default().join("Downloads")
        })
    }

    pub fn trash_directory() -> PathBuf {
        dirs::home_dir().unwrap_or_default().join(".local/share/Trash/files")
    }

    pub fn urls_for_directory(&self, directory: FileManagerDirectory, domain: FileManagerDomain) -> Result<Vec<PathBuf>> {
        let path = match directory {
            FileManagerDirectory::Document => Self::document_directory(),
            FileManagerDirectory::Cache => Self::cache_directory(),
            FileManagerDirectory::ApplicationSupport => Self::application_support_directory(),
            FileManagerDirectory::Desktop => Self::desktop_directory(),
            FileManagerDirectory::Downloads => Self::downloads_directory(),
            FileManagerDirectory::Home => Self::home_directory(),
            FileManagerDirectory::Temp => Self::temp_directory(),
        };
        Ok(vec![path])
    }

    pub fn enumerator_at_path(&self, path: &Path) -> Result<FileEnumerator> {
        FileEnumerator::new(path)
    }

    pub fn displayName_at_path(&self, path: &Path) -> String {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string()
    }

    pub fn components_to_display_for_path(&self, path: &Path) -> Vec<String> {
        path.components()
            .filter_map(|c| c.as_os_str().to_str().map(|s| s.to_string()))
            .collect()
    }

    pub fn is_deletable_file_at_path(&self, path: &Path) -> bool {
        path.exists()
    }

    pub fn is_executable_file_at_path(&self, path: &Path) -> bool {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::metadata(path)
                .map(|m| m.permissions().mode() & 0o111 != 0)
                .unwrap_or(false)
        }
        #[cfg(not(unix))]
        {
            path.exists()
        }
    }

    pub fn is_readable_file_at_path(&self, path: &Path) -> bool {
        path.exists() && fs::File::open(path).is_ok()
    }

    pub fn is_writable_file_at_path(&self, path: &Path) -> bool {
        path.exists() && fs::metadata(path)
            .map(|m| !m.permissions().readonly())
            .unwrap_or(false)
    }

    pub fn file_size(&self, path: &Path) -> Option<u64> {
        fs::metadata(path).ok().map(|m| m.len())
    }

    pub fn set_attributes(&self, path: &Path, attrs: &FileAttributes) -> Result<()> {
        if attrs.readonly {
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_readonly(true);
            fs::set_permissions(path, perms)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileManagerDirectory {
    Document,
    Cache,
    ApplicationSupport,
    Desktop,
    Downloads,
    Home,
    Temp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileManagerDomain {
    User,
    Local,
    Network,
    System,
}

#[derive(Debug, Clone)]
pub struct FileAttributes {
    pub size: u64,
    pub created: Option<std::time::SystemTime>,
    pub modified: Option<std::time::SystemTime>,
    pub is_directory: bool,
    pub is_file: bool,
    pub readonly: bool,
}

/// FileEnumerator – recursively enumerate directory contents
pub struct FileEnumerator {
    stack: Vec<PathBuf>,
    recursive: bool,
}

impl FileEnumerator {
    pub fn new(path: &Path) -> Result<Self> {
        if !path.is_dir() {
            return Err(FoundationError::NotFound(format!("Not a directory: {:?}", path)));
        }
        Ok(Self {
            stack: vec![path.to_path_buf()],
            recursive: true,
        })
    }

    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }
}

impl Iterator for FileEnumerator {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(current) = self.stack.pop() {
            if let Ok(entries) = fs::read_dir(&current) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() && self.recursive {
                        self.stack.push(path.clone());
                    }
                    return Some(path);
                }
            }
        }
        None
    }
}

/// FileHandle – read/write file operations
pub struct FileHandle {
    path: PathBuf,
}

impl FileHandle {
    pub fn for_reading(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(FoundationError::NotFound(format!("File not found: {:?}", path)));
        }
        Ok(Self { path: path.to_path_buf() })
    }

    pub fn for_writing(path: &Path) -> Self {
        Self { path: path.to_path_buf() }
    }

    pub fn for_updating(path: &Path) -> Self {
        Self { path: path.to_path_buf() }
    }

    pub fn read_to_end(&self) -> Result<Vec<u8>> {
        Ok(fs::read(&self.path)?)
    }

    pub fn read_to_string(&self) -> Result<String> {
        Ok(fs::read_to_string(&self.path)?)
    }

    pub fn read_line(&self) -> Result<Option<String>> {
        use std::io::BufRead;
        let file = fs::File::open(&self.path)?;
        let mut reader = std::io::BufReader::new(file);
        let mut line = String::new();
        match reader.read_line(&mut line)? {
            0 => Ok(None),
            _ => {
                if line.ends_with('\n') {
                    line.pop();
                }
                Ok(Some(line))
            }
        }
    }

    pub fn write(&self, data: &[u8]) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.path, data)?;
        Ok(())
    }

    pub fn write_string(&self, data: &str) -> Result<()> {
        self.write(data.as_bytes())
    }

    pub fn append(&self, data: &[u8]) -> Result<()> {
        use std::io::Write;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        file.write_all(data)?;
        Ok(())
    }

    pub fn seek_to_end(&self) -> Result<()> {
        Ok(())
    }

    pub fn close(self) -> Result<()> {
        Ok(())
    }

    pub fn file_size(&self) -> Result<u64> {
        Ok(fs::metadata(&self.path)?.len())
    }

    pub fn truncate_at_offset(&self, offset: u64) -> Result<()> {
        let file = fs::OpenOptions::new().write(true).open(&self.path)?;
        file.set_len(offset)?;
        Ok(())
    }

    pub fn offset(&self) -> Result<u64> {
        Ok(0)
    }

    pub fn synchronize(&self) -> Result<()> {
        Ok(())
    }
}

/// NSBundle equivalent
pub struct Bundle {
    path: PathBuf,
    info: Option<HashMap<String, String>>,
}

impl Bundle {
    pub fn main() -> Self {
        let exe = std::env::current_exe().unwrap_or_default();
        let path = exe.parent().unwrap_or(Path::new(".")).to_path_buf();
        Self { path, info: None }
    }

    pub fn from_path(path: &Path) -> Self {
        Self { path: path.to_path_buf(), info: None }
    }

    pub fn load_info_dictionary(&mut self) -> Result<()> {
        let info_plist = self.path.join("Info.plist");
        let info_json = self.path.join("Info.json");

        if info_plist.exists() {
            let content = fs::read_to_string(&info_plist)?;
            let plist: HashMap<String, String> = plist::from_file(&info_plist)
                .map_err(|e| FoundationError::InvalidPlist(e.to_string()))?;
            self.info = Some(plist.into_iter().map(|(k, v)| (k, v.to_string())).collect());
        } else if info_json.exists() {
            let content = fs::read_to_string(&info_json)?;
            let info: HashMap<String, String> = serde_json::from_str(&content)?;
            self.info = Some(info);
        }
        Ok(())
    }

    pub fn bundle_path(&self) -> &Path {
        &self.path
    }

    pub fn bundle_identifier(&self) -> Option<&str> {
        self.info.as_ref()?.get("CFBundleIdentifier").map(|s| s.as_str())
    }

    pub fn bundle_name(&self) -> Option<&str> {
        self.info.as_ref()?.get("CFBundleName").map(|s| s.as_str())
    }

    pub fn bundle_version(&self) -> Option<&str> {
        self.info.as_ref()?.get("CFBundleVersion").map(|s| s.as_str())
    }

    pub fn bundle_short_version(&self) -> Option<&str> {
        self.info.as_ref()?.get("CFBundleShortVersionString").map(|s| s.as_str())
    }

    pub fn executable_name(&self) -> Option<&str> {
        self.info.as_ref()?.get("CFBundleExecutable").map(|s| s.as_str())
    }

    pub fn path_for_resource(&self, name: &str, extension: &str) -> Option<PathBuf> {
        self.path.join("Resources")
            .join(format!("{}.{}", name, extension))
            .canonicalize().ok()
    }

    pub fn path_for_resource_of_type(&self, name: &str, ext: &str, subdir: Option<&str>) -> Option<PathBuf> {
        let mut path = self.path.join("Resources");
        if let Some(sd) = subdir {
            path = path.join(sd);
        }
        path = path.join(format!("{}.{}", name, ext));
        path.canonicalize().ok()
    }

    pub fn resource_path(&self) -> PathBuf {
        self.path.join("Resources")
    }

    pub fn private_frameworks_path(&self) -> PathBuf {
        self.path.join("Frameworks")
    }

    pub fn shared_frameworks_path(&self) -> PathBuf {
        self.path.join("SharedFrameworks")
    }

    pub fn built_in_plug_ins_path(&self) -> PathBuf {
        self.path.join("PlugIns")
    }

    pub fn shared_support_path(&self) -> PathBuf {
        self.path.join("SharedSupport")
    }

    pub fn localized_string_for_value(&self, key: &str, value: &str, table_name: Option<&str>) -> String {
        key.to_string()
    }

    pub fn app_store_receipt_url(&self) -> Option<PathBuf> {
        None
    }
}
