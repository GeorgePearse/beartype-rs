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

mod abc_collections;
mod annotated;
mod callable_hint;
mod classvar_final;
mod containers_fixed_tuple;
mod containers_mapping;
mod containers_sequence;
mod containers_set;
mod forward_ref;
mod generic_typevar;
mod literal;
mod newtype;
mod numeric_tower;
mod numpy_array;
mod paramspec_typevartuple;
mod plain_class;
mod protocol;
mod self_and_special_forms;
mod typed_dict;
mod union_optional;

use crate::registry::{Checker, Registry};

pub fn register_all() -> Registry {
    let checkers: Vec<Box<dyn Checker>> = vec![
        // New categories go above this line, in matches()-specificity order; plain_class
        // is the catch-all for anything that is just a runtime type, so it stays last.

        // Must run before every __origin__-based container/union check: Annotated[dict, "x"]
        // etc. would otherwise be misclaimed by a container checker comparing __origin__.
        Box::new(annotated::AnnotatedChecker),

        // Narrow, exact-marker checks that would otherwise be misclaimed by plain_class
        // (TypedDict/Protocol/Any are all `type`-instances; isinstance() on them can even
        // raise instead of just giving the wrong answer).
        Box::new(typed_dict::TypedDictChecker),
        Box::new(protocol::ProtocolChecker),
        Box::new(self_and_special_forms::SelfAndSpecialFormsChecker),

        // __origin__-based structural categories.
        Box::new(union_optional::UnionOptionalChecker),
        Box::new(literal::LiteralChecker),
        Box::new(classvar_final::ClassvarFinalChecker),
        Box::new(callable_hint::CallableHintChecker),
        Box::new(generic_typevar::GenericTypevarChecker),
        Box::new(paramspec_typevartuple::ParamspecTypevartupleChecker),
        Box::new(newtype::NewtypeChecker),
        Box::new(forward_ref::ForwardRefChecker),

        // Container categories - each matches()'s a disjoint __origin__/identity, no
        // ordering constraint among themselves, but containers_fixed_tuple's matches()
        // excludes variadic Tuple[int, ...] (left for containers_sequence), so put it
        // first among the two tuple-adjacent checkers per its own ordering note.
        Box::new(containers_fixed_tuple::ContainersFixedTupleChecker),
        Box::new(containers_sequence::ContainersSequenceChecker),
        Box::new(containers_set::ContainersSetChecker),
        Box::new(containers_mapping::ContainersMappingChecker),

        // Identity-marker checks against collections.abc / numbers / numpy; disjoint from
        // everything else, but still ahead of plain_class by convention.
        Box::new(abc_collections::AbcCollectionsChecker),
        Box::new(numeric_tower::NumericTowerChecker),
        Box::new(numpy_array::NumpyArrayChecker),

        Box::new(plain_class::PlainClassChecker),
    ];
    Registry::new(checkers)
}
