"""`typing.ParamSpec` (PEP 612) and `typing.TypeVarTuple`/`typing.Unpack` (PEP 646) used
directly as a bare hint.

These are purely static, signature-shape constructs with no single runtime value they
could meaningfully be checked against, so `paramspec_typevartuple`'s `check()` is a
documented permissive no-op: it claims the hint and always accepts. There is therefore no
"failing case" *of this category's own check logic* to exercise - by design, nothing a
bare `ParamSpec`/`TypeVarTuple`/`Unpack` hint sees can fail. What we exercise instead,
mirroring `test_plain_class.py`'s `@beartype` coverage, is that this permissiveness holds
for arbitrary values (the "passing" cases) and that it does not leak into permissiveness
for *other*, ordinary parameters checked by other checkers in the same call - a
`@beartype`-decorated function mixing a `TypeVarTuple`/`Unpack` varargs parameter with a
plain `int` parameter still raises on a bad `int` (the "failing" case).
"""

import typing

from beartype_rs import BeartypeCallHintViolation, beartype, is_bearable


def test_bare_paramspec_accepts_any_value() -> None:
    P = typing.ParamSpec("P")  # noqa: N806
    assert is_bearable(3, P) is True
    assert is_bearable("not a callable shape at all", P) is True
    assert is_bearable(None, P) is True


def test_bare_typevartuple_accepts_any_value() -> None:
    Ts = typing.TypeVarTuple("Ts")  # noqa: N806
    assert is_bearable((1, "two", 3.0), Ts) is True
    assert is_bearable(42, Ts) is True


def test_unpack_of_typevartuple_accepts_any_value() -> None:
    Ts = typing.TypeVarTuple("Ts")  # noqa: N806
    unpacked = typing.Unpack[Ts]
    assert is_bearable((1, "two", 3.0), unpacked) is True
    assert is_bearable(object(), unpacked) is True


def test_beartype_decorator_accepts_variadic_args_of_any_shape() -> None:
    Ts = typing.TypeVarTuple("Ts")  # noqa: N806

    @beartype
    def first_of(*args: *Ts) -> None:
        return None

    # `*args: *Ts` resolves (via `typing.get_type_hints`) to a bare `Unpack[Ts]` hint for
    # the `args` parameter - no runtime shape of the collected tuple can violate it.
    first_of(1, "two", 3.0)
    first_of()
    first_of(object(), object())


def test_beartype_decorator_still_enforces_other_parameters() -> None:
    """The permissiveness of ParamSpec/TypeVarTuple/Unpack must not leak into other,
    ordinary parameters checked by other checkers in the same call."""
    Ts = typing.TypeVarTuple("Ts")  # noqa: N806

    @beartype
    def labelled(label: int, *args: *Ts) -> int:
        return label

    assert labelled(1, "two", 3.0) == 1
    try:
        labelled("not an int", "two", 3.0)
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation for the bad `label`")
