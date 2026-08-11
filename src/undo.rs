//! Undo Manager – UndoManager

use std::collections::VecDeque;

/// NSUndoManager equivalent
pub struct UndoManager {
    undo_stack: VecDeque<UndoAction>,
    redo_stack: VecDeque<UndoAction>,
    levels_of_undo: usize,
    is_undo_registration_enabled: bool,
    is_undoing: bool,
    is_redoing: bool,
}

type UndoClosure = Box<dyn FnMut() + Send>;

pub struct UndoAction {
    target: String,
    label: String,
    action_name: String,
    undo_fn: UndoClosure,
}

impl UndoManager {
    pub fn new() -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            levels_of_undo: 0,
            is_undo_registration_enabled: true,
            is_undoing: false,
            is_redoing: false,
        }
    }

    pub fn register_undo<F>(&mut self, target: &str, mut undo: F)
    where
        F: FnMut() + Send + 'static,
    {
        if !self.is_undo_registration_enabled {
            return;
        }

        let action = UndoAction {
            target: target.to_string(),
            label: String::new(),
            action_name: String::new(),
            undo_fn: Box::new(undo),
        };

        self.undo_stack.push_back(action);
        if self.levels_of_undo > 0 && self.undo_stack.len() > self.levels_of_undo {
            self.undo_stack.pop_front();
        }

        if !self.is_undoing && !self.is_redoing {
            self.redo_stack.clear();
        }
    }

    pub fn set_action_name(&mut self, name: &str) {
        if let Some(action) = self.undo_stack.back_mut() {
            action.action_name = name.to_string();
        }
    }

    pub fn action_name(&self) -> Option<&str> {
        self.undo_stack.back().map(|a| a.action_name.as_str())
    }

    pub fn undo(&mut self) {
        if let Some(mut action) = self.undo_stack.pop_back() {
            self.is_undoing = true;
            (action.undo_fn)();
            self.redo_stack.push_back(action);
            self.is_undoing = false;
        }
    }

    pub fn redo(&mut self) {
        if let Some(mut action) = self.redo_stack.pop_back() {
            self.is_redoing = true;
            (action.undo_fn)();
            self.undo_stack.push_back(action);
            self.is_redoing = false;
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    pub fn remove_all_actions(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    pub fn remove_all_actions_with_target(&mut self, target: &str) {
        self.undo_stack.retain(|a| a.target != target);
        self.redo_stack.retain(|a| a.target != target);
    }

    pub fn set_levels_of_undo(&mut self, levels: usize) {
        self.levels_of_undo = levels;
    }

    pub fn levels_of_undo(&self) -> usize {
        self.levels_of_undo
    }

    pub fn is_undo_registration_enabled(&self) -> bool {
        self.is_undo_registration_enabled
    }

    pub fn disable_undo_registration(&mut self) {
        self.is_undo_registration_enabled = false;
    }

    pub fn enable_undo_registration(&mut self) {
        self.is_undo_registration_enabled = true;
    }

    pub fn is_undoing(&self) -> bool {
        self.is_undoing
    }

    pub fn is_redoing(&self) -> bool {
        self.is_redoing
    }

    pub fn undo_menu_title(&self) -> String {
        let name = self.action_name().unwrap_or("Action");
        let mut s = String::from("Undo ");
        s.push_str(name);
        s
    }

    pub fn redo_menu_title(&self) -> String {
        let name = self.action_name().unwrap_or("Action");
        let mut s = String::from("Redo ");
        s.push_str(name);
        s
    }
}

impl Default for UndoManager {
    fn default() -> Self {
        Self::new()
    }
}
