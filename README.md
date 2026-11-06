# beartype-rs

Rust-accelerated runtime type checking for Python, in the spirit of [beartype](https://github.com/beartype/beartype).

```python
from beartype_rs import beartype

@beartype
def greet(name: str) -> str:
    return f"hello, {name}"

greet(3)  # BeartypeCallHintViolation: greet() parameter name=3 violates type hint <class 'str'>
```

## Status

Early: the core engine (a Rust-side registry of per-category type-hint checkers, exposed
to Python as `is_bearable`/`die_if_unbearable`, wrapped by a `@beartype` decorator) is in
place and covers plain classes (`isinstance`-style checks). See
[`ARCHITECTURE.md`](ARCHITECTURE.md) for how a new category (containers, unions, literals,
protocols, ...) gets added - each lives in its own file under `src/checkers/` against a
documented trait, so categories can be built independently without stepping on each other.

## Build

```bash
uv venv .venv
uvx maturin develop --uv -m Cargo.toml
.venv/bin/python -m pytest tests/ -q   # Python-level tests
cargo test                             # Rust-level unit tests
```

## Name

`beartype-rs` is a working title. Maybe we should call it **grizzly**.
