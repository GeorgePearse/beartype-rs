//! Homogeneous sequences: bare `list`, `typing.List[X]`/`list[X]`,
//! `typing.Sequence[X]`/`collections.abc.Sequence[X]`, and the *variable*-arity tuple
//! form `Tuple[X, ...]`/`tuple[X, ...]` (exactly one type argument followed by
//! `Ellipsis` in `__args__`).
//!
//! The *fixed*-arity tuple form (`Tuple[X, Y, Z]`) is a different category - our
//! `matches()` is careful to check that `__args__` actually ends in `Ellipsis` before
//! claiming a `tuple`-origin hint, so it never steals that one.

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyList, PyTuple};

use crate::introspect;
use crate::registry::{Checker, Registry};

pub struct ContainersSequenceChecker;

impl ContainersSequenceChecker {
    /// The `list` type object, as a `Bound<PyAny>` so it compares/`isinstance`s like any
    /// other hint our introspect helpers hand back.
    fn list_type<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        py.get_type_bound::<PyList>().into_any()
    }

    /// The `tuple` type object.
    fn tuple_type<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        py.get_type_bound::<PyTuple>().into_any()
    }

    /// `collections.abc.Sequence` - not a PyO3-native type, so fetched by import rather
    /// than `py.get_type_bound`.
    fn sequence_abc<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        py.import_bound("collections.abc")?.getattr("Sequence")
    }

    /// Whether `hint`'s `__origin__` is `tuple` *and* its `__args__` is the
    /// variable-arity shape `(X, Ellipsis)` - exactly one type argument followed by
    /// `...`. Fixed-arity `Tuple[X, Y, Z]` has `__args__` of a different shape (no
    /// trailing `Ellipsis`, or more/fewer than two entries) and must return `false`
    /// here so that category's checker gets to claim it instead.
    fn is_variadic_tuple_args(&self, py: Python<'_>, args: &[Bound<'_, PyAny>]) -> bool {
        match args {
            [_, last] => last.is(&py.Ellipsis()),
            _ => false,
        }
    }
}

impl Checker for ContainersSequenceChecker {
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        let py = hint.py();
        let list_ty = self.list_type(py);

        // A bare `list` with no subscript at all carries no `__origin__` - it IS the
        // origin.
        if hint.is(&list_ty) {
            return Ok(true);
        }

        let origin = match introspect::origin(hint)? {
            Some(origin) => origin,
            None => return Ok(false),
        };

        if origin.is(&list_ty) {
            // `typing.List[X]` / `list[X]`.
            return Ok(true);
        }

        let seq_ty = self.sequence_abc(py)?;
        if origin.is(&seq_ty) {
            // `typing.Sequence[X]` / `collections.abc.Sequence[X]` (and the bare,
            // unsubscripted `typing.Sequence`, whose `__origin__` is also the abc).
            return Ok(true);
        }

        let tuple_ty = self.tuple_type(py);
        if origin.is(&tuple_ty) {
            let args = introspect::args(py, hint)?;
            return Ok(self.is_variadic_tuple_args(py, &args));
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
        let list_ty = self.list_type(py);

        // Bare `list`: isinstance only, there is no element type to recurse into.
        if hint.is(&list_ty) {
            return introspect::isinstance_of_class(value, &list_ty);
        }

        let origin = introspect::origin(hint)?.ok_or_else(|| {
            PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
                "containers_sequence checker claimed hint {} but it has no __origin__",
                introspect::describe(hint)
            ))
        })?;
        let args = introspect::args(py, hint)?;

        let tuple_ty = self.tuple_type(py);
        if origin.is(&tuple_ty) {
            // Variable-arity `Tuple[X, ...]` / `tuple[X, ...]`: isinstance(tuple, ...)
            // then every element against the single element hint `X`.
            if !introspect::isinstance_of_class(value, &tuple_ty)? {
                return Ok(false);
            }
            let elem_hint = &args[0];
            for item in value.iter()? {
                let item = item?;
                if !registry.check(py, elem_hint, &item)? {
                    return Ok(false);
                }
            }
            return Ok(true);
        }

        // `list[X]`/`List[X]` or `Sequence[X]`/`collections.abc.Sequence[X]`: pick the
        // matching runtime class to isinstance against.
        let runtime_class = if origin.is(&list_ty) {
            list_ty
        } else {
            self.sequence_abc(py)?
        };

        if !introspect::isinstance_of_class(value, &runtime_class)? {
            return Ok(false);
        }

        // A bare, unsubscripted `Sequence`/`list[int]`-less hint has no element type to
        // check elementwise.
        if args.is_empty() {
            return Ok(true);
        }

        let elem_hint = &args[0];
        for item in value.iter()? {
            let item = item?;
            if !registry.check(py, elem_hint, &item)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "containers_sequence"
    }
}

#[cfg(test)]
mod tests {
    // Exercised end-to-end through the public Python API in
    // tests/test_containers_sequence.py once this checker is wired into
    // `checkers::register_all` by the integration pass; no standalone Rust-only unit
    // tests here since every meaningful assertion needs a live `Python` GIL token and a
    // registry containing at least `plain_class` to recurse into.
}
