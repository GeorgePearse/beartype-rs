"""Bare-string forward references (`"int"`) and `typing.ForwardRef` instances, resolved
against `builtins` only.

Note: in the normal `@beartype` decorator path these never reach beartype_rs at all -
`typing.get_type_hints()` resolves stringified annotations before the Rust side ever sees
them - so these tests go through `is_bearable` directly, the path where a caller can still
hand beartype_rs an unresolved forward reference.
"""

import typing

from beartype_rs import is_bearable


def test_bare_string_hint_resolves_against_builtins_and_passes() -> None:
    assert is_bearable(3, "int") is True


def test_bare_string_hint_resolves_against_builtins_and_fails() -> None:
    assert is_bearable("not an int", "int") is False


def test_forward_ref_instance_resolves_against_builtins_and_passes() -> None:
    assert is_bearable([1, 2], typing.ForwardRef("list")) is True


def test_forward_ref_instance_resolves_against_builtins_and_fails() -> None:
    assert is_bearable("nope", typing.ForwardRef("list")) is False


def test_unresolvable_forward_ref_name_raises_type_error_not_false() -> None:
    # "MyClass" isn't a builtin, and resolving it against some defining module's globals
    # is explicitly out of scope here - this must raise loudly, not just say `False`.
    try:
        is_bearable(object(), "MyClass")
    except TypeError:
        pass
    else:
        raise AssertionError("expected a TypeError for an unresolvable forward reference")


def test_unresolvable_forward_ref_instance_raises_type_error_not_false() -> None:
    try:
        is_bearable(object(), typing.ForwardRef("MyClass"))
    except TypeError:
        pass
    else:
        raise AssertionError("expected a TypeError for an unresolvable forward reference")
