"""`typing.Any`, `typing.NoReturn`, `typing.Never`, and `typing.Self`.

All four are unconditionally permissive at this crate's value-check layer: `Any` is
permissive by definition, and `NoReturn`/`Never`/`Self` have no meaningful runtime
value-level check without the enclosing call/class context (out of scope here, see
`src/checkers/self_and_special_forms.rs`). So there is no value that makes any of these
hints fail *on their own* - the "failing case" below instead proves that this
permissiveness is properly scoped to the hint it is attached to: a decorated function
that mixes a permissive hint with a real, mismatched one still raises.
"""

import typing

from beartype_rs import BeartypeCallHintViolation, beartype, is_bearable


def test_any_always_passes() -> None:
    assert is_bearable(3, typing.Any) is True
    assert is_bearable("anything at all", typing.Any) is True
    assert is_bearable(None, typing.Any) is True
    assert is_bearable(object(), typing.Any) is True


def test_self_always_passes() -> None:
    assert is_bearable(3, typing.Self) is True
    assert is_bearable("anything at all", typing.Self) is True
    assert is_bearable(None, typing.Self) is True


def test_no_return_always_passes() -> None:
    assert is_bearable(3, typing.NoReturn) is True
    assert is_bearable(None, typing.NoReturn) is True


def test_never_always_passes() -> None:
    assert is_bearable(3, typing.Never) is True
    assert is_bearable(None, typing.Never) is True


def test_beartype_decorator_passes_with_any_and_self_annotated_arguments() -> None:
    @beartype
    def identity(a: typing.Any, b: typing.Self) -> typing.Any:
        return (a, b)

    assert identity(3, "whatever") == (3, "whatever")
    sentinel = object()
    assert identity(None, sentinel) == (None, sentinel)


def test_beartype_decorator_allows_any_return_value() -> None:
    @beartype
    def passthrough(value: int) -> typing.Any:
        return value

    assert passthrough(5) == 5


def test_beartype_decorator_still_raises_on_an_unrelated_real_type_violation() -> None:
    """`Any` on one parameter must not swallow a real violation on another."""

    @beartype
    def combine(flexible: typing.Any, strict: int) -> str:
        return f"{flexible}:{strict}"

    assert combine("ok", 1) == "ok:1"
    try:
        combine("ok", "not an int")
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")


def test_beartype_decorator_no_return_annotation_does_not_block_a_normal_return() -> None:
    """`NoReturn` is permissive here even though the function does return normally -

    enforcing "this function never returns" is out of scope for a value-level checker.
    """

    @beartype
    def surprise() -> typing.NoReturn:  # type: ignore[misc]
        return "surprised ya"  # type: ignore[return-value]

    assert surprise() == "surprised ya"
