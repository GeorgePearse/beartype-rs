//! `typing.TypedDict` subclasses.
//!
//! A `TypedDict` class is not a "plain class" in the `isinstance`-friendly sense even
//! though its metaclass (`typing._TypedDictMeta`) happens to subclass `type` - calling
//! `isinstance(value, SomeTypedDict)` raises `TypeError: TypedDict does not support
//! instance and class checks`. So this checker must claim a `TypedDict` hint before
//! `plain_class` ever gets a look at it (see `src/checkers/mod.rs`'s ordering).
//!
//! The real runtime shape of a value conforming to a `TypedDict` is just a plain
//! `dict`: `TypedDict` has no runtime enforcement of its own, it only carries metadata
//! (`__required_keys__`, `__optional_keys__`, `__annotations__`) that beartype-rs uses
//! to validate a `dict` value structurally.

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict};

use crate::registry::{Checker, Registry};

pub struct TypedDictChecker;

impl Checker for TypedDictChecker {
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        // `__required_keys__` is set by `typing._TypedDictMeta.__new__` on every
        // TypedDict class (and only on those) - a plain class or a `@dataclass` has no
        // such attribute. Checked in the REPL: `type(X)` is `typing._TypedDictMeta`,
        // which itself subclasses `type`, so a hint-level `isinstance(hint, type)`
        // check alone (what `introspect::is_plain_class` does) can't tell a TypedDict
        // apart from an ordinary class - this attribute can.
        hint.hasattr("__required_keys__")
    }

    fn check(
        &self,
        py: Python<'_>,
        hint: &Bound<'_, PyAny>,
        value: &Bound<'_, PyAny>,
        registry: &Registry,
    ) -> PyResult<bool> {
        // A TypedDict value is, at runtime, just a `dict`.
        let dict = match value.downcast::<PyDict>() {
            Ok(dict) => dict,
            Err(_) => return Ok(false),
        };

        // Every required key must be present.
        let required_keys = hint.getattr("__required_keys__")?;
        for key in required_keys.iter()? {
            let key = key?;
            let key_str: String = key.extract()?;
            if !dict.contains(key_str.as_str())? {
                return Ok(false);
            }
        }

        // Every key present in `value` that we have an annotation for must satisfy
        // that annotation's hint. Keys in `value` with no corresponding annotation
        // (not declared on the TypedDict at all) are ignored, matching real beartype's
        // structurally-open behaviour for TypedDict.
        let annotations = hint.getattr("__annotations__")?;
        let annotations = annotations.downcast::<PyDict>()?;

        for (key, nested_hint) in annotations.iter() {
            let key_str: String = key.extract()?;
            if let Some(item) = dict.get_item(key_str.as_str())? {
                if !registry.check(py, &nested_hint, &item)? {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    fn name(&self) -> &'static str {
        "typed_dict"
    }
}
