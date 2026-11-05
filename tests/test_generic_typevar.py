"""A bare `typing.TypeVar` used directly as a hint: unbound TypeVars accept anything,
a `bound=` TypeVar defers to the bound, and a constrained TypeVar (positional
alternatives) passes if any constraint matches.
"""

import typing

from beartype_rs import BeartypeCallHintViolation, beartype, is_bearable

T = typing.TypeVar("T")
Bounded = typing.TypeVar("Bounded", bound=int)
Constrained = typing.TypeVar("Constrained", int, str)


def test_unconstrained_typevar_accepts_any_value() -> None:
    assert is_bearable(3, T) is True
    assert is_bearable("hello", T) is True
    assert is_bearable(None, T) is True


def test_bound_typevar_passes_when_value_satisfies_bound() -> None:
    assert is_bearable(3, Bounded) is True


def test_bound_typevar_fails_when_value_violates_bound() -> None:
    assert is_bearable("not an int", Bounded) is False


def test_constrained_typevar_passes_if_any_constraint_matches() -> None:
    assert is_bearable(3, Constrained) is True
    assert is_bearable("x", Constrained) is True


def test_constrained_typevar_fails_when_no_constraint_matches() -> None:
    assert is_bearable(3.5, Constrained) is False


def test_beartype_decorator_passes_with_matching_bound() -> None:
    @beartype
    def first(x: Bounded) -> Bounded:
        return x

    assert first(5) == 5


def test_beartype_decorator_raises_on_bound_violation() -> None:
    @beartype
    def first(x: Bounded) -> Bounded:
        return x

    try:
        first("nope")
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")
