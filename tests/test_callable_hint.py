"""`typing.Callable` / `collections.abc.Callable` hints.

Only confirms `callable(value)` - beartype_rs does not verify an arbitrary callable's
own parameter/return types against the hint's sub-hints at runtime (see the comment in
`src/checkers/callable_hint.rs`), so a function with a mismatched signature but that is
still callable is accepted; only non-callables are rejected.
"""

import collections.abc
import typing

from beartype_rs import BeartypeCallHintViolation, beartype, is_bearable


def test_bare_typing_callable_accepts_a_function() -> None:
    def fn(x: int) -> int:
        return x

    assert is_bearable(fn, typing.Callable) is True


def test_bare_typing_callable_rejects_non_callable() -> None:
    assert is_bearable(42, typing.Callable) is False


def test_subscripted_typing_callable_accepts_matching_function() -> None:
    def combine(a: int, b: str) -> bool:
        return bool(a) and bool(b)

    assert is_bearable(combine, typing.Callable[[int, str], bool]) is True


def test_subscripted_typing_callable_rejects_non_callable() -> None:
    assert is_bearable("not callable", typing.Callable[[int, str], bool]) is False


def test_subscripted_abc_callable_accepts_lambda() -> None:
    assert is_bearable(lambda x: x, collections.abc.Callable[[int], int]) is True


def test_subscripted_abc_callable_rejects_non_callable() -> None:
    assert is_bearable(None, collections.abc.Callable[[int], int]) is False


def test_callable_class_instance_is_accepted() -> None:
    class Adder:
        def __call__(self, a: int, b: int) -> int:
            return a + b

    assert is_bearable(Adder(), typing.Callable[[int, int], int]) is True


def test_beartype_decorator_accepts_callable_argument() -> None:
    @beartype
    def apply(fn: typing.Callable[[int], int], value: int) -> int:
        return fn(value)

    assert apply(lambda x: x + 1, 5) == 6


def test_beartype_decorator_raises_on_non_callable_argument() -> None:
    @beartype
    def apply(fn: typing.Callable[[int], int], value: int) -> int:
        return fn(value)

    try:
        apply("not a function", 5)
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")
