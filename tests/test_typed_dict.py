"""typing.TypedDict subclasses: structural dict checking against declared keys."""

import typing

from beartype_rs import BeartypeCallHintViolation, beartype, is_bearable


class Movie(typing.TypedDict):
    title: str
    year: int


class Rating(typing.TypedDict, total=False):
    stars: float


def test_matching_dict_passes() -> None:
    assert is_bearable({"title": "Arrival", "year": 2016}, Movie) is True


def test_missing_required_key_fails() -> None:
    assert is_bearable({"title": "Arrival"}, Movie) is False


def test_wrong_value_type_for_key_fails() -> None:
    assert is_bearable({"title": "Arrival", "year": "2016"}, Movie) is False


def test_extra_unknown_keys_are_ignored() -> None:
    assert (
        is_bearable({"title": "Arrival", "year": 2016, "extra": "ignored"}, Movie)
        is True
    )


def test_non_dict_value_fails() -> None:
    assert is_bearable(object(), Movie) is False
    assert is_bearable(["title", "year"], Movie) is False


def test_optional_key_may_be_absent() -> None:
    assert is_bearable({}, Rating) is True


def test_optional_key_checked_when_present() -> None:
    assert is_bearable({"stars": 4.5}, Rating) is True
    assert is_bearable({"stars": "not-a-float"}, Rating) is False


def test_beartype_decorator_raises_on_violation() -> None:
    @beartype
    def summarize(movie: Movie) -> str:
        return f"{movie['title']} ({movie['year']})"

    assert summarize({"title": "Arrival", "year": 2016}) == "Arrival (2016)"
    try:
        summarize({"title": "Arrival"})
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")
