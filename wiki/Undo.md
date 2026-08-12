# Undo

Undo/redo management providing Apple Foundation-like UndoManager for TontooOS.

## API Overview

| Type | Description |
|---|---|
| `UndoManager` | Undo/redo stack manager |

## UndoManager

Manages undo and redo stacks with action names.

```rust
pub struct UndoManager {
    undo_stack: VecDeque<UndoAction>,
    redo_stack: VecDeque<UndoAction>,
    levels_of_undo: usize,
    is_undo_registration_enabled: bool,
    is_undoing: bool,
    is_redoing: bool,
}
```

### Constructor

```rust
pub fn new() -> Self
```

### Registering Actions

```rust
pub fn register_undo<F>(&mut self, target: &str, undo: F)
where
    F: FnMut() + Send + 'static
```

Registers an undo action. When `undo()` is called, the closure executes. The closure is `FnMut` so it can modify captured state.

### Action Name

```rust
pub fn set_action_name(&mut self, name: &str)
pub fn action_name(&self) -> Option<&str>
pub fn undo_menu_title(&self) -> String
pub fn redo_menu_title(&self) -> String
```

`set_action_name` sets the name for the most recently registered action. Menu titles return "Undo {name}" and "Redo {name}".

### Executing

```rust
pub fn undo(&mut self)
pub fn redo(&mut self)
```

`undo` pops from the undo stack, executes the action, and pushes to the redo stack. `redo` does the reverse. Does nothing if the respective stack is empty.

### State

```rust
pub fn can_undo(&self) -> bool
pub fn can_redo(&self) -> bool
pub fn undo_count(&self) -> usize
pub fn redo_count(&self) -> usize
pub fn is_undoing(&self) -> bool
pub fn is_redoing(&self) -> bool
```

### Cleanup

```rust
pub fn remove_all_actions(&mut self)
pub fn remove_all_actions_with_target(&mut self, target: &str)
```

### Configuration

```rust
pub fn set_levels_of_undo(&mut self, levels: usize)
pub fn levels_of_undo(&self) -> usize
pub fn is_undo_registration_enabled(&self) -> bool
pub fn disable_undo_registration(&mut self)
pub fn enable_undo_registration(&mut self)
```

When `levels_of_undo` is 0, there is no limit. When disabled, new undo registrations are ignored. Registering a new action while not undoing/redoing clears the redo stack.

## Usage

```rust
use tontoo_foundation::prelude::*;
use std::sync::Arc;

let mut manager = UndoManager::new();
let value = Arc::new(std::sync::Mutex::new(0));

// Register undo
let v = value.clone();
manager.register_undo("increment", move || {
    *v.lock().unwrap() -= 1;
});
manager.set_action_name("Increment");

// Execute undo
assert!(manager.can_undo());
manager.undo();
assert_eq!(*value.lock().unwrap(), -1);

// Redo
assert!(manager.can_redo());
manager.redo();
assert_eq!(*value.lock().unwrap(), 0);

// Clear
manager.remove_all_actions();
assert!(!manager.can_undo());
```

## Cross References

- [Threading.md](Threading.md) - Thread safety for undo operations
