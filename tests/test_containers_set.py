"""set/frozenset: bare classes and typing.Set[X]/set[X], typing.FrozenSet[X]/frozenset[X]."""

import typing

from beartype_rs import BeartypeCallHintViolation, beartype, is_bearable


def test_bare_set_passes() -> None:
    assert is_bearable({1, 2, 3}, set) is True


def test_bare_set_rejects_non_set() -> None:
    assert is_bearable([1, 2, 3], set) is False


def test_bare_frozenset_passes() -> None:
    assert is_bearable(frozenset({1, 2, 3}), frozenset) is True


def test_set_and_frozenset_are_not_interchangeable() -> None:
    assert is_bearable({1, 2, 3}, frozenset) is False
    assert is_bearable(frozenset({1, 2, 3}), set) is False


def test_builtin_generic_set_passes_with_matching_elements() -> None:
    assert is_bearable({1, 2, 3}, set[int]) is True


def test_builtin_generic_set_fails_with_mismatched_element() -> None:
    assert is_bearable({1, 2, "oops"}, set[int]) is False


def test_builtin_generic_frozenset_passes_with_matching_elements() -> None:
    assert is_bearable(frozenset({"a", "b"}), frozenset[str]) is True


def test_builtin_generic_frozenset_fails_with_mismatched_element() -> None:
    assert is_bearable(frozenset({"a", 2}), frozenset[str]) is False


def test_typing_set_alias_passes() -> None:
    assert is_bearable({1, 2, 3}, typing.Set[int]) is True


def test_typing_frozenset_alias_fails_on_wrong_element_type() -> None:
    assert is_bearable(frozenset({1, 2}), typing.FrozenSet[str]) is False


def test_beartype_decorator_passes_and_raises() -> None:
    @beartype
    def total(values: set[int]) -> int:
        return sum(values)

    assert total({1, 2, 3}) == 6

    try:
        total({1, "two", 3})
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")
