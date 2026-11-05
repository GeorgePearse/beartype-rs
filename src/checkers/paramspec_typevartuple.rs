//! `typing.ParamSpec` (PEP 612) and `typing.TypeVarTuple`/`typing.Unpack` (PEP 646) used
//! directly as a bare hint - e.g. `def f(x: P) -> None` or `def g(*args: *Ts) -> None`.
//!
//! This is deliberately *not* about `Callable[P, R]` (a `ParamSpec` standing in for a
//! callable's parameter list inside a `Callable` subscript) - that shape is out of scope
//! here and falls to (the limitations of) `callable_hint`'s own `matches()`/`check()`.
//! This checker only claims a `ParamSpec`/`TypeVarTuple`/`Unpack` appearing as a hint in
//! its own right.
//!
//! Verified in the REPL (`.venv/bin/python`, Python 3.13):
//!
//! ```text
//! >>> import typing
//! >>> P = typing.ParamSpec("P")
//! >>> Ts = typing.TypeVarTuple("Ts")
//! >>> type(P) is typing.ParamSpec
//! True
//! >>> type(Ts) is typing.TypeVarTuple
//! True
//! >>> U = typing.Unpack[Ts]
//! >>> type(U)
//! <class 'typing._UnpackGenericAlias'>
//! >>> typing.get_origin(U) is typing.Unpack
//! True
//! >>> U.__origin__
//! typing.Unpack
//! ```
//!
//! So, mirroring how a function signature actually produces these at runtime
//! (`typing.get_type_hints` on `def g(*args: *Ts) -> None` yields `{'args':
//! typing.Unpack[Ts], ...}` - a bare `Unpack[Ts]`, not a tuple wrapping it):
//!
//! - a bare `ParamSpec` instance has `type(hint) is typing.ParamSpec`;
//! - a bare `TypeVarTuple` instance has `type(hint) is typing.TypeVarTuple`;
//! - `Unpack[Ts]` (however it reached the hint - directly, or via `*args: *Ts` sugar) has
//!   `__origin__ is typing.Unpack`, exactly like `introspect::origin` already extracts for
//!   every other subscripted special form.
//!
//! `check()` is a deliberate, documented no-op: these are purely static, compile-time-only
//! constructs (a `ParamSpec` describes a *signature shape*; a `TypeVarTuple`/`Unpack`
//! describes an *arity*-polymorphic sequence of types) with no single runtime value they
//! could meaningfully be checked against. We do not pretend otherwise - this always
//! returns `Ok(true)` ("permissive pass"), not a real check, and that permissiveness is
//! the documented, intentional behaviour for this category rather than an oversight.

use pyo3::prelude::*;
use pyo3::types::PyAny;

use crate::introspect;
use crate::registry::{Checker, Registry};

pub struct ParamspecTypevartupleChecker;

impl Checker for ParamspecTypevartupleChecker {
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        let py = hint.py();
        let typing_mod = py.import_bound("typing")?;

        // Bare `typing.ParamSpec` instance (PEP 612).
        let param_spec_class = typing_mod.getattr("ParamSpec")?;
        if hint.is_exact_instance(&param_spec_class) {
            return Ok(true);
        }

        // Bare `typing.TypeVarTuple` instance (PEP 646).
        let type_var_tuple_class = typing_mod.getattr("TypeVarTuple")?;
        if hint.is_exact_instance(&type_var_tuple_class) {
            return Ok(true);
        }

        // `typing.Unpack[Ts]` (PEP 646), however it reached this hint - directly written,
        // or produced by `*args: *Ts` signature sugar. `introspect::origin` reads
        // `__origin__`, which for this special form is the `typing.Unpack` singleton
        // itself (verified above), the same pattern `literal`/`union_optional` use for
        // their own special forms.
        if let Some(origin) = introspect::origin(hint)? {
            let unpack_marker = typing_mod.getattr("Unpack")?;
            if origin.is(&unpack_marker) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn check(
        &self,
        _py: Python<'_>,
        _hint: &Bound<'_, PyAny>,
        _value: &Bound<'_, PyAny>,
        _registry: &Registry,
    ) -> PyResult<bool> {
        // Permissive no-op, documented above: `ParamSpec`/`TypeVarTuple`/`Unpack` as a
        // bare hint describes something only meaningful at the signature/static-typing
        // level, not a shape any single runtime `value` could satisfy or violate. There
        // is nothing to recurse into via `registry` here - no nested hint to extract from
        // either a `ParamSpec` or a `TypeVarTuple`/`Unpack` used this way - so this always
        // succeeds rather than silently (and incorrectly) rejecting every call that
        // mentions one.
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "paramspec_typevartuple"
    }
}
