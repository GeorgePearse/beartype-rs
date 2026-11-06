"""enum.Enum subclasses used directly as a hint (e.g. ``def f(x: Color)``).

There is no dedicated `enum_check` Rust checker: `plain_class`'s `isinstance` fallback
already handles this correctly. Confirmed in the REPL before writing this file:

    >>> import enum
    >>> class Color(enum.Enum):
    ...     RED = 1
    ...     BLUE = 2
    >>> isinstance(Color, type)          # True - so introspect::is_plain_class claims it
    True
    >>> isinstance(Color.RED, Color)     # True - a real member passes
    True
    >>> isinstance(1, Color)             # False - the raw value does NOT pass
    False
    >>> isinstance("RED", Color)         # False - the member name does NOT pass
    False

    >>> class OtherColor(enum.Enum):
    ...     RED = 1
    >>> isinstance(OtherColor.RED, Color)  # False - a same-named member of a
    False                                   # *different* Enum class does NOT pass

    >>> class IntColor(enum.IntEnum):
    ...     RED = 1
    >>> isinstance(1, IntColor)          # False - even though IntEnum members are ints,
    False                                 # the raw int itself is not a member instance

This file exercises that fallback through the public API, per the category spec.
"""

import enum

from beartype_rs import BeartypeCallHintViolation, beartype, is_bearable


class Color(enum.Enum):
    RED = 1
    BLUE = 2


class OtherColor(enum.Enum):
    RED = 1


class IntColor(enum.IntEnum):
    RED = 1
    BLUE = 2


def test_enum_member_passes() -> None:
    assert is_bearable(Color.RED, Color) is True
    assert is_bearable(Color.BLUE, Color) is True


def test_raw_value_fails() -> None:
    assert is_bearable(1, Color) is False


def test_member_name_string_fails() -> None:
    assert is_bearable("RED", Color) is False


def test_member_of_different_enum_class_fails() -> None:
    assert is_bearable(OtherColor.RED, Color) is False


def test_int_enum_member_passes_but_plain_int_fails() -> None:
    assert is_bearable(IntColor.RED, IntColor) is True
    assert is_bearable(1, IntColor) is False


def test_non_member_object_fails() -> None:
    assert is_bearable(object(), Color) is False


def test_beartype_decorator_accepts_matching_enum_member() -> None:
    @beartype
    def describe(c: Color) -> str:
        return c.name

    assert describe(Color.RED) == "RED"


def test_beartype_decorator_raises_on_violation() -> None:
    @beartype
    def describe(c: Color) -> str:
        return c.name

    try:
        describe(1)  # type: ignore[arg-type]
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")
