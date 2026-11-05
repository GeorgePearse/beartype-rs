//! `collections.abc.Iterable[X]`/`Iterator[X]`/`Sized`/`Hashable`/`Container[X]`/
//! `Collection[X]`, and their `typing` aliases (`typing.Iterable[X]`, `typing.Sized`, ...).
//!
//! `typing.Iterable[int]` and `collections.abc.Iterable[int]` are the exact same object as
//! far as `__origin__` is concerned - both carry `__origin__ is collections.abc.Iterable`.
//! Even the *bare*, unsubscripted `typing` spellings (`typing.Sized`, `typing.Hashable`,
//! ...) carry `__origin__` pointing at the matching `collections.abc` class (unlike, say,
//! bare `dict`, which has no `__origin__` at all). The bare `collections.abc` classes
//! themselves (`collections.abc.Sized` used directly, with no subscript and no `typing`
//! indirection) have no `__origin__` - the hint *is* the class. So, as in
//! `containers_mapping.rs`, the marker to compare is `__origin__` when present, falling
//! back to the hint itself otherwise.
//!
//! `check()` is deliberately isinstance-only for the parameterized ones (`Iterable[X]`,
//! `Container[X]`, `Collection[X]`): unlike a `list[int]` or `set[int]`, these abstract
//! base classes make no promise that the value is a finite, re-iterable, already-realised
//! collection. `Iterator[X]` in particular can be an infinite generator
//! (`itertools.count()`) or a one-shot stream - consuming it to check every element would
//! both exhaust it (breaking the caller, who still wants to iterate it afterwards) and
//! potentially never terminate. `Iterable[X]`/`Collection[X]` are less dangerous (most are
//! finite and re-iterable) but still not guaranteed to be, and there is no principled way
//! to tell "safe to drain" apart from "will hang forever" from the hint alone. So the
//! element-type argument `X` is intentionally never inspected here, for any of the three -
//! only `isinstance(value, <the abc class>)` is checked. `Sized` and `Hashable` take no
//! element-type argument at all, so this is moot for them; `isinstance` is the whole
//! check either way.

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyModule};

use crate::introspect;
use crate::registry::{Checker, Registry};

pub struct AbcCollectionsChecker;

impl AbcCollectionsChecker {
    /// Is `candidate` *exactly* (by identity) one of
    /// `collections.abc.{Iterable,Iterator,Sized,Hashable,Container,Collection}`? Checked
    /// against a hint's `__origin__` when it has one (covers every subscripted form, and
    /// the bare `typing.X` aliases which carry `__origin__` even unsubscripted), or against
    /// the hint itself when it doesn't (covers the bare `collections.abc.X` classes used
    /// directly).
    fn abc_marker(&self, candidate: &Bound<'_, PyAny>) -> PyResult<bool> {
        let py = candidate.py();
        let abc = PyModule::import_bound(py, "collections.abc")?;
        for name in ["Iterable", "Iterator", "Sized", "Hashable", "Container", "Collection"] {
            let marker = abc.getattr(name)?;
            if candidate.is(&marker) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl Checker for AbcCollectionsChecker {
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        match introspect::origin(hint)? {
            // typing.Iterable[int], collections.abc.Iterable[int], bare typing.Sized, ...
            Some(origin) => self.abc_marker(&origin),
            // Bare collections.abc.Iterable/Iterator/Sized/Hashable/Container/Collection
            // used directly, with no typing indirection at all.
            None => self.abc_marker(hint),
        }
    }

    fn check(
        &self,
        py: Python<'_>,
        hint: &Bound<'_, PyAny>,
        value: &Bound<'_, PyAny>,
        _registry: &Registry,
    ) -> PyResult<bool> {
        // The runtime class to isinstance-check against: __origin__ for a (possibly bare)
        // typing-aliased form, or the hint itself for a bare collections.abc class.
        let class = match introspect::origin(hint)? {
            Some(origin) => origin,
            None => hint.clone(),
        };

        // Deliberately isinstance-only: see the module doc comment for why element-type
        // args (Iterable[X]'s X, etc.) are never recursed into here. `_registry` is unused
        // on purpose - there is nothing nested that is safe to check.
        let _ = py;
        introspect::isinstance_of_class(value, &class)
    }

    fn name(&self) -> &'static str {
        "abc_collections"
    }
}

#[cfg(test)]
mod tests {
    // Exercised end-to-end through the public Python API in
    // tests/test_abc_collections.py once this checker is wired into
    // `checkers::register_all` by the integration pass; no standalone Rust-only unit
    // tests here since every meaningful assertion needs a live `Python` GIL token plus a
    // registry containing `plain_class` (for the nested-type cases exercised elsewhere) -
    // consistent with the other category checkers in this crate.
}
