//! A bare string (a stringified forward reference, e.g. `"MyClass"` - common with
//! `from __future__ import annotations`) or a `typing.ForwardRef` instance.
//!
//! beartype_rs's own `@beartype` decorator already resolves these before they ever reach
//! a `Checker`: it calls `typing.get_type_hints(func)` up front (see
//! `python/beartype_rs/__init__.py`), which evaluates every stringified annotation against
//! the function's defining module before `is_bearable`/`die_if_unbearable` see it. So in
//! the normal decorator path this checker never fires. But a caller going through
//! `is_bearable`/`die_if_unbearable` directly - `is_bearable(value, "SomeClass")` or
//! `is_bearable(value, typing.ForwardRef("SomeClass"))` - bypasses that resolution
//! entirely. Without this checker that hint would either fall through to `plain_class`
//! (which would reject it: nothing is ever `isinstance` of a `str` *instance*, so every
//! value would wrongly fail) or hit the registry's generic "no checker claims this hint"
//! error. Neither is a sensible answer, so this checker exists to give one - or, when the
//! name genuinely cannot be resolved, a clear error instead of a silent `False`.
//!
//! REPL-verified (`.venv/bin/python`):
//! - `isinstance("MyClass", str)` is `True`; a bare string hint has no `__origin__` and no
//!   `__args__` (`getattr` on either raises `AttributeError`).
//! - `type(typing.ForwardRef("int"))` is `typing.ForwardRef`.
//! - `typing.ForwardRef("int").__forward_arg__` is `"int"` - the original string,
//!   unparsed (not yet evaluated into a type).
//! - `typing._eval_type(typing.ForwardRef("int"), {}, {})` resolves to `<class 'int'>` -
//!   it resolves even against *empty* global/local namespaces, because the `eval` it uses
//!   under the hood always has implicit access to the `builtins` module.
//! - `typing._eval_type(typing.ForwardRef("NotARealName"), {}, {})` raises `NameError`.
//!
//! Resolution here is deliberately narrow and does not call into `typing._eval_type` (a
//! private API) or `eval` at all: it only looks the name up as an attribute of the
//! `builtins` module directly (`"int"` -> `int`, `"list"` -> `list`). A name that refers
//! to a user-defined class (`"MyClass"`) is *not* resolvable this way - that requires the
//! defining module's globals/locals, which `is_bearable` has no access to and which is
//! exactly what `@beartype`'s own `typing.get_type_hints()` call already handles upstream
//! of here. Full forward-ref resolution is out of scope for this checker by design. An
//! unresolvable name raises `TypeError` rather than returning `Ok(false)`: per the
//! category's contract, a forward reference beartype_rs cannot resolve is a configuration
//! problem the caller should hear about loudly, not a silent type mismatch indistinguishable
//! from "the value just didn't match".

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyString};

use crate::registry::{Checker, Registry};

pub struct ForwardRefChecker;

impl ForwardRefChecker {
    /// The raw, unparsed forward-ref string carried by `hint`, whichever of the two
    /// shapes (`matches` already confirmed it is one of them) it is.
    fn forward_arg(hint: &Bound<'_, PyAny>) -> PyResult<String> {
        if hint.is_instance_of::<PyString>() {
            return hint.extract::<String>();
        }
        // Not a bare string, so `matches` only returned `true` because this is a
        // `typing.ForwardRef`; `__forward_arg__` is the original string it was built
        // from (confirmed above: unparsed, not yet evaluated).
        hint.getattr("__forward_arg__")?.extract::<String>()
    }
}

impl Checker for ForwardRefChecker {
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        if hint.is_instance_of::<PyString>() {
            return Ok(true);
        }
        let py = hint.py();
        let forward_ref_class = py.import_bound("typing")?.getattr("ForwardRef")?;
        hint.is_instance(&forward_ref_class)
    }

    fn check(
        &self,
        py: Python<'_>,
        hint: &Bound<'_, PyAny>,
        value: &Bound<'_, PyAny>,
        registry: &Registry,
    ) -> PyResult<bool> {
        let name = Self::forward_arg(hint)?;

        let builtins = py.import_bound("builtins")?;
        let resolved = builtins.getattr(name.as_str()).map_err(|_| {
            PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                "beartype_rs cannot resolve forward reference {name:?}: it is not a name \
                 in the `builtins` module, and resolving a forward reference against a \
                 defining module's own globals/locals is out of scope for is_bearable() \
                 (use @beartype, whose typing.get_type_hints() call resolves stringified \
                 annotations like this one before beartype_rs ever sees them)"
            ))
        })?;

        // Recurse properly, not via `try_check`: once the name is resolved, "the value
        // doesn't match the resolved type" is a normal `false`, but it still has to go
        // through the registry (not an inline `isinstance`) since the resolved name could
        // in principle be anything another checker owns, not just a plain class.
        registry.check(py, &resolved, value)
    }

    fn name(&self) -> &'static str {
        "forward_ref"
    }
}
