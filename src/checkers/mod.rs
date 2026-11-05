//! One module per typing-construct category. Each implements `crate::registry::Checker`
//! and adds itself below in `register_all`.
//!
//! **Adding a category**: create `src/checkers/your_category.rs`, implement `Checker` for
//! a small struct, add `mod your_category;` below and one `Box::new(your_category::...)` to
//! the `vec!` in `register_all`. That is the entire integration surface - do not add
//! branches anywhere else for a new category.
//!
//! Order matters: `matches` runs top to bottom and the first `true` wins, so put narrower
//! checks (an exact `__origin__` match) before broader fallbacks (`plain_class`, which
//! claims anything `isinstance` already understands and must stay last).

mod plain_class;

use crate::registry::{Checker, Registry};

pub fn register_all() -> Registry {
    let checkers: Vec<Box<dyn Checker>> = vec![
        // New categories go above this line, in matches()-specificity order; plain_class
        // is the catch-all for anything that is just a runtime type, so it stays last.
        Box::new(plain_class::PlainClassChecker),
    ];
    Registry::new(checkers)
}
