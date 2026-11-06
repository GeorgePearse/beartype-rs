"""Quick benchmark: beartype_rs.is_bearable() vs. plain isinstance() vs. (optionally) beartype.

Run with:

    python benchmarks/compare.py

This is a rough, single-process microbenchmark (time.perf_counter, a few thousand
iterations per case) meant to give a quick sense of relative overhead -- not a
rigorous statistical comparison.
"""

from __future__ import annotations

import time
from typing import Any, Dict, List, Union

from beartype_rs import is_bearable as rs_is_bearable

try:
    from beartype.door import is_bearable as bt_is_bearable

    _HAVE_BEARTYPE = True
except ImportError:
    _HAVE_BEARTYPE = False

ITERATIONS = 5_000

# Each case: (label, value, hint, isinstance_check)
# `isinstance_check` is a plain-Python stand-in for the hint, using isinstance()
# directly (isinstance() can't take things like List[int] or Union[...] itself).
_LIST_INT = list(range(100))
_DICT_STR_INT = {f"key{i}": i for i in range(50)}

CASES = [
    (
        "int",
        123,
        int,
        lambda v: isinstance(v, int),
    ),
    (
        "List[int] (len 100)",
        _LIST_INT,
        List[int],
        lambda v: isinstance(v, list) and all(isinstance(x, int) for x in v),
    ),
    (
        "Dict[str, int] (len 50)",
        _DICT_STR_INT,
        Dict[str, int],
        lambda v: isinstance(v, dict)
        and all(isinstance(k, str) and isinstance(val, int) for k, val in v.items()),
    ),
    (
        "Union[int, str]",
        "hello",
        Union[int, str],
        lambda v: isinstance(v, (int, str)),
    ),
]


def _time_calls(fn, *args, iterations: int = ITERATIONS) -> float:
    """Return nanoseconds per call of fn(*args), averaged over `iterations` calls."""
    start = time.perf_counter()
    for _ in range(iterations):
        fn(*args)
    elapsed = time.perf_counter() - start
    return (elapsed / iterations) * 1e9


def main() -> None:
    if not _HAVE_BEARTYPE:
        print("Note: `beartype` package is not installed; skipping that comparison.\n")

    rows = []
    for label, value, hint, isinstance_check in CASES:
        rs_ns = _time_calls(rs_is_bearable, value, hint)
        isinstance_ns = _time_calls(isinstance_check, value)
        row = [label, f"{rs_ns:,.1f}", f"{isinstance_ns:,.1f}"]
        if _HAVE_BEARTYPE:
            bt_ns = _time_calls(bt_is_bearable, value, hint)
            row.append(f"{bt_ns:,.1f}")
        rows.append(row)

    headers = ["hint", "beartype_rs (ns/call)", "isinstance (ns/call)"]
    if _HAVE_BEARTYPE:
        headers.append("beartype (ns/call)")

    widths = [
        max(len(headers[i]), *(len(row[i]) for row in rows)) for i in range(len(headers))
    ]

    def fmt_row(cells: list[str]) -> str:
        return "  ".join(cell.ljust(widths[i]) for i, cell in enumerate(cells))

    print(fmt_row(headers))
    print("  ".join("-" * w for w in widths))
    for row in rows:
        print(fmt_row(row))


if __name__ == "__main__":
    main()
