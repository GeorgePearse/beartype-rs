//! `numbers.Number`, `numbers.Complex`, `numbers.Real`, `numbers.Rational`,
//! `numbers.Integral` used directly as hints - the standard-library "numeric tower" of
//! abstract base classes.
//!
//! These are ordinary ABCs (`type(numbers.Number) is abc.ABCMeta`), and the builtin
//! numeric types register themselves as virtual subclasses via `ABCMeta.register`
//! (`bool`/`int` -> `Integral`, `float` -> `Real`, `complex` -> `Complex`,
//! `fractions.Fraction` -> `Rational`), which `decimal.Decimal` notably does *not* do -
//! verified in the REPL:
//!
//! ```text
//! >>> isinstance(3, numbers.Integral)          # True
//! >>> isinstance(3.0, numbers.Integral)        # False
//! >>> isinstance(3.0, numbers.Real)            # True
//! >>> isinstance(True, numbers.Integral)       # True (bool is an int subclass)
//! >>> isinstance(3+4j, numbers.Real)           # False (Complex, not Real)
//! >>> isinstance(Fraction(1, 2), numbers.Rational)  # True
//! >>> isinstance(Fraction(1, 2), numbers.Integral)  # False
//! ```
//!
//! Because these ABCs carry no `__origin__`/`__args__` (a bare hint like `numbers.Number`
//! *is* the class - confirmed `hasattr(numbers.Number, "__origin__")` is `False`) and
//! `type(numbers.Number)` is `ABCMeta`, a subclass of `type`, `introspect::is_plain_class`
//! (which checks `isinstance(hint, type)`) would actually already accept these hints and
//! `plain_class`'s plain `isinstance(value, hint)` would already produce the right
//! answer. This checker exists anyway, ahead of `plain_class` in the registry, purely so
//! the numeric-tower ABCs are an explicit, tested, documented category rather than an
//! accident of `plain_class`'s generality that a future refactor could silently break.

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyModule};

use crate::introspect;
use crate::registry::{Checker, Registry};

pub struct NumericTowerChecker;

impl NumericTowerChecker {
    /// Is `hint` *exactly* (by identity) one of
    /// `numbers.{Number,Complex,Real,Rational,Integral}`?
    fn numeric_tower_marker(&self, py: Python<'_>, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        let numbers = PyModule::import_bound(py, "numbers")?;
        for name in ["Number", "Complex", "Real", "Rational", "Integral"] {
            let marker = numbers.getattr(name)?;
            if hint.is(&marker) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl Checker for NumericTowerChecker {
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        let py = hint.py();
        self.numeric_tower_marker(py, hint)
    }

    fn check(
        &self,
        py: Python<'_>,
        hint: &Bound<'_, PyAny>,
        value: &Bound<'_, PyAny>,
        _registry: &Registry,
    ) -> PyResult<bool> {
        // No type parameters to recurse into - numbers.Number and friends take none, so
        // this is isinstance-only, same as the plain_class fallback it preempts.
        let _ = py;
        introspect::isinstance_of_class(value, hint)
    }

    fn name(&self) -> &'static str {
        "numeric_tower"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::PyInt;

    #[test]
    fn matches_the_five_numeric_tower_abcs_and_nothing_else() {
        Python::with_gil(|py| {
            let numbers = PyModule::import_bound(py, "numbers").unwrap();
            let checker = NumericTowerChecker;

            for name in ["Number", "Complex", "Real", "Rational", "Integral"] {
                let marker = numbers.getattr(name).unwrap();
                assert!(checker.matches(&marker).unwrap(), "expected to match numbers.{name}");
            }

            let plain_int = py.get_type_bound::<PyInt>();
            assert!(!checker.matches(&plain_int).unwrap());
        });
    }

    #[test]
    fn check_honours_the_isinstance_virtual_subclass_registrations() {
        Python::with_gil(|py| {
            let numbers = PyModule::import_bound(py, "numbers").unwrap();
            let registry = Registry::new(vec![Box::new(NumericTowerChecker)]);
            let checker = NumericTowerChecker;

            let integral = numbers.getattr("Integral").unwrap();
            let real = numbers.getattr("Real").unwrap();
            let complex_abc = numbers.getattr("Complex").unwrap();

            // Values are built via `py.eval_bound` rather than per-type constructors, so
            // this test does not depend on which concrete `Py*` constructor APIs this
            // PyO3 version happens to expose (e.g. `PyLong`/`PyInt` has none).
            let three = py.eval_bound("3", None, None).unwrap();
            assert!(checker.check(py, &integral, &three, &registry).unwrap());

            let three_point_oh = py.eval_bound("3.0", None, None).unwrap();
            assert!(!checker.check(py, &integral, &three_point_oh, &registry).unwrap());
            assert!(checker.check(py, &real, &three_point_oh, &registry).unwrap());

            let true_value = py.eval_bound("True", None, None).unwrap();
            assert!(checker.check(py, &integral, &true_value, &registry).unwrap());

            let complex_value = py.eval_bound("3 + 4j", None, None).unwrap();
            assert!(checker.check(py, &complex_abc, &complex_value, &registry).unwrap());
            assert!(!checker.check(py, &real, &complex_value, &registry).unwrap());

            let not_a_number = py.eval_bound("'x'", None, None).unwrap();
            assert!(!checker.check(py, &integral, &not_a_number, &registry).unwrap());
        });
    }
}
