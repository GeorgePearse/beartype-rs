//! `typing.Protocol` subclasses decorated with `@typing.runtime_checkable`.
//!
//! A runtime-checkable Protocol class *is* a plain runtime class as far as
//! `introspect::is_plain_class` is concerned (`isinstance(Proto, type)` is `True`, confirmed
//! in the REPL), so this checker has to run before `plain_class` or the latter would claim
//! it first and fall back to a structural-vs-nominal `isinstance` call that happens to work
//! here only by coincidence (Python's `isinstance` already special-cases runtime-checkable
//! Protocols). We still want our own category so the distinction is explicit and so a
//! non-runtime-checkable Protocol (which raises `TypeError` from `isinstance`) is rejected
//! at `matches()` time instead of surfacing a confusing error from `check()`.
//!
//! Confirmed in the REPL (see ARCHITECTURE.md's "Finding the right marker" section):
//! - `isinstance(Proto, type)` is `True` for a `Protocol` subclass - it's a real class.
//! - `issubclass(Proto, typing.Protocol)` is `True` for any `Protocol` subclass, runtime
//!   checkable or not.
//! - `@typing.runtime_checkable` sets `Proto._is_runtime_protocol = True`; a `Protocol`
//!   subclass that was never decorated has no such attribute (so `getattr(..., False)`
//!   reads `False`), even though it still has `_is_protocol = True`.
//! - `typing.Protocol` itself: `issubclass(typing.Protocol, typing.Protocol)` is `True`, but
//!   it has no `_is_runtime_protocol` attribute (not decorated), so `matches()` correctly
//!   rejects the bare `typing.Protocol` special form itself, not just non-decorated
//!   subclasses.
//! - `isinstance(value, Proto)` for a decorated Protocol does real structural checking
//!   (checks for the presence of the methods/attributes `Proto` declares), which is exactly
//!   what `check()` should defer to.

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyModule};

use crate::introspect;
use crate::registry::{Checker, Registry};

pub struct ProtocolChecker;

impl Checker for ProtocolChecker {
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        // Must be a real class first: `issubclass`/`getattr` below assume that, and a
        // non-class hint (e.g. `List[int]`) should just fall through to `Ok(false)` rather
        // than risk `issubclass` raising on a non-class first argument.
        if !introspect::is_plain_class(hint)? {
            return Ok(false);
        }

        let py = hint.py();
        let typing_mod = PyModule::import_bound(py, "typing")?;
        let protocol_marker = typing_mod.getattr("Protocol")?;

        // `issubclass(hint, typing.Protocol)` - both arguments are classes at this point, so
        // this cannot raise the "issubclass() arg 1 must be a class" TypeError. `issubclass`
        // is a builtin, not a `typing` attribute, so it comes from the `builtins` module.
        let builtins_mod = PyModule::import_bound(py, "builtins")?;
        let issubclass_fn = builtins_mod.getattr("issubclass")?;
        let is_protocol_subclass: bool = issubclass_fn.call1((hint, &protocol_marker))?.extract()?;

        if !is_protocol_subclass {
            return Ok(false);
        }

        // `runtime_checkable` is what makes `isinstance(value, hint)` legal for a Protocol
        // at all; without it Python's own `isinstance` raises `TypeError`. `getattr(...,
        // False)` mirrors the description's own check and also covers `typing.Protocol`
        // itself, which is a "subclass" of itself but was never decorated.
        let is_runtime_protocol = hint
            .getattr("_is_runtime_protocol")
            .map(|value| value.is_truthy().unwrap_or(false))
            .unwrap_or(false);

        Ok(is_runtime_protocol)
    }

    fn check(&self, _py: Python<'_>, hint: &Bound<'_, PyAny>, value: &Bound<'_, PyAny>, _registry: &Registry) -> PyResult<bool> {
        // Python's own `isinstance` already implements runtime_checkable Protocol's
        // structural check (does `value` have every method/attribute `hint` declares) -
        // `matches()` is what carves this case out from `plain_class`'s nominal `isinstance`
        // fallback; `check()` itself is just the same call, since it's correct for both.
        value.is_instance(hint)
    }

    fn name(&self) -> &'static str {
        "protocol"
    }
}
