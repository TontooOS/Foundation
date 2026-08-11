//! Predicate & Sort Descriptor – NSPredicate, NSSortDescriptor

use crate::error::{FoundationError, Result};
use std::collections::HashMap;

/// NSPredicate equivalent
pub struct Predicate {
    format: String,
    operator: PredicateOperator,
    left: String,
    right: String,
    options: PredicateOptions,
    sub_predicates: Vec<Predicate>,
    compound_type: Option<CompoundType>,
}

impl Clone for Predicate {
    fn clone(&self) -> Self {
        Self {
            format: self.format.clone(),
            operator: self.operator,
            left: self.left.clone(),
            right: self.right.clone(),
            options: self.options,
            sub_predicates: self.sub_predicates.clone(),
            compound_type: self.compound_type,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredicateOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Between,
    Contains,
    BeginsWith,
    EndsWith,
    Like,
    Matches,
    In,
    CustomSelector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredicateOptions {
    None,
    CaseInsensitive,
    DiacriticInsensitive,
    Normalized,
    LocaleSensitive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompoundType {
    Not,
    And,
    Or,
}

/// Predicate value for evaluation
#[derive(Debug, Clone, PartialEq)]
pub enum PredicateValue {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

impl Predicate {
    pub fn from_format(format: &str) -> Result<Self> {
        Ok(Self {
            format: format.to_string(),
            operator: PredicateOperator::Equal,
            left: String::new(),
            right: String::new(),
            options: PredicateOptions::None,
            sub_predicates: Vec::new(),
            compound_type: None,
        })
    }

    pub fn with_left(mut self, left: &str) -> Self {
        self.left = left.to_string();
        self
    }

    pub fn with_right(mut self, right: &str) -> Self {
        self.right = right.to_string();
        self
    }

    pub fn with_operator(mut self, op: PredicateOperator) -> Self {
        self.operator = op;
        self
    }

    pub fn with_options(mut self, options: PredicateOptions) -> Self {
        self.options = options;
        self
    }

    pub fn evaluate_with(&self, object: &HashMap<String, PredicateValue>) -> bool {
        let left_val = object.get(&self.left);
        let right_val = match &self.right.as_str() {
            &"true" => PredicateValue::Bool(true),
            &"false" => PredicateValue::Bool(false),
            s if s.parse::<f64>().is_ok() => PredicateValue::Number(s.parse().unwrap()),
            s => PredicateValue::String(s.to_string()),
        };

        match (&self.operator, left_val) {
            (PredicateOperator::Equal, Some(left)) => *left == right_val,
            (PredicateOperator::NotEqual, Some(left)) => *left != right_val,
            (PredicateOperator::LessThan, Some(PredicateValue::Number(l))) => {
                if let PredicateValue::Number(r) = right_val {
                    l < &r
                } else {
                    false
                }
            }
            (PredicateOperator::LessThanOrEqual, Some(PredicateValue::Number(l))) => {
                if let PredicateValue::Number(r) = right_val {
                    l <= &r
                } else {
                    false
                }
            }
            (PredicateOperator::GreaterThan, Some(PredicateValue::Number(l))) => {
                if let PredicateValue::Number(r) = right_val {
                    l > &r
                } else {
                    false
                }
            }
            (PredicateOperator::GreaterThanOrEqual, Some(PredicateValue::Number(l))) => {
                if let PredicateValue::Number(r) = right_val {
                    l >= &r
                } else {
                    false
                }
            }
            (PredicateOperator::Contains, Some(PredicateValue::String(l))) => {
                if let PredicateValue::String(r) = &right_val {
                    l.contains(r)
                } else {
                    false
                }
            }
            (PredicateOperator::BeginsWith, Some(PredicateValue::String(l))) => {
                if let PredicateValue::String(r) = &right_val {
                    l.starts_with(r)
                } else {
                    false
                }
            }
            (PredicateOperator::EndsWith, Some(PredicateValue::String(l))) => {
                if let PredicateValue::String(r) = &right_val {
                    l.ends_with(r)
                } else {
                    false
                }
            }
            (PredicateOperator::In, Some(left)) => {
                let values: Vec<&str> = self.right.split(',').collect();
                values.iter().any(|v| PredicateValue::String(v.trim().to_string()) == *left)
            }
            (PredicateOperator::Matches, Some(PredicateValue::String(l))) => {
                if let PredicateValue::String(r) = &right_val {
                    l.contains(r)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn evaluate_with_object(&self, object: &HashMap<String, String>) -> bool {
        let converted: HashMap<String, PredicateValue> = object.iter()
            .map(|(k, v)| {
                let val = if let Ok(n) = v.parse::<f64>() {
                    PredicateValue::Number(n)
                } else if v == "true" {
                    PredicateValue::Bool(true)
                } else if v == "false" {
                    PredicateValue::Bool(false)
                } else {
                    PredicateValue::String(v.clone())
                };
                (k.clone(), val)
            })
            .collect();
        self.evaluate_with(&converted)
    }

    pub fn not() -> Self {
        Self {
            format: "NOT".to_string(),
            operator: PredicateOperator::Equal,
            left: String::new(),
            right: String::new(),
            options: PredicateOptions::None,
            sub_predicates: Vec::new(),
            compound_type: Some(CompoundType::Not),
        }
    }

    pub fn and(predicates: &[Predicate]) -> Self {
        Self {
            format: "AND".to_string(),
            operator: PredicateOperator::Equal,
            left: String::new(),
            right: String::new(),
            options: PredicateOptions::None,
            sub_predicates: predicates.to_vec(),
            compound_type: Some(CompoundType::And),
        }
    }

    pub fn or(predicates: &[Predicate]) -> Self {
        Self {
            format: "OR".to_string(),
            operator: PredicateOperator::Equal,
            left: String::new(),
            right: String::new(),
            options: PredicateOptions::None,
            sub_predicates: predicates.to_vec(),
            compound_type: Some(CompoundType::Or),
        }
    }

    pub fn evaluate_compound(&self, object: &HashMap<String, PredicateValue>) -> bool {
        match self.compound_type {
            Some(CompoundType::Not) => {
                if let Some(first) = self.sub_predicates.first() {
                    !first.evaluate_with(object)
                } else {
                    true
                }
            }
            Some(CompoundType::And) => {
                self.sub_predicates.iter().all(|p| p.evaluate_with(object))
            }
            Some(CompoundType::Or) => {
                self.sub_predicates.iter().any(|p| p.evaluate_with(object))
            }
            None => self.evaluate_with(object),
        }
    }

    pub fn format(&self) -> &str {
        &self.format
    }

    pub fn predicate_operator(&self) -> PredicateOperator {
        self.operator
    }

    pub fn sub_predicates(&self) -> &[Predicate] {
        &self.sub_predicates
    }
}

impl Default for Predicate {
    fn default() -> Self {
        Self::from_format("").unwrap()
    }
}

/// NSPredicateResult for compound predicates
pub struct PredicateResult {
    pub matched: bool,
    pub bindings: HashMap<String, PredicateValue>,
}

/// NSSortDescriptor equivalent
pub struct SortDescriptor {
    key: String,
    ascending: bool,
    selector: Option<String>,
    comparator: Option<Box<dyn Fn(&str, &str) -> std::cmp::Ordering>>,
}

impl SortDescriptor {
    pub fn new(key: &str, ascending: bool) -> Self {
        Self {
            key: key.to_string(),
            ascending,
            selector: None,
            comparator: None,
        }
    }

    pub fn with_selector(mut self, selector: &str) -> Self {
        self.selector = Some(selector.to_string());
        self
    }

    pub fn with_comparator<F>(mut self, f: F) -> Self
    where
        F: Fn(&str, &str) -> std::cmp::Ordering + 'static,
    {
        self.comparator = Some(Box::new(f));
        self
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn ascending(&self) -> bool {
        self.ascending
    }

    pub fn selector(&self) -> Option<&str> {
        self.selector.as_deref()
    }

    pub fn comparator(&self) -> Option<&dyn Fn(&str, &str) -> std::cmp::Ordering> {
        self.comparator.as_deref()
    }

    pub fn compare(&self, a: &str, b: &str) -> std::cmp::Ordering {
        let result = if let Some(ref cmp) = self.comparator {
            cmp(a, b)
        } else {
            a.cmp(b)
        };
        if self.ascending {
            result
        } else {
            result.reverse()
        }
    }
}

/// NSSortDescriptor extensions for array sorting
pub trait Sortable {
    fn sort_by_descriptors(&mut self, descriptors: &[SortDescriptor]);
}

impl Sortable for Vec<HashMap<String, String>> {
    fn sort_by_descriptors(&mut self, descriptors: &[SortDescriptor]) {
        self.sort_by(|a, b| {
            for desc in descriptors {
                let a_val = a.get(desc.key()).unwrap_or(&String::new()).clone();
                let b_val = b.get(desc.key()).unwrap_or(&String::new()).clone();
                let cmp = desc.compare(&a_val, &b_val);
                if cmp != std::cmp::Ordering::Equal {
                    return cmp;
                }
            }
            std::cmp::Ordering::Equal
        });
    }
}

/// NSComparisonPredicate equivalent
pub struct ComparisonPredicate {
    predicate: Predicate,
    left_expression: String,
    right_expression: String,
    modifier: ComparisonModifier,
    options: PredicateOptions,
    custom_predicate_fn: Option<Box<dyn Fn(&HashMap<String, PredicateValue>) -> bool>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonModifier {
    Direct,
    All,
    Any,
}

impl ComparisonPredicate {
    pub fn new(
        left: &str,
        operator: PredicateOperator,
        right: &str,
    ) -> Self {
        Self {
            predicate: Predicate::from_format("")
                .unwrap()
                .with_operator(operator)
                .with_left(left)
                .with_right(right),
            left_expression: left.to_string(),
            right_expression: right.to_string(),
            modifier: ComparisonModifier::Direct,
            options: PredicateOptions::None,
            custom_predicate_fn: None,
        }
    }

    pub fn evaluate(&self, object: &HashMap<String, PredicateValue>) -> bool {
        if let Some(ref custom) = self.custom_predicate_fn {
            custom(object)
        } else {
            self.predicate.evaluate_with(object)
        }
    }

    pub fn with_modifier(mut self, modifier: ComparisonModifier) -> Self {
        self.modifier = modifier;
        self
    }

    pub fn with_options(mut self, options: PredicateOptions) -> Self {
        self.options = options;
        self.predicate = self.predicate.with_options(options);
        self
    }

    pub fn with_custom_predicate<F>(mut self, f: F) -> Self
    where
        F: Fn(&HashMap<String, PredicateValue>) -> bool + 'static,
    {
        self.custom_predicate_fn = Some(Box::new(f));
        self
    }
}

impl Default for ComparisonPredicate {
    fn default() -> Self {
        Self::new("", PredicateOperator::Equal, "")
    }
}

/// NSExpression equivalent (simplified)
#[derive(Clone)]
pub struct Expression {
    key_path: String,
    function: Option<String>,
    arguments: Vec<Expression>,
}

impl Expression {
    pub fn key_path(key_path: &str) -> Self {
        Self {
            key_path: key_path.to_string(),
            function: None,
            arguments: Vec::new(),
        }
    }

    pub fn function(name: &str, arguments: &[Expression]) -> Self {
        Self {
            key_path: String::new(),
            function: Some(name.to_string()),
            arguments: arguments.to_vec(),
        }
    }

    pub fn evaluate_with(&self, object: &HashMap<String, PredicateValue>) -> Option<PredicateValue> {
        if let Some(ref func) = self.function {
            match func.as_str() {
                "count" => Some(PredicateValue::Number(self.arguments.len() as f64)),
                "sum" => {
                    let sum: f64 = self.arguments.iter()
                        .filter_map(|arg| arg.evaluate_with(object))
                        .filter_map(|v| match v {
                            PredicateValue::Number(n) => Some(n),
                            _ => None,
                        })
                        .sum();
                    Some(PredicateValue::Number(sum))
                }
                "avg" => {
                    let values: Vec<f64> = self.arguments.iter()
                        .filter_map(|arg| arg.evaluate_with(object))
                        .filter_map(|v| match v {
                            PredicateValue::Number(n) => Some(n),
                            _ => None,
                        })
                        .collect();
                    if values.is_empty() {
                        None
                    } else {
                        Some(PredicateValue::Number(values.iter().sum::<f64>() / values.len() as f64))
                    }
                }
                _ => None,
            }
        } else {
            object.get(&self.key_path).cloned()
        }
    }

    pub fn get_key_path(&self) -> &str {
        &self.key_path
    }

    pub fn function_name(&self) -> Option<&str> {
        self.function.as_deref()
    }
}
