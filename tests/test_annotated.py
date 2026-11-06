"""`typing.Annotated[X, metadata, ...]` (PEP 593): the wrapped type `X` is checked,
metadata is ignored (no runtime-validator support here)."""

import typing

from beartype_rs import BeartypeCallHintViolation, beartype, is_bearable


def test_matching_wrapped_type_passes() -> None:
    assert is_bearable(3, typing.Annotated[int, "positive"]) is True


def test_mismatched_wrapped_type_fails() -> None:
    assert is_bearable("x", typing.Annotated[int, "positive"]) is False


def test_metadata_is_ignored_even_when_it_would_fail_as_a_validator() -> None:
    # No validator support: metadata is just inert payload here, so a value that would
    # fail a real `Is[...]` validator still passes as long as the wrapped type matches.
    assert is_bearable(-5, typing.Annotated[int, "must be positive, but unchecked"]) is True


def test_nested_container_hint_still_recurses_through_the_registry() -> None:
    hint = typing.Annotated[typing.List[int], "a list of ints"]
    assert is_bearable([1, 2, 3], hint) is True
    assert is_bearable([1, "two", 3], hint) is False


def test_multiple_metadata_entries_are_all_ignored() -> None:
    hint = typing.Annotated[str, "meta-one", 42, ("meta", "three")]
    assert is_bearable("hello", hint) is True
    assert is_bearable(123, hint) is False


def test_beartype_decorator_raises_on_violation() -> None:
    @beartype
    def double(n: typing.Annotated[int, "must be an int"]) -> int:
        return n * 2

    assert double(21) == 42
    try:
        double("not an int")
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")
