//! The fixed-arity tuple form: `Tuple[int, str, float]` / `tuple[int, str]` - a specific
//! per-position sequence of hints with no trailing `Ellipsis`. This also covers the
//! empty-tuple hint `Tuple[()]` / `tuple[()]`, whose `__args__` is an empty tuple on
//! modern Python (verified in the REPL against this repo's `.venv`: Python 3.13 gives
//! `()` for both spellings) but was historically `((),)` - a single element that is
//! itself an empty tuple - on older `typing.Tuple[()]`. We accept either shape so this
//! checker isn't silently version-dependent.
//!
//! This is deliberately a different checker from `containers_sequence`: a sequence hint
//! (`List[int]`, `Tuple[int, ...]`) applies ONE hint to every element; this one applies a
//! DIFFERENT hint to each position, so `check` must zip element-by-element against the
//! corresponding positional hint rather than broadcast a single hint.

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyTuple};

use crate::introspect;
use crate::registry::{Checker, Registry};

/// The per-position hints for a fixed-arity tuple hint, or `Some(vec![])` for the
/// empty-tuple sentinel (`Tuple[()]`/`tuple[()]`, in either `__args__` shape Python has
/// used for it), or `None` if `hint`'s `__args__` doesn't describe a fixed-arity tuple at
/// all (variadic `Tuple[int, ...]`, or no `__args__` present at all - a bare `tuple` or
/// bare `typing.Tuple`).
fn fixed_arity_hints<'py>(hint: &Bound<'py, PyAny>) -> PyResult<Option<Vec<Bound<'py, PyAny>>>> {
    // Bare `tuple` / bare `typing.Tuple` carry no `__args__` attribute at all - that's
    // "no fixed arity known", not "fixed arity of zero", so they must not match here.
    let raw_args = match hint.getattr("__args__") {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    let args_tuple: Bound<'py, PyTuple> = raw_args.extract()?;

    if args_tuple.is_empty() {
        // Modern-Python `Tuple[()]` / `tuple[()]`: __args__ == ().
        return Ok(Some(Vec::new()));
    }

    if args_tuple.len() == 1 {
        // Older-Python `typing.Tuple[()]`: __args__ == ((),) - one element which is
        // itself an empty tuple.
        if let Ok(inner) = args_tuple.get_item(0)?.extract::<Bound<'py, PyTuple>>() {
            if inner.is_empty() {
                return Ok(Some(Vec::new()));
            }
        }
    }

    // Variadic form (`Tuple[int, ...]`) ends in Ellipsis - that belongs to
    // containers_sequence, not us.
    let last = args_tuple.get_item(args_tuple.len() - 1)?;
    if last.is_ellipsis() {
        return Ok(None);
    }

    Ok(Some(args_tuple.iter().collect()))
}

pub struct ContainersFixedTupleChecker;

impl Checker for ContainersFixedTupleChecker {
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        let py = hint.py();

        let origin = match introspect::origin(hint)? {
            Some(o) => o,
            None => return Ok(false),
        };
        if !origin.is(&py.get_type_bound::<PyTuple>()) {
            return Ok(false);
        }

        Ok(fixed_arity_hints(hint)?.is_some())
    }

    fn check(&self, py: Python<'_>, hint: &Bound<'_, PyAny>, value: &Bound<'_, PyAny>, registry: &Registry) -> PyResult<bool> {
        if !value.is_instance_of::<PyTuple>() {
            return Ok(false);
        }

        // `matches` already confirmed this hint is one of ours, so this is always
        // `Some`; treat a surprise `None` as "doesn't match" rather than panicking.
        let per_position_hints = match fixed_arity_hints(hint)? {
            Some(hints) => hints,
            None => return Ok(false),
        };

        let value_tuple: Bound<'_, PyTuple> = value.extract()?;
        if value_tuple.len() != per_position_hints.len() {
            return Ok(false);
        }

        for (element_hint, element) in per_position_hints.iter().zip(value_tuple.iter()) {
            if !registry.check(py, element_hint, &element)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn name(&self) -> &'static str {
        "containers_fixed_tuple"
    }
}
