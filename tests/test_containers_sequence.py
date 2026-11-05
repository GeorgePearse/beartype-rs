"""Homogeneous sequences: bare list, List[X]/list[X], Sequence[X]/abc.Sequence[X], and
the variable-arity tuple form Tuple[X, ...]/tuple[X, ...]."""

import collections.abc
import typing

from beartype_rs import BeartypeCallHintViolation, beartype, is_bearable


def test_bare_list_passes_for_any_list() -> None:
    assert is_bearable([1, "two", 3.0], list) is True


def test_bare_list_fails_for_non_list() -> None:
    assert is_bearable((1, 2, 3), list) is False


def test_typing_list_passes_when_elements_match() -> None:
    assert is_bearable([1, 2, 3], typing.List[int]) is True


def test_typing_list_fails_when_an_element_mismatches() -> None:
    assert is_bearable([1, 2, "three"], typing.List[int]) is False


def test_builtin_generic_list_passes_when_elements_match() -> None:
    assert is_bearable(["a", "b"], list[str]) is True


def test_builtin_generic_list_fails_for_non_list_value() -> None:
    assert is_bearable(("a", "b"), list[str]) is False


def test_typing_sequence_passes_for_list_and_tuple() -> None:
    assert is_bearable([1, 2, 3], typing.Sequence[int]) is True
    assert is_bearable((1, 2, 3), typing.Sequence[int]) is True


def test_typing_sequence_fails_when_an_element_mismatches() -> None:
    assert is_bearable([1, 2, "three"], typing.Sequence[int]) is False


def test_collections_abc_sequence_subscripted_passes_and_fails() -> None:
    assert is_bearable([1, 2, 3], collections.abc.Sequence[int]) is True
    assert is_bearable([1, 2, "three"], collections.abc.Sequence[int]) is False


def test_variadic_tuple_passes_when_all_elements_match() -> None:
    assert is_bearable((1, 2, 3), typing.Tuple[int, ...]) is True
    assert is_bearable((1, 2, 3), tuple[int, ...]) is True


def test_variadic_tuple_fails_when_an_element_mismatches() -> None:
    assert is_bearable((1, 2, "three"), typing.Tuple[int, ...]) is False


def test_variadic_tuple_fails_for_non_tuple_value() -> None:
    assert is_bearable([1, 2, 3], typing.Tuple[int, ...]) is False


def test_empty_tuple_passes_variadic_tuple_hint() -> None:
    assert is_bearable((), typing.Tuple[int, ...]) is True


def test_beartype_decorator_passes_and_raises_for_list_int() -> None:
    @beartype
    def total(values: typing.List[int]) -> int:
        return sum(values)

    assert total([1, 2, 3]) == 6
    try:
        total([1, 2, "three"])
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")


def test_beartype_decorator_passes_and_raises_for_variadic_tuple() -> None:
    @beartype
    def total(values: typing.Tuple[int, ...]) -> int:
        return sum(values)

    assert total((1, 2, 3)) == 6
    try:
        total([1, 2, 3])
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")
