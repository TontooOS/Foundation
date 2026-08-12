# Predicate

Predicate and sort descriptor types providing Apple Foundation-like data filtering and sorting for TontooOS.

## API Overview

| Type | Description |
|---|---|
| `Predicate` | Boolean expression for filtering |
| `SortDescriptor` | Sort key with direction |
| `ComparisonPredicate` | Predicate with operator |
| `Expression` | Key-path or function expression |
| `PredicateValue` | Value type for evaluation |
| `PredicateOperator` | Comparison operator enum |
| `CompoundType` | AND/OR/NOT enum |
| `Sortable` | Trait for sort descriptor support |

## Predicate

Boolean expression that evaluates against a key-value object.

```rust
pub struct Predicate {
    format: String,
    operator: PredicateOperator,
    left: String,
    right: String,
    options: PredicateOptions,
    sub_predicates: Vec<Predicate>,
    compound_type: Option<CompoundType>,
}
```

### Constructor

```rust
pub fn from_format(format: &str) -> Result<Self>
pub fn with_left(self, left: &str) -> Self
pub fn with_right(self, right: &str) -> Self
pub fn with_operator(self, op: PredicateOperator) -> Self
pub fn with_options(self, options: PredicateOptions) -> Self
```

### Compound Predicates

```rust
pub fn not() -> Self
pub fn and(predicates: &[Predicate]) -> Self
pub fn or(predicates: &[Predicate]) -> Self
```

### Evaluation

```rust
pub fn evaluate_with(&self, object: &HashMap<String, PredicateValue>) -> bool
pub fn evaluate_with_object(&self, object: &HashMap<String, String>) -> bool
pub fn evaluate_compound(&self, object: &HashMap<String, PredicateValue>) -> bool
```

## PredicateOperator

| Variant | Description |
|---|---|
| `Equal` | `==` |
| `NotEqual` | `!=` |
| `LessThan` | `<` |
| `LessThanOrEqual` | `<=` |
| `GreaterThan` | `>` |
| `GreaterThanOrEqual` | `>=` |
| `Contains` | String contains |
| `BeginsWith` | String starts with |
| `EndsWith` | String ends with |
| `In` | Value in comma-separated list |
| `Matches` | Pattern match |

## PredicateValue

```rust
pub enum PredicateValue {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}
```

## SortDescriptor

Sort key with direction for array sorting.

```rust
pub fn new(key: &str, ascending: bool) -> Self
pub fn with_selector(self, selector: &str) -> Self
pub fn with_comparator<F>(self, f: F) -> Self
pub fn key(&self) -> &str
pub fn ascending(&self) -> bool
pub fn compare(&self, a: &str, b: &str) -> std::cmp::Ordering
```

### Sortable Trait

```rust
pub trait Sortable {
    fn sort_by_descriptors(&mut self, descriptors: &[SortDescriptor]);
}
```

Implemented for `Vec<HashMap<String, String>>`. Sorts by multiple keys with fallback.

## Expression

Key-path or function expression for predicate evaluation.

```rust
pub fn key_path(key_path: &str) -> Self
pub fn function(name: &str, arguments: &[Expression]) -> Self
pub fn evaluate_with(&self, object: &HashMap<String, PredicateValue>) -> Option<PredicateValue>
pub fn get_key_path(&self) -> &str
pub fn function_name(&self) -> Option<&str>
```

Built-in functions: `count`, `sum`, `avg`.

## Usage

```rust
use tontoo_foundation::prelude::*;
use crate::predicate::Sortable;

// Simple predicate
let mut pred = Predicate::new("age", PredicateOperator::GreaterThan, "18");
pred = pred.with_left("age").with_right("18");

let mut obj = std::collections::HashMap::new();
obj.insert("age".to_string(), PredicateValue::Number(25.0));
assert!(pred.evaluate_with(&obj));

// Compound predicate
let p1 = Predicate::new("active", PredicateOperator::Equal, "true");
let p1 = p1.with_left("active").with_right("true");
let p2 = Predicate::new("role", PredicateOperator::Equal, "admin");
let p2 = p2.with_left("role").with_right("admin");
let combined = Predicate::and(&[p1, p2]);

// Sort descriptors
let mut data: Vec<std::collections::HashMap<String, String>> = vec![
    vec![("name".to_string(), "Charlie".to_string())].into_iter().collect(),
    vec![("name".to_string(), "Alice".to_string())].into_iter().collect(),
];
let desc = SortDescriptor::new("name", true);
data.sort_by_descriptors(&[desc]);
assert_eq!(data[0].get("name"), Some(&"Alice".to_string()));
```

## Cross References

- [Collections.md](Collections.md) - Array and Dictionary types
