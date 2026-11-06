//! `typing.Any`, `typing.NoReturn`, `typing.Never` (PEP 484/646/...), and `typing.Self`
//! (PEP 673): four "special form" singletons that, at the value-check level this crate
//! operates at, are all unconditionally permissive.
//!
//! Confirmed in the REPL (`.venv/bin/python`), not guessed:
//!
//! ```text
//! >>> import typing
//! >>> typing.Any, type(typing.Any)            # typing.Any, <class 'typing._AnyMeta'>
//! >>> typing.NoReturn, type(typing.NoReturn)   # typing.NoReturn, <class 'typing._SpecialForm'>
//! >>> typing.Never, type(typing.Never)         # typing.Never, <class 'typing._SpecialForm'>
//! >>> typing.Self, type(typing.Self)           # typing.Self, <class 'typing._SpecialForm'>
//! >>> hasattr(typing.Any, "__origin__"), hasattr(typing.Any, "__args__")       # False, False
//! >>> hasattr(typing.NoReturn, "__origin__")                                   # False
//! >>> typing.Any is typing.Any                                                 # True (stable singleton)
//! >>> isinstance(typing.Any, type)             # True! typing._AnyMeta subclasses `type`.
//! ```
//!
//! That last line matters: `introspect::is_plain_class` (used by the `plain_class`
//! catch-all) tests `hint.is_instance_of::<PyType>()`, and `typing.Any` passes that test
//! despite not being a class anyone can meaningfully `isinstance()` a value against as a
//! *class* - `isinstance(3, typing.Any)` itself raises `TypeError` at runtime. So this
//! checker **must** be registered before `plain_class` in `register_all()`, or `Any`
//! would be misrouted into the catch-all and crash instead of permissively passing.
//!
//! None of the four have `__origin__`/`__args__`, so `matches()` can't use
//! `introspect::origin`/`introspect::args` - it has to compare `hint`'s identity directly
//! against the four marker objects pulled live from the `typing` module (mirroring the
//! bare-form branch of `classvar_final.rs`).
//!
//! `check()` is a deliberate, documented no-op returning `true` unconditionally:
//! - `Any` is permissive by definition - every value satisfies it.
//! - `NoReturn`/`Never` describe a function's return behaviour (it never returns) or an
//!   exhaustiveness marker, not a runtime value shape; there is no value to validate
//!   against them in the first place at this layer.
//! - `Self` refers to "whatever the enclosing class instance's type is" - checking that
//!   properly needs the enclosing class/call context (the bound method's defining class),
//!   which `Checker::check`'s `(hint, value)` signature does not carry. That is out of
//!   scope for this checker; treating it as permissive is documented here rather than
//!   silently assumed.

use pyo3::prelude::*;
use pyo3::types::PyAny;

use crate::registry::{Checker, Registry};

pub struct SelfAndSpecialFormsChecker;

impl Checker for SelfAndSpecialFormsChecker {
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        let py = hint.py();
        let typing_mod = py.import_bound("typing")?;
        let any_marker = typing_mod.getattr("Any")?;
        let no_return_marker = typing_mod.getattr("NoReturn")?;
        let never_marker = typing_mod.getattr("Never")?;
        let self_marker = typing_mod.getattr("Self")?;

        Ok(hint.is(&any_marker)
            || hint.is(&no_return_marker)
            || hint.is(&never_marker)
            || hint.is(&self_marker))
    }

    fn check(
        &self,
        _py: Python<'_>,
        _hint: &Bound<'_, PyAny>,
        _value: &Bound<'_, PyAny>,
        _registry: &Registry,
    ) -> PyResult<bool> {
        // Permissive no-op for all four forms - see the module doc comment for why each
        // one has no meaningful runtime value-level check at this layer.
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "self_and_special_forms"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::Python;

    #[test]
    fn matches_any_no_return_never_and_self() {
        Python::with_gil(|py| {
            let typing_mod = py.import_bound("typing").unwrap();
            let checker = SelfAndSpecialFormsChecker;

            for name in ["Any", "NoReturn", "Never", "Self"] {
                let marker = typing_mod.getattr(name).unwrap();
                assert!(
                    checker.matches(&marker).unwrap(),
                    "expected matches() to claim typing.{name}"
                );
            }
        });
    }

    #[test]
    fn does_not_match_a_plain_class() {
        Python::with_gil(|py| {
            let checker = SelfAndSpecialFormsChecker;
            let int_type = py.get_type_bound::<pyo3::types::PyInt>();
            assert!(!checker.matches(&int_type).unwrap());
        });
    }

    #[test]
    fn check_is_always_permissive() {
        Python::with_gil(|py| {
            let typing_mod = py.import_bound("typing").unwrap();
            let any_marker = typing_mod.getattr("Any").unwrap();
            let checker = SelfAndSpecialFormsChecker;
            let registry = Registry::new(vec![Box::new(SelfAndSpecialFormsChecker)]);
            let value = py.None();
            let value_bound = value.bind(py);
            assert!(checker
                .check(py, &any_marker, value_bound, &registry)
                .unwrap());
        });
    }
}
