//! `typing.Union[X, Y, ...]`, `typing.Optional[X]` (which is just `Union[X, None]` once
//! `typing.get_origin`/`typing.get_args` have been called on it - confirmed in the REPL,
//! not guessed), and PEP 604's `X | Y` syntax (`types.UnionType` on 3.10+).
//!
//! The two forms are *not* unified at the attribute level: a `typing.Union[...]` alias
//! carries `__origin__ is typing.Union`, while an `int | str` object has no `__origin__`
//! attribute at all (confirmed: accessing it raises `AttributeError`) - its identity is
//! carried by its own runtime type, `types.UnionType`, instead. So `matches()` has to check
//! both shapes explicitly; there is no single `__origin__` comparison that covers both.
//!
//! `check()` accepts if *any* branch accepts, short-circuiting on the first success, and
//! uses `registry.try_check` (not `registry.check`) per branch: a branch not matching the
//! value is the expected case for every branch but (at most) one, not a configuration
//! error the way an unrecognised *top-level* hint would be.

use pyo3::prelude::*;
use pyo3::types::PyAny;

use crate::introspect;
use crate::registry::{Checker, Registry};

pub struct UnionOptionalChecker;

impl Checker for UnionOptionalChecker {
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        let py = hint.py();

        // Form 1: `typing.Union[...]` / `typing.Optional[...]`. Both report
        // `__origin__ is typing.Union` (verified in the REPL: `typing.get_origin(typing.Optional[int])
        // is typing.Union` is `True`, same as for a plain `Union[int, str]`).
        if let Some(origin) = introspect::origin(hint)? {
            let typing_mod = py.import_bound("typing")?;
            let union_marker = typing_mod.getattr("Union")?;
            if origin.is(&union_marker) {
                return Ok(true);
            }
        }

        // Form 2: PEP 604's `X | Y`. `type(int | str) is types.UnionType`, and such a hint
        // has no `__origin__` of its own at all (verified: `(int | str).__origin__` raises
        // `AttributeError`), so this is a separate check, not a fallback of the first.
        let types_mod = py.import_bound("types")?;
        let union_type = types_mod.getattr("UnionType")?;
        hint.is_instance(&union_type)
    }

    fn check(
        &self,
        py: Python<'_>,
        hint: &Bound<'_, PyAny>,
        value: &Bound<'_, PyAny>,
        registry: &Registry,
    ) -> PyResult<bool> {
        let branches = introspect::args(py, hint)?;
        for branch in branches {
            if registry.try_check(py, &branch, value)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn name(&self) -> &'static str {
        "union_optional"
    }
}
