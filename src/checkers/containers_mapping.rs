//! Mapping containers: bare `dict`, `typing.Dict[K, V]`/`dict[K, V]`,
//! `typing.Mapping[K, V]`/`collections.abc.Mapping`, and
//! `typing.MutableMapping[K, V]`/`collections.abc.MutableMapping`.
//!
//! All five spellings boil down to the same two questions once `__origin__` (or the bare
//! hint itself, when there is no `__origin__`) is one of `dict`, `collections.abc.Mapping`,
//! or `collections.abc.MutableMapping`:
//!   1. is `value` an instance of that class at all, and
//!   2. if the hint carries `(K, V)` `__args__`, does every `(k, v)` pair in `value.items()`
//!      satisfy `K` and `V` respectively.

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict, PyModule};

use crate::introspect;
use crate::registry::{Checker, Registry};

pub struct ContainersMappingChecker;

impl ContainersMappingChecker {
    /// Is `candidate` *exactly* (by identity, not subclassing) `dict`,
    /// `collections.abc.Mapping`, or `collections.abc.MutableMapping`? This is checked
    /// against both a hint's `__origin__` (the parameterized case, `Dict[K, V]`) and a bare
    /// hint with no `__args__` at all (`dict`, `collections.abc.Mapping` on their own).
    fn is_mapping_marker(&self, candidate: &Bound<'_, PyAny>) -> PyResult<bool> {
        let py = candidate.py();

        let dict_type = py.get_type_bound::<PyDict>();
        if candidate.is(&dict_type) {
            return Ok(true);
        }

        let abc = PyModule::import_bound(py, "collections.abc")?;
        let mapping = abc.getattr("Mapping")?;
        if candidate.is(&mapping) {
            return Ok(true);
        }
        let mutable_mapping = abc.getattr("MutableMapping")?;
        if candidate.is(&mutable_mapping) {
            return Ok(true);
        }

        Ok(false)
    }
}

impl Checker for ContainersMappingChecker {
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        match introspect::origin(hint)? {
            // Parameterized form: typing.Dict[K, V], dict[K, V], typing.Mapping[K, V],
            // collections.abc.MutableMapping (subscripted), ...
            Some(origin) => self.is_mapping_marker(&origin),
            // No __origin__ at all: the hint might still be the bare class itself, e.g.
            // plain `dict` or `collections.abc.Mapping` used without a subscript.
            None => self.is_mapping_marker(hint),
        }
    }

    fn check(
        &self,
        py: Python<'_>,
        hint: &Bound<'_, PyAny>,
        value: &Bound<'_, PyAny>,
        registry: &Registry,
    ) -> PyResult<bool> {
        // The class to isinstance-check against is __origin__ for a parameterized hint, or
        // the hint itself for a bare one.
        let class = match introspect::origin(hint)? {
            Some(origin) => origin,
            None => hint.clone(),
        };
        if !introspect::isinstance_of_class(value, &class)? {
            return Ok(false);
        }

        let args = introspect::args(py, hint)?;
        if args.is_empty() {
            // Bare dict / Mapping / MutableMapping: isinstance is the whole story, there is
            // no (K, V) to check elements against.
            return Ok(true);
        }
        // A real Dict/Mapping/MutableMapping hint always carries exactly two args (K, V). If
        // it somehow doesn't, there is nothing sensible left to check.
        if args.len() != 2 {
            return Ok(false);
        }
        let key_hint = &args[0];
        let value_hint = &args[1];

        // `value` is already known (via the isinstance check above) to be mapping-like, but
        // not necessarily a concrete `dict` - go through the mapping protocol's `.items()`
        // rather than assuming any particular implementation.
        let items = value.call_method0("items")?;
        for pair in items.iter()? {
            let pair = pair?;
            let k = pair.get_item(0)?;
            let v = pair.get_item(1)?;
            if !registry.check(py, key_hint, &k)? {
                return Ok(false);
            }
            if !registry.check(py, value_hint, &v)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn name(&self) -> &'static str {
        "containers_mapping"
    }
}
