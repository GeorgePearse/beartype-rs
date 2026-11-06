"""collections.abc.{Iterable,Iterator,Sized,Hashable,Container,Collection}, bare and
subscripted (via both `collections.abc` and the `typing` aliases, which share the exact
same `__origin__`).

Note the deliberate isinstance-only contract: for the parameterized forms
(`Iterable[X]`, `Container[X]`, `Collection[X]`) the element type `X` is never checked -
only `isinstance(value, <abc class>)`. This is load-bearing, not an oversight: an
`Iterator[X]` can be infinite or one-shot, so eagerly walking it to check elements would
risk hanging forever or consuming it out from under the caller.
"""

import collections.abc
import typing

from beartype_rs import BeartypeCallHintViolation, beartype, is_bearable


# -- Iterable -----------------------------------------------------------------


def test_bare_abc_iterable_passes() -> None:
    assert is_bearable([1, 2, 3], collections.abc.Iterable) is True


def test_bare_abc_iterable_rejects_non_iterable() -> None:
    assert is_bearable(3, collections.abc.Iterable) is False


def test_typing_iterable_alias_passes_with_matching_elements() -> None:
    assert is_bearable([1, 2, 3], typing.Iterable[int]) is True


def test_typing_iterable_alias_is_isinstance_only_and_skips_element_check() -> None:
    # Documented contract: Iterable[X]'s element type is never inspected, so a list
    # with a mismatched element type still passes - only the bare isinstance matters.
    assert is_bearable([1, "two", 3], typing.Iterable[int]) is True
    # A non-iterable still fails regardless of the element argument.
    assert is_bearable(3, typing.Iterable[int]) is False


# -- Iterator -------------------------------------------------------------------


def test_abc_iterator_passes_for_an_actual_iterator() -> None:
    assert is_bearable(iter([1, 2, 3]), typing.Iterator[int]) is True


def test_abc_iterator_rejects_a_plain_list() -> None:
    # A list is Iterable but not itself an Iterator (no __next__).
    assert is_bearable([1, 2, 3], typing.Iterator[int]) is False


# -- Sized ------------------------------------------------------------------------


def test_bare_sized_passes_for_something_with_len() -> None:
    assert is_bearable([1, 2, 3], collections.abc.Sized) is True
    assert is_bearable("hello", typing.Sized) is True


def test_bare_sized_rejects_something_without_len() -> None:
    assert is_bearable(3, collections.abc.Sized) is False


# -- Hashable ----------------------------------------------------------------------


def test_hashable_passes_for_a_hashable_value() -> None:
    assert is_bearable(3, collections.abc.Hashable) is True
    assert is_bearable("x", typing.Hashable) is True


def test_hashable_rejects_an_unhashable_value() -> None:
    assert is_bearable([1, 2, 3], collections.abc.Hashable) is False
    assert is_bearable({"a": 1}, typing.Hashable) is False


# -- Container ----------------------------------------------------------------------


def test_container_alias_passes_regardless_of_element_type() -> None:
    assert is_bearable("abc", typing.Container[str]) is True
    # Isinstance-only: wrong element type is not checked.
    assert is_bearable([1, 2, 3], typing.Container[str]) is True


def test_container_alias_rejects_non_container() -> None:
    assert is_bearable(3, typing.Container[str]) is False


# -- Collection ---------------------------------------------------------------------


def test_collection_alias_passes_for_a_list() -> None:
    assert is_bearable([1, 2, 3], typing.Collection[int]) is True


def test_collection_alias_rejects_a_bare_iterator() -> None:
    # An iterator has no __len__/__contains__, so it is not a Collection even though
    # it is an Iterable/Iterator.
    assert is_bearable(iter([1, 2, 3]), typing.Collection[int]) is False


# -- Through the public decorator ----------------------------------------------------


def test_beartype_decorator_passes_and_raises() -> None:
    @beartype
    def describe(values: collections.abc.Sized) -> int:
        return len(values)

    assert describe([1, 2, 3]) == 3

    try:
        describe(3)
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")


def test_beartype_decorator_with_typing_iterable_alias() -> None:
    @beartype
    def first(values: typing.Iterable[int]) -> int:
        return next(iter(values))

    assert first([10, 20]) == 10

    try:
        first(5)
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")
