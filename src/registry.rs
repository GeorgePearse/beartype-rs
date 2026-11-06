//! The `Checker` trait and the registry that dispatches a hint to the checker that
//! understands it.
//!
//! ## Why a registry of trait objects, not one big enum
//!
//! Every category of typing construct (containers, unions, literals, protocols, ...) wants
//! to own its own matching and checking logic. A single `enum Hint { List(...), Union(...),
//! ... }` would need every category's variant added to the same enum and the same `match`,
//! which is exactly the file two people (or two agents) touch at once. A registry of
//! `Box<dyn Checker>` lets each category live in its own file and add itself to
//! `checkers::register_all` with one line - the only shared touchpoint, and a line, not a
//! match arm, so two additions next to each other merge cleanly.
//!
//! `matches` and `check` are split so the registry can find the right checker without that
//! checker doing any actual (possibly expensive, possibly recursive) checking work first.

use pyo3::prelude::*;
use pyo3::types::PyAny;

pub trait Checker: Send + Sync {
    /// Does this checker know how to handle `hint`? Must be cheap: every checker's
    /// `matches` runs, in registration order, until one returns `true`.
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool>;

    /// `value` is being checked against `hint`, which this checker has already claimed via
    /// `matches`. Recurse through `registry` for any nested hints (a `List[int]` checker
    /// calls `registry.check(py, int_hint, item)` per element) rather than assuming
    /// anything about which other checker will end up handling them.
    fn check(&self, py: Python<'_>, hint: &Bound<'_, PyAny>, value: &Bound<'_, PyAny>, registry: &Registry) -> PyResult<bool>;

    /// A short, human name for this category (e.g. `"union"`), used only in error messages
    /// and `repr` - never in dispatch.
    fn name(&self) -> &'static str;
}

pub struct Registry {
    checkers: Vec<Box<dyn Checker>>,
}

impl Registry {
    pub fn new(checkers: Vec<Box<dyn Checker>>) -> Self {
        Self { checkers }
    }

    /// Find the first checker that claims `hint` and ask it about `value`. A hint no
    /// checker claims is an error, not a silent pass: a typo'd or not-yet-supported hint
    /// should say so loudly rather than let anything through.
    pub fn check(&self, py: Python<'_>, hint: &Bound<'_, PyAny>, value: &Bound<'_, PyAny>) -> PyResult<bool> {
        for checker in &self.checkers {
            if checker.matches(hint)? {
                return checker.check(py, hint, value, self);
            }
        }
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
            "beartype_rs does not know how to check against hint {}",
            crate::introspect::describe(hint)
        )))
    }

    /// Like `check`, but reports "no checker claims this hint" as `false` instead of an
    /// error. Used by the recursive, structural categories (`Union`, `Optional`,
    /// `Annotated`'s wrapped type, ...) where "this branch wasn't it" is a normal outcome,
    /// not a configuration mistake - unlike a *top-level* unrecognised hint, which is.
    pub fn try_check(&self, py: Python<'_>, hint: &Bound<'_, PyAny>, value: &Bound<'_, PyAny>) -> PyResult<bool> {
        for checker in &self.checkers {
            if checker.matches(hint)? {
                return checker.check(py, hint, value, self);
            }
        }
        Ok(false)
    }
}
