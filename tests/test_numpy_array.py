"""Best-effort numpy support: the bare `numpy.ndarray` class used directly as a hint
(no shape/dtype parametrisation - that's out of scope). Skips gracefully if numpy isn't
installed in the test environment, since numpy is an optional dependency as far as
beartype-rs itself is concerned."""

import pytest

from beartype_rs import BeartypeCallHintViolation, beartype, is_bearable

np = pytest.importorskip("numpy")


def test_ndarray_passes_for_an_actual_array() -> None:
    assert is_bearable(np.array([1, 2, 3]), np.ndarray) is True


def test_ndarray_fails_for_a_non_array_value() -> None:
    assert is_bearable([1, 2, 3], np.ndarray) is False
    assert is_bearable(5, np.ndarray) is False


def test_beartype_decorator_passes_a_matching_array() -> None:
    @beartype
    def total(arr: np.ndarray) -> float:
        return float(arr.sum())

    assert total(np.array([1.0, 2.0, 3.0])) == 6.0


def test_beartype_decorator_raises_on_violation() -> None:
    @beartype
    def total(arr: np.ndarray) -> float:
        return float(arr.sum())

    try:
        total([1.0, 2.0, 3.0])
    except BeartypeCallHintViolation:
        pass
    else:
        raise AssertionError("expected a BeartypeCallHintViolation")
