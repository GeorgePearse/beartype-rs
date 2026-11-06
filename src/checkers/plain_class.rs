//! The catch-all: a hint that is just a runtime class (`int`, `MyClass`, ...), which
//! `isinstance` already knows how to check. Every more specific category (containers,
//! unions, ...) should claim its hints before this one ever sees them - this is the
//! fallback, not a shortcut around writing a real checker.

use pyo3::prelude::*;
use pyo3::types::PyAny;

use crate::introspect;
use crate::registry::{Checker, Registry};

pub struct PlainClassChecker;

impl Checker for PlainClassChecker {
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        introspect::is_plain_class(hint)
    }

    fn check(&self, _py: Python<'_>, hint: &Bound<'_, PyAny>, value: &Bound<'_, PyAny>, _registry: &Registry) -> PyResult<bool> {
        introspect::isinstance_of_class(value, hint)
    }

    fn name(&self) -> &'static str {
        "plain_class"
    }
}
