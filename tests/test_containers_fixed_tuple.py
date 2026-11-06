"""Fixed-arity tuples (`Tuple[int, str]` / `tuple[int, str]`), including the
empty-tuple hint `Tuple[()]` / `tuple[()]`: a different hint per position, checked
positionally - not to be confused with the variadic `Tuple[int, ...]` form."""

import typing

from beartype_rs import BeartypeCallHintViolation, beartype, is_bearable


def test_matching_fixed_tuple_passes() -> None:
    assert is_bearable((1, "two", 3.0), typing.Tuple[int, str, float]) is True


def test_matching_fixed_tuple_passes_builtin_generic() -> None:
    assert is_bearable((1, "two"), tuple[int, str]) is True


def test_wrong_type_at_position_fails() -> None:
    assert is_bearable((1, 2), typing.Tuple[int, str]) is False


def test_wrong_length_fails() -> None:
    assert is_bearable((1, "two", 3.0), typing.Tuple[int, str]) is False
    assert is_bearable((1,), typing.Tuple[int, str]) is False


def test_non_tuple_value_fails() -> None:
    assert is_bearable([1, "two"], typing.Tuple[int, str]) is False


def test_empty_tuple_hint_matches_only_empty_tuple() -> None:
    assert is_bearable((), typing.Tuple[()]) is True
    assert is_bearable((), tuple[()]) is True
    assert is_bearable((1,), typing.Tuple[()]) is False


def test_nested_fixed_tuple_recurses_through_registry() -> None:
    hint = typing.Tuple[typing.Tuple[int, int], str]
    assert is_bearable(((1, 2), "ok"), hint) is True
    assert is_bearable(((1, "bad"), "ok"), hint) is False


def test_beartype_decorator_raises_on_violation() -> None:
    @beartype
    def first_point(point: typing.Tuple[float, float]) -> float:
        x, _y = point
        return x

    assert first_point((1.0, 2.0)) == 1.0
    try:
        first_point((1.0, "two"))
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")
