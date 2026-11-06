//! `typing.Annotated[X, metadata, ...]` (PEP 593).
//!
//! The marker: an `Annotated` hint carries a `__metadata__` attribute (a tuple of the
//! extra, non-type payloads passed after the wrapped type) that no other hint shape
//! carries - verified in the REPL: `typing.Annotated[int, "meta"].__metadata__ ==
//! ('meta',)`, and plain `int`/`list[int]`/etc. don't have the attribute at all. That
//! makes `hasattr(hint, "__metadata__")` a safe, cheap `matches()` test.
//!
//! The trap: `Annotated`'s own `__origin__` is NOT the container-class sense of
//! `__origin__` every other category uses (where `__origin__` of `List[int]` is the
//! class `list`). For `Annotated`, `__origin__` is the *entire wrapped hint* instead -
//! verified in the REPL:
//!
//! ```text
//! >>> typing.Annotated[int, "meta"].__origin__
//! <class 'int'>
//! >>> typing.Annotated[typing.List[int], "x"].__origin__
//! typing.List[int]
//! >>> typing.Annotated[int, "meta"].__args__
//! (<class 'int'>,)          # NOT (int, "meta") - that's typing.get_args(hint), which
//!                           # special-cases Annotated to append the metadata back on
//! ```
//!
//! So this checker reads `__origin__` straight off the hint with `hint.getattr(...)`
//! rather than going through `crate::introspect::origin`, specifically to keep the
//! "origin means the wrapped hint, not a container class" distinction visible at the
//! call site instead of hiding it behind a helper whose name suggests the other meaning.
//! (Using the shared helper would actually give the same value here - it also just reads
//! `__origin__` - but spelling it out avoids a future reader assuming this `origin` means
//! what it means in `containers_*.rs`.)
//!
//! `check()` strips the metadata entirely and recurses on the wrapped hint: this project
//! does not implement beartype's "metadata entries that are themselves runtime
//! validators" extension (e.g. `Annotated[int, Is[lambda x: x > 0]]`), so any validators
//! present in `__metadata__` are silently ignored rather than evaluated. That's a real
//! limitation, not an oversight - it means `Annotated[int, SomeValidator(...)]` currently
//! behaves exactly like `int`.

use pyo3::prelude::*;
use pyo3::types::PyAny;

use crate::registry::{Checker, Registry};

pub struct AnnotatedChecker;

impl Checker for AnnotatedChecker {
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        hint.hasattr("__metadata__")
    }

    fn check(
        &self,
        py: Python<'_>,
        hint: &Bound<'_, PyAny>,
        value: &Bound<'_, PyAny>,
        registry: &Registry,
    ) -> PyResult<bool> {
        // Deliberately `hint.getattr("__origin__")`, not `introspect::origin(hint)`: see
        // the module doc comment above. Same attribute read either way, but spelled out
        // here so it reads as "the wrapped hint", not "the container class".
        let wrapped_hint = hint.getattr("__origin__")?;
        registry.check(py, &wrapped_hint, value)
    }

    fn name(&self) -> &'static str {
        "annotated"
    }
}
