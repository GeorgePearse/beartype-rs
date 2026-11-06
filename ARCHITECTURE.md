# Architecture

## Layout

- `src/lib.rs` - the PyO3 module (`beartype_rs._beartype_rs`). Exposes exactly two
  functions to Python: `is_bearable(value, hint) -> bool` and
  `die_if_unbearable(value, hint, description) -> None`. Do not add more surface here;
  everything else is a `Checker`.
- `src/registry.rs` - the `Checker` trait and the `Registry` that dispatches a hint to
  the checker that claims it.
- `src/introspect.rs` - shared helpers for picking apart a Python typing hint
  (`origin`, `args`, `is_plain_class`, `describe`). Use these instead of calling
  `getattr`/`typing.get_origin` yourself; every checker reading `__origin__` slightly
  differently is how this kind of project rots.
- `src/checkers/` - one file per category. `src/checkers/mod.rs`'s `register_all()` is
  the *only* place a new category gets wired in.
- `python/beartype_rs/__init__.py` - the `@beartype` decorator, `is_bearable`,
  `die_if_unbearable`, `BeartypeCallHintViolation`. Pure Python; it never touches
  `typing` internals the Rust side should own.
- `tests/` - pytest, run against the built extension (`maturin develop` first).

## Adding a checker category

1. Create `src/checkers/your_category.rs`.
2. Implement `crate::registry::Checker` for a small unit struct:

   ```rust
   use pyo3::prelude::*;
   use pyo3::types::PyAny;
   use crate::introspect;
   use crate::registry::{Checker, Registry};

   pub struct YourCategoryChecker;

   impl Checker for YourCategoryChecker {
       fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
           // Cheap check: is this hint one of mine? e.g. compare introspect::origin(hint)
           // against the builtin you care about, or check hint.getattr("__class__") for
           // typing-internal marker classes (see "Finding the right marker" below).
           todo!()
       }

       fn check(&self, py: Python<'_>, hint: &Bound<'_, PyAny>, value: &Bound<'_, PyAny>, registry: &Registry) -> PyResult<bool> {
           // Do the actual check. Recurse through `registry.check(py, nested_hint, nested_value)`
           // for any inner hints (e.g. List[int]'s element type) - never assume which
           // checker will end up handling a nested hint.
           todo!()
       }

       fn name(&self) -> &'static str {
           "your_category"
       }
   }
   ```

3. In `src/checkers/mod.rs`: add `mod your_category;` near the top, and one
   `Box::new(your_category::YourCategoryChecker)` to the `vec!` in `register_all()`.
   Put it **before** `plain_class` (the catch-all stays last) and, among the others,
   roughly in order of how specific your `matches()` check is - an exact-origin check
   before anything that might loosely overlap with it.
4. Add `#[cfg(test)]` unit tests in the same file exercising `matches`/`check` directly
   (see `src/checkers/plain_class.rs` for the pattern once a second checker exists, or
   write the pattern yourself if you're first).
5. Add a pytest file under `tests/` exercising your category through the *public*
   Python API (`is_bearable`, and a `@beartype`-decorated function) - the thing a real
   user will actually call, not the Rust internals.

## Finding the right marker for a typing construct

Different `typing` constructs are identifiable in different ways; check in the Python
REPL before writing `matches()`, don't guess:

```python
>>> import typing
>>> typing.get_origin(list[int])       # list
>>> typing.get_origin(typing.Union[int, str])  # typing.Union
>>> typing.get_origin(typing.Literal[1, 2])    # typing.Literal
>>> type(typing.TypedDict("X", {}))            # a metaclass you can isinstance() against
>>> typing.get_type_hints(SomeNewType)         # NewType is callable, has .__supertype__
```

`introspect::origin` gives you `__origin__` directly (what `typing.get_origin` reads
for the common case); for hints that don't follow that pattern (`Literal`, `TypedDict`,
`NewType`, `Annotated`, `ParamSpec`, `TypeVarTuple`...) check the hint's own attributes
or `type(hint)` instead - there's no one-size-fits-all `matches()` shape, which is
exactly why each category needing its own file matters.

## Error messages

`Checker::check` returning `Ok(false)` is enough - the Python side
(`die_if_unbearable` in `python/beartype_rs/__init__.py`) builds the actual message
using the function name, parameter name and repr it has and the Rust side doesn't. Do
not try to build a descriptive message in Rust; return a plain bool.

## Build & test

```bash
uv venv .venv   # once
uvx maturin develop --uv -m Cargo.toml   # after any Rust change
.venv/bin/python -m pytest tests/ -q
cargo test      # Rust-only unit tests, no Python needed
```
