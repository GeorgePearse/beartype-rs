"""Mapping containers: dict, typing.Dict/Mapping/MutableMapping and their abc equivalents."""

import collections.abc
import typing

from beartype_rs import BeartypeCallHintViolation, beartype, is_bearable


def test_bare_dict_passes_for_any_dict() -> None:
    assert is_bearable({"a": 1}, dict) is True


def test_bare_dict_fails_for_non_dict() -> None:
    assert is_bearable([1, 2], dict) is False


def test_typing_dict_parameterized_passes() -> None:
    assert is_bearable({1: "a", 2: "b"}, typing.Dict[int, str]) is True


def test_builtin_generic_dict_parameterized_passes() -> None:
    assert is_bearable({1: "a"}, dict[int, str]) is True


def test_typing_dict_parameterized_fails_on_wrong_value_type() -> None:
    assert is_bearable({1: 2}, typing.Dict[int, str]) is False


def test_typing_dict_parameterized_fails_on_wrong_key_type() -> None:
    assert is_bearable({"x": "a"}, typing.Dict[int, str]) is False


def test_typing_mapping_parameterized_passes_for_plain_dict() -> None:
    assert is_bearable({1: "a"}, typing.Mapping[int, str]) is True


def test_collections_abc_mapping_bare_passes_for_dict() -> None:
    assert is_bearable({1: "a"}, collections.abc.Mapping) is True


def test_collections_abc_mapping_bare_fails_for_non_mapping() -> None:
    assert is_bearable([1, 2], collections.abc.Mapping) is False


def test_typing_mutable_mapping_parameterized_passes() -> None:
    assert is_bearable({"a": 1, "b": 2}, typing.MutableMapping[str, int]) is True


def test_typing_mutable_mapping_parameterized_fails_on_wrong_value_type() -> None:
    assert is_bearable({"a": "not-an-int"}, typing.MutableMapping[str, int]) is False


def test_collections_abc_mutable_mapping_bare_passes() -> None:
    assert is_bearable({"a": 1}, collections.abc.MutableMapping) is True


def test_beartype_decorator_passes_on_valid_mapping() -> None:
    @beartype
    def total_scores(scores: typing.Dict[str, int]) -> int:
        return sum(scores.values())

    assert total_scores({"alice": 3, "bob": 5}) == 8


def test_beartype_decorator_raises_on_mapping_violation() -> None:
    @beartype
    def total_scores(scores: typing.Dict[str, int]) -> int:
        return sum(scores.values())

    try:
        total_scores({"alice": "not-a-score"})
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")
