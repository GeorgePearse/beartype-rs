"""`typing.Union`, `typing.Optional`, and PEP 604 `X | Y` unions."""

import typing

from beartype_rs import BeartypeCallHintViolation, beartype, is_bearable


def test_union_passes_for_either_branch() -> None:
    hint = typing.Union[int, str]
    assert is_bearable(3, hint) is True
    assert is_bearable("x", hint) is True


def test_union_fails_for_neither_branch() -> None:
    hint = typing.Union[int, str]
    assert is_bearable(3.5, hint) is False


def test_optional_accepts_none_and_the_wrapped_type() -> None:
    hint = typing.Optional[int]
    assert is_bearable(None, hint) is True
    assert is_bearable(5, hint) is True
    assert is_bearable("x", hint) is False


def test_pep604_union_syntax_passes_for_either_branch() -> None:
    hint = int | str
    assert is_bearable(3, hint) is True
    assert is_bearable("x", hint) is True
    assert is_bearable(3.5, hint) is False


def test_pep604_optional_style_accepts_none() -> None:
    hint = int | None
    assert is_bearable(None, hint) is True
    assert is_bearable(7, hint) is True
    assert is_bearable("x", hint) is False


def test_beartype_decorator_passes_matching_union_argument() -> None:
    @beartype
    def describe(value: typing.Union[int, str]) -> str:
        return str(value)

    assert describe(3) == "3"
    assert describe("x") == "x"


def test_beartype_decorator_raises_on_union_violation() -> None:
    @beartype
    def describe(value: int | str) -> str:
        return str(value)

    try:
        describe(3.5)
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")
