//! `typing.Callable`, `typing.Callable[[int, str], bool]`, `collections.abc.Callable[...]`.
//!
//! Confirmed in the REPL (`.venv/bin/python`), not guessed:
//!
//! ```text
//! >>> import typing, collections.abc
//! >>> typing.get_origin(typing.Callable[[int, str], bool])
//! <class 'collections.abc.Callable'>
//! >>> typing.Callable[[int, str], bool].__origin__ is collections.abc.Callable
//! True
//! >>> typing.Callable.__origin__ is collections.abc.Callable   # bare `typing.Callable`
//! True
//! >>> typing.get_args(typing.Callable[[int, str], bool])
//! ([<class 'int'>, <class 'str'>], <class 'bool'>)
//! >>> typing.get_args(typing.Callable[..., int])
//! (Ellipsis, <class 'int'>)
//! >>> collections.abc.Callable[[int], int].__origin__ is collections.abc.Callable
//! True
//! >>> getattr(collections.abc.Callable, '__origin__', 'NONE')   # bare abc class itself
//! 'NONE'
//! ```
//!
//! So both `typing.Callable[...]` and the subscripted `collections.abc.Callable[...]`
//! carry `__origin__ is collections.abc.Callable`, as does bare (unsubscripted)
//! `typing.Callable` itself. The bare, unsubscripted `collections.abc.Callable` has *no*
//! `__origin__` at all - but it is itself a plain runtime class (an ABC with
//! `__subclasshook__`), so `introspect::is_plain_class` already claims it and
//! `isinstance(value, collections.abc.Callable)` does the right thing via the
//! `plain_class` fallback checker. This checker only needs to claim the `__origin__`
//! form.
//!
//! **Limitation (by design, matches real beartype's O(1) runtime guarantee)**: `check()`
//! only confirms `callable(value)` is true. It does *not* verify that `value`'s own
//! parameter types or return type match the `Callable[[...], ...]` signature - inspecting
//! an arbitrary callable's actual argument/return types at runtime (and verifying
//! sub-hints like `Callable[[int], int]` vs. what the callable actually accepts) is not
//! something Python's runtime exposes in general for e.g. builtins, C extensions, or
//! `functools.partial` objects, so real beartype also only enforces this structurally at
//! static-analysis time, not at runtime. We therefore do not recurse into the inner
//! argument/return hints via `registry.check`/`registry.try_check` here - there is no
//! runtime value to check them against.

use pyo3::prelude::*;
use pyo3::types::PyAny;

use crate::introspect;
use crate::registry::{Checker, Registry};

pub struct CallableHintChecker;

impl Checker for CallableHintChecker {
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        let py = hint.py();

        // `typing.Callable`, `typing.Callable[[int, str], bool]`, and
        // `collections.abc.Callable[...]` all carry `__origin__ is collections.abc.Callable`.
        // The bare `collections.abc.Callable` class itself has no `__origin__` and is left
        // to the `plain_class` fallback (isinstance already understands it).
        if let Some(origin) = introspect::origin(hint)? {
            let abc_mod = py.import_bound("collections.abc")?;
            let callable_marker = abc_mod.getattr("Callable")?;
            if origin.is(&callable_marker) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn check(
        &self,
        _py: Python<'_>,
        _hint: &Bound<'_, PyAny>,
        value: &Bound<'_, PyAny>,
        _registry: &Registry,
    ) -> PyResult<bool> {
        // See module docs: we deliberately do not inspect `value`'s own signature against
        // the hint's argument/return sub-hints, only that it is callable at all.
        Ok(value.is_callable())
    }

    fn name(&self) -> &'static str {
        "callable_hint"
    }
}
