"""`typing.Protocol` subclasses decorated `@typing.runtime_checkable`: structural checking
via Python's own `isinstance`, not nominal (the plain_class checker's) subclassing.
"""

import typing

from beartype_rs import BeartypeCallHintViolation, beartype, is_bearable


@typing.runtime_checkable
class SupportsQuack(typing.Protocol):
    def quack(self) -> str: ...


class Duck:
    def quack(self) -> str:
        return "quack"


class Rock:
    pass


def test_structurally_matching_instance_passes() -> None:
    assert is_bearable(Duck(), SupportsQuack) is True


def test_structurally_mismatched_instance_fails() -> None:
    assert is_bearable(Rock(), SupportsQuack) is False


def test_structural_match_does_not_require_nominal_subclassing() -> None:
    # Duck's class statement never names SupportsQuack as a base - this must pass on
    # structure (the `quack` method) alone, proving the protocol checker (not the nominal
    # plain_class isinstance fallback, which would require `class Duck(SupportsQuack)`)
    # is in play. Note `issubclass(Duck, SupportsQuack)` is *also* True here - a
    # `@runtime_checkable` Protocol's issubclass is itself structural (method presence,
    # not nominal inheritance) - so it can't be used to prove the absence of a nominal
    # relationship; checking `SupportsQuack not in Duck.__mro__` can.
    assert SupportsQuack not in Duck.__mro__
    assert is_bearable(Duck(), SupportsQuack) is True


def test_beartype_decorator_passes_structurally_compatible_value() -> None:
    @beartype
    def make_it_quack(thing: SupportsQuack) -> str:
        return thing.quack()

    assert make_it_quack(Duck()) == "quack"


def test_beartype_decorator_raises_on_structural_violation() -> None:
    @beartype
    def make_it_quack(thing: SupportsQuack) -> str:
        return thing.quack()

    try:
        make_it_quack(Rock())
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")
