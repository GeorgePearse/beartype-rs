"""`typing.Literal[...]`: value must equal one of a closed set of literal values."""

import enum
from typing import Literal

from beartype_rs import BeartypeCallHintViolation, beartype, is_bearable


def test_matching_literal_value_passes() -> None:
    assert is_bearable(2, Literal[1, 2, "a"]) is True
    assert is_bearable("a", Literal[1, 2, "a"]) is True


def test_mismatched_literal_value_fails() -> None:
    assert is_bearable(3, Literal[1, 2, "a"]) is False
    assert is_bearable("b", Literal[1, 2, "a"]) is False


def test_bool_is_not_confused_with_int() -> None:
    # True == 1 in Python, but Literal[1] must not accept True, and Literal[True]
    # must not accept 1 - they are different literal values even though they compare
    # equal.
    assert is_bearable(True, Literal[1]) is False
    assert is_bearable(1, Literal[True]) is False
    assert is_bearable(True, Literal[True]) is True
    assert is_bearable(1, Literal[1]) is True


def test_none_literal() -> None:
    assert is_bearable(None, Literal[None]) is True
    assert is_bearable(0, Literal[None]) is False


def test_enum_member_literal() -> None:
    class Color(enum.Enum):
        RED = 1
        BLUE = 2

    assert is_bearable(Color.RED, Literal[Color.RED, Color.BLUE]) is True
    assert is_bearable(Color.BLUE, Literal[Color.RED]) is False
    # An int that happens to equal the enum member's value must not match either.
    assert is_bearable(1, Literal[Color.RED]) is False


def test_beartype_decorator_raises_on_violation() -> None:
    @beartype
    def set_mode(mode: Literal["fast", "slow"]) -> str:
        return mode

    assert set_mode("fast") == "fast"
    try:
        set_mode("turbo")
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")
