"""`typing.ClassVar[X]` and `typing.Final[X]`, including the bare (unsubscripted) form.

These wrappers only matter to a static type checker, but at runtime they must not crash
and must check the wrapped type exactly as if the wrapper weren't there.
"""

import typing

from beartype_rs import BeartypeCallHintViolation, beartype, is_bearable


def test_classvar_passes_for_matching_wrapped_type() -> None:
    assert is_bearable(3, typing.ClassVar[int]) is True


def test_classvar_fails_for_mismatched_wrapped_type() -> None:
    assert is_bearable("x", typing.ClassVar[int]) is False


def test_final_passes_for_matching_wrapped_type() -> None:
    assert is_bearable("hello", typing.Final[str]) is True


def test_final_fails_for_mismatched_wrapped_type() -> None:
    assert is_bearable(3, typing.Final[str]) is False


def test_bare_final_always_passes() -> None:
    assert is_bearable(3, typing.Final) is True
    assert is_bearable("anything at all", typing.Final) is True
    assert is_bearable(None, typing.Final) is True


def test_bare_classvar_always_passes() -> None:
    assert is_bearable(3, typing.ClassVar) is True
    assert is_bearable(object(), typing.ClassVar) is True


def test_beartype_decorator_passes_matching_final_argument() -> None:
    @beartype
    def describe(value: typing.Final[int]) -> str:
        return str(value)

    assert describe(3) == "3"


def test_beartype_decorator_raises_on_classvar_violation() -> None:
    @beartype
    def describe(value: typing.ClassVar[int]) -> str:
        return str(value)

    try:
        describe("not an int")
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")
