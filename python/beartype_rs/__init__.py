"""Rust-accelerated runtime type checking for Python, in the spirit of beartype.

The actual checking happens in the compiled `_beartype_rs` extension (see `src/`); this
module is the pure-Python surface: the `@beartype` decorator, `is_bearable` /
`die_if_unbearable` (mirroring `beartype.door`'s names), and the exception type.
"""

from __future__ import annotations

import functools
import inspect
import typing
from collections.abc import Callable
from typing import Any, TypeVar

from ._beartype_rs import die_if_unbearable as _die_if_unbearable_rs
from ._beartype_rs import is_bearable as _is_bearable_rs

__all__ = ["BeartypeCallHintViolation", "beartype", "die_if_unbearable", "is_bearable"]

_F = TypeVar("_F", bound=Callable[..., Any])


class BeartypeCallHintViolation(TypeError):
    """Raised when a ``@beartype``-decorated call violates one of its type hints."""


def is_bearable(value: Any, hint: Any) -> bool:
    """Return whether ``value`` satisfies the type hint ``hint``."""
    return _is_bearable_rs(value, hint)


def die_if_unbearable(value: Any, hint: Any, description: str) -> None:
    """Raise :class:`BeartypeCallHintViolation` if ``value`` does not satisfy ``hint``."""
    try:
        _die_if_unbearable_rs(value, hint, description)
    except TypeError as exc:
        raise BeartypeCallHintViolation(str(exc)) from None


def beartype(func: _F) -> _F:
    """Decorate ``func`` so every call is checked against its type hints at the Rust layer."""
    hints = typing.get_type_hints(func)
    signature = inspect.signature(func)
    return_hint = hints.pop("return", None)

    @functools.wraps(func)
    def wrapper(*args: Any, **kwargs: Any) -> Any:
        bound = signature.bind(*args, **kwargs)
        bound.apply_defaults()
        for name, value in bound.arguments.items():
            hint = hints.get(name)
            if hint is None:
                continue
            die_if_unbearable(
                value,
                hint,
                f"{func.__qualname__}() parameter {name}={value!r} violates type hint {hint!r}",
            )
        result = func(*args, **kwargs)
        if return_hint is not None:
            die_if_unbearable(
                result,
                return_hint,
                f"{func.__qualname__}() return value {result!r} violates type hint {return_hint!r}",
            )
        return result

    return wrapper  # type: ignore[return-value]
