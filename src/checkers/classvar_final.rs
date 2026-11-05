//! `typing.ClassVar[X]` and `typing.Final[X]`, including the bare (unsubscripted) form of
//! either (`x: ClassVar` / `x: Final` with no type argument).
//!
//! Both of these only matter to a static type checker - at runtime they are just wrappers
//! around the type they annotate - but a checker still has to recognise them and not
//! crash: a dataclass or `@beartype`-decorated class may well carry a `ClassVar[int]` or
//! `Final[str]` annotation, and the value behind it is checked the same as if the wrapper
//! weren't there.
//!
//! Confirmed in the REPL (`.venv/bin/python`), not guessed:
//!
//! ```text
//! >>> typing.get_origin(typing.ClassVar[int])   # typing.ClassVar
//! >>> typing.get_origin(typing.Final[int])      # typing.Final
//! >>> typing.get_args(typing.ClassVar[int])     # (int,)
//! >>> typing.get_args(typing.Final[int])        # (int,)
//! >>> typing.ClassVar[int].__origin__ is typing.ClassVar   # True
//! >>> typing.Final[int].__origin__ is typing.Final         # True
//! ```
//!
//! The *bare* forms (`typing.ClassVar`, `typing.Final` with no `[...]`) are a different
//! shape entirely: each is a `typing._SpecialForm` instance with no `__origin__` and no
//! `__args__` attribute at all (`typing.Final.__origin__` raises `AttributeError`, same for
//! `__args__`) - confirmed in the REPL. So `matches()` can't rely on `introspect::origin`
//! alone for the bare form; it has to also check whether `hint` *is* the `typing.ClassVar`
//! / `typing.Final` marker object itself. `check()` then treats a bare wrapper (no
//! `__args__`) as `Final`/`ClassVar` of `Any`, which always passes.

use pyo3::prelude::*;
use pyo3::types::PyAny;

use crate::introspect;
use crate::registry::{Checker, Registry};

pub struct ClassvarFinalChecker;

impl Checker for ClassvarFinalChecker {
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        let py = hint.py();
        let typing_mod = py.import_bound("typing")?;
        let classvar_marker = typing_mod.getattr("ClassVar")?;
        let final_marker = typing_mod.getattr("Final")?;

        // Subscripted form: `ClassVar[int]` / `Final[int]` report their origin as the
        // marker itself.
        if let Some(origin) = introspect::origin(hint)? {
            if origin.is(&classvar_marker) || origin.is(&final_marker) {
                return Ok(true);
            }
        }

        // Bare form: `hint` *is* `typing.ClassVar` / `typing.Final` with no subscript at
        // all, so there is no `__origin__` to read - compare identity directly.
        if hint.is(&classvar_marker) || hint.is(&final_marker) {
            return Ok(true);
        }

        Ok(false)
    }

    fn check(
        &self,
        py: Python<'_>,
        hint: &Bound<'_, PyAny>,
        value: &Bound<'_, PyAny>,
        registry: &Registry,
    ) -> PyResult<bool> {
        let wrapped = introspect::args(py, hint)?;
        match wrapped.first() {
            // `ClassVar[X]` / `Final[X]`: defer to whichever checker claims `X`.
            Some(inner) => registry.check(py, inner, value),
            // Bare `ClassVar` / `Final`: no argument means `Any`, which always passes.
            None => Ok(true),
        }
    }

    fn name(&self) -> &'static str {
        "classvar_final"
    }
}
