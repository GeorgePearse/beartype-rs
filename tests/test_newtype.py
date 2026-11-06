"""typing.NewType: checked exactly like its __supertype__ at runtime."""

import typing

from beartype_rs import BeartypeCallHintViolation, beartype, is_bearable

UserId = typing.NewType("UserId", int)


def test_value_matching_supertype_passes() -> None:
    assert is_bearable(3, UserId) is True


def test_value_mismatching_supertype_fails() -> None:
    assert is_bearable("3", UserId) is False


def test_beartype_decorator_accepts_supertype_value() -> None:
    @beartype
    def greet(user_id: UserId) -> str:
        return f"hello {user_id}"

    assert greet(3) == "hello 3"


def test_beartype_decorator_raises_on_violation() -> None:
    @beartype
    def greet(user_id: UserId) -> str:
        return f"hello {user_id}"

    try:
        greet("not an int")
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")
