"""`numbers.Number`/`Complex`/`Real`/`Rational`/`Integral` used directly as hints.

These are plain `abc.ABCMeta` classes that `bool`/`int`/`float`/`complex`/
`fractions.Fraction` register themselves against as virtual subclasses, which plain
`isinstance` already honours - confirmed in the REPL before writing the checker:

    >>> isinstance(3, numbers.Integral)
    True
    >>> isinstance(3.0, numbers.Real)
    True
    >>> isinstance(3.0, numbers.Integral)
    False
"""

import numbers
from fractions import Fraction

from beartype_rs import BeartypeCallHintViolation, beartype, is_bearable


def test_int_passes_integral() -> None:
    assert is_bearable(3, numbers.Integral) is True


def test_float_fails_integral() -> None:
    assert is_bearable(3.0, numbers.Integral) is False


def test_float_passes_real() -> None:
    assert is_bearable(3.0, numbers.Real) is True


def test_bool_passes_integral_since_bool_is_an_int_subclass() -> None:
    assert is_bearable(True, numbers.Integral) is True


def test_complex_passes_complex_but_not_real() -> None:
    assert is_bearable(3 + 4j, numbers.Complex) is True
    assert is_bearable(3 + 4j, numbers.Real) is False


def test_fraction_passes_rational_but_not_integral() -> None:
    assert is_bearable(Fraction(1, 2), numbers.Rational) is True
    assert is_bearable(Fraction(1, 2), numbers.Integral) is False


def test_everything_numeric_passes_number() -> None:
    assert is_bearable(3, numbers.Number) is True
    assert is_bearable(3.0, numbers.Number) is True
    assert is_bearable(3 + 4j, numbers.Number) is True
    assert is_bearable(Fraction(1, 2), numbers.Number) is True


def test_string_fails_number() -> None:
    assert is_bearable("3", numbers.Number) is False


def test_beartype_decorator_passes_on_matching_numeric_tower_hint() -> None:
    @beartype
    def double(x: numbers.Real) -> numbers.Real:
        return x * 2  # type: ignore[operator]

    assert double(3.5) == 7.0
    assert double(3) == 6


def test_beartype_decorator_raises_on_violation() -> None:
    @beartype
    def increment(x: numbers.Integral) -> numbers.Integral:
        return x + 1  # type: ignore[operator]

    assert increment(3) == 4
    try:
        increment(3.0)
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")
