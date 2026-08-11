//! Progress – NSProgress equivalent

use crate::error::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// NSProgress equivalent
pub struct Progress {
    total_unit_count: i64,
    completed_unit_count: i64,
    kind: Option<String>,
    estimated_time_remaining: Option<f64>,
    throughput: Option<f64>,
    file_operation_kind: Option<FileOperationKind>,
    file_total_count: Option<i64>,
    file_completed_count: Option<i64>,
    file_url: Option<String>,
    cancelled: bool,
    paused: bool,
    cancellable: bool,
    pausable: bool,
    cancellation_handler: Option<Box<dyn Fn() + Send>>,
    pausing_handler: Option<Box<dyn Fn() + Send>>,
    resuming_handler: Option<Box<dyn Fn() + Send>>,
    localized_description: Option<String>,
    localized_additional_description: Option<String>,
    user_info: HashMap<String, String>,
    parent: Option<Arc<Mutex<Progress>>>,
    children: Vec<Arc<Mutex<Progress>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileOperationKind {
    Downloading,
    DecompressingAfterDownloading,
    Receiving,
    Uploading,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            total_unit_count: 0,
            completed_unit_count: 0,
            kind: None,
            estimated_time_remaining: None,
            throughput: None,
            file_operation_kind: None,
            file_total_count: None,
            file_completed_count: None,
            file_url: None,
            cancelled: false,
            paused: false,
            cancellable: true,
            pausable: true,
            cancellation_handler: None,
            pausing_handler: None,
            resuming_handler: None,
            localized_description: None,
            localized_additional_description: None,
            user_info: HashMap::new(),
            parent: None,
            children: Vec::new(),
        }
    }

    pub fn with_total_unit_count(total: i64) -> Self {
        let mut p = Self::new();
        p.total_unit_count = total;
        p
    }

    pub fn current() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new()))
    }

    pub fn discrete_progress(total_unit_count: i64) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::with_total_unit_count(total_unit_count)))
    }

    pub fn progress_with_total_unit_count(total: i64, parent: Arc<Mutex<Progress>>) -> Arc<Mutex<Self>> {
        let mut child = Self::with_total_unit_count(total);
        child.parent = Some(parent.clone());
        let child_arc = Arc::new(Mutex::new(child));
        parent.lock().unwrap().add_child(child_arc.clone());
        child_arc
    }

    pub fn add_child(&mut self, child: Arc<Mutex<Progress>>) {
        self.children.push(child);
    }

    pub fn total_unit_count(&self) -> i64 {
        self.total_unit_count
    }

    pub fn set_total_unit_count(&mut self, count: i64) {
        self.total_unit_count = count;
    }

    pub fn completed_unit_count(&self) -> i64 {
        self.completed_unit_count
    }

    pub fn set_completed_unit_count(&mut self, count: i64) {
        self.completed_unit_count = count.min(self.total_unit_count);
    }

    pub fn increment_completed_unit_count(&mut self, increment: i64) {
        self.completed_unit_count = (self.completed_unit_count + increment).min(self.total_unit_count);
    }

    pub fn fraction_completed(&self) -> f64 {
        if self.total_unit_count == 0 {
            0.0
        } else {
            self.completed_unit_count as f64 / self.total_unit_count as f64
        }
    }

    pub fn is_finished(&self) -> bool {
        self.completed_unit_count >= self.total_unit_count
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
        if let Some(ref handler) = self.cancellation_handler {
            handler();
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn pause(&mut self) {
        self.paused = true;
        if let Some(ref handler) = self.pausing_handler {
            handler();
        }
    }

    pub fn resume(&mut self) {
        self.paused = false;
        if let Some(ref handler) = self.resuming_handler {
            handler();
        }
    }

    pub fn is_cancellable(&self) -> bool {
        self.cancellable
    }

    pub fn set_cancellable(&mut self, cancellable: bool) {
        self.cancellable = cancellable;
    }

    pub fn is_pausable(&self) -> bool {
        self.pausable
    }

    pub fn set_pausable(&mut self, pausable: bool) {
        self.pausable = pausable;
    }

    pub fn cancellation_handler(&self) -> Option<&(dyn Fn() + Send)> {
        self.cancellation_handler.as_deref()
    }

    pub fn set_cancellation_handler<F>(&mut self, handler: F)
    where
        F: Fn() + Send + 'static,
    {
        self.cancellation_handler = Some(Box::new(handler));
    }

    pub fn pausing_handler(&self) -> Option<&(dyn Fn() + Send)> {
        self.pausing_handler.as_deref()
    }

    pub fn set_pausing_handler<F>(&mut self, handler: F)
    where
        F: Fn() + Send + 'static,
    {
        self.pausing_handler = Some(Box::new(handler));
    }

    pub fn resuming_handler(&self) -> Option<&(dyn Fn() + Send)> {
        self.resuming_handler.as_deref()
    }

    pub fn set_resuming_handler<F>(&mut self, handler: F)
    where
        F: Fn() + Send + 'static,
    {
        self.resuming_handler = Some(Box::new(handler));
    }

    pub fn estimated_time_remaining(&self) -> Option<f64> {
        self.estimated_time_remaining
    }

    pub fn set_estimated_time_remaining(&mut self, time: Option<f64>) {
        self.estimated_time_remaining = time;
    }

    pub fn throughput(&self) -> Option<f64> {
        self.throughput
    }

    pub fn set_throughput(&mut self, throughput: Option<f64>) {
        self.throughput = throughput;
    }

    pub fn kind(&self) -> Option<&str> {
        self.kind.as_deref()
    }

    pub fn set_kind(&mut self, kind: Option<&str>) {
        self.kind = kind.map(|s| s.to_string());
    }

    pub fn file_operation_kind(&self) -> Option<FileOperationKind> {
        self.file_operation_kind
    }

    pub fn set_file_operation_kind(&mut self, kind: Option<FileOperationKind>) {
        self.file_operation_kind = kind;
    }

    pub fn file_total_count(&self) -> Option<i64> {
        self.file_total_count
    }

    pub fn set_file_total_count(&mut self, count: Option<i64>) {
        self.file_total_count = count;
    }

    pub fn file_completed_count(&self) -> Option<i64> {
        self.file_completed_count
    }

    pub fn set_file_completed_count(&mut self, count: Option<i64>) {
        self.file_completed_count = count;
    }

    pub fn file_url(&self) -> Option<&str> {
        self.file_url.as_deref()
    }

    pub fn set_file_url(&mut self, url: Option<&str>) {
        self.file_url = url.map(|s| s.to_string());
    }

    pub fn localized_description(&self) -> Option<&str> {
        self.localized_description.as_deref()
    }

    pub fn set_localized_description(&mut self, desc: Option<&str>) {
        self.localized_description = desc.map(|s| s.to_string());
    }

    pub fn localized_additional_description(&self) -> Option<&str> {
        self.localized_additional_description.as_deref()
    }

    pub fn set_localized_additional_description(&mut self, desc: Option<&str>) {
        self.localized_additional_description = desc.map(|s| s.to_string());
    }

    pub fn user_info(&self) -> &HashMap<String, String> {
        &self.user_info
    }

    pub fn user_info_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.user_info
    }

    pub fn set_user_info_object_for_key(&mut self, key: &str, value: &str) {
        self.user_info.insert(key.to_string(), value.to_string());
    }

    pub fn remove_user_info_object_for_key(&mut self, key: &str) {
        self.user_info.remove(key);
    }

    pub fn is_indeterminate(&self) -> bool {
        self.total_unit_count == 0
    }

    pub fn publish(&self) {
        // For KVO compliance, would notify observers
    }

    pub fn unpublish(&self) {
        // For KVO compliance, would remove observers
    }

    pub fn add_child_with_pending_unit_count(&mut self, _child: Arc<Mutex<Progress>>, _unit_count: i64) {
        // Implementation for child progress tracking
    }

    pub fn become_current_with_pending_unit_count(&mut self, _unit_count: i64) {
        // Implementation for parent progress tracking
    }

    pub fn resign_current(&mut self) {
        // Implementation for parent progress tracking
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Progress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pct = self.fraction_completed() * 100.0;
        write!(f, "{:.1}% ({}/{} units)", pct, self.completed_unit_count, self.total_unit_count)
    }
}
