"""Plain classes (int, str, a user class, ...): the isinstance fallback checker."""

from beartype_rs import BeartypeCallHintViolation, beartype, is_bearable


def test_matching_builtin_passes() -> None:
    assert is_bearable(3, int) is True


def test_mismatched_builtin_fails() -> None:
    assert is_bearable("x", int) is False


def test_user_class_is_checked_by_isinstance() -> None:
    class Widget:
        pass

    assert is_bearable(Widget(), Widget) is True
    assert is_bearable(object(), Widget) is False


def test_beartype_decorator_raises_on_violation() -> None:
    @beartype
    def add(a: int, b: int) -> int:
        return a + b

    assert add(1, 2) == 3
    try:
        add(1, "two")
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")
