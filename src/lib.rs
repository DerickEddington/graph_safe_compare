//! Generic equivalence predicate that can handle cyclic, very-deep, very-large, and degenerate
//! graphs.  Extends the algorithm described in the paper [Efficient Nondestructive Equality
//! Checking for Trees and Graphs](https://michaeldadams.org/papers/efficient_equality/).  TODO:
//! Has further enhancements, like ordering comparison ...

#![forbid(unsafe_code)]
// Warn about desired lints that would otherwise be allowed by default.
#![warn(
    // Groups
    future_incompatible,
    nonstandard_style,
    rust_2018_compatibility, // unsure if needed with edition="2018"
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,
    clippy::all,
    clippy::pedantic,
    clippy::restriction,
    // Individual lints not included in above groups and desired.
    macro_use_extern_crate,
    meta_variable_misuse,
    // missing_copy_implementations,
    // missing_debug_implementations,
    // missing_docs,
    // // missing_doc_code_examples, // maybe someday
    noop_method_call,
    pointer_structural_match,
    single_use_lifetimes, // annoying hits on invisible derived impls
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    // unused_crate_dependencies, // annoying hits for dev-dependencies
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences,
)]
// Exclude (re-allow) undesired lints included in above groups.
#![allow(
    clippy::implicit_return,
    clippy::blanket_clippy_restriction_lints,
    clippy::pattern_type_mismatch,
    clippy::shadow_reuse
)]
// TODO: Temporary
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::missing_panics_doc,
    clippy::exhaustive_enums,
    clippy::missing_inline_in_public_items,
    clippy::similar_names,
    clippy::default_numeric_fallback
)]


/// The most robust and most generic variation of the algorithm.  It supports:
/// TODO:
///
/// - Graphs: cyclic, directed-acyclic, degenerate, tree.
///
/// - Very-deep graphs, limited only by available memory, by not using the call stack for the
/// recursions done while traversing.  This does its own tail-call elimination where possible
/// which can reduce memory consumption significantly for some graph shapes.
///
/// - Very-large graphs that cannot fit in memory at once, by loading nodes as needed and then
/// unloading them, and by unloading/loading internal data structures to/from persistent storage.
/// For graph types that do not need this, there is no extra cost for this, thanks to this being
/// optimized away when the generic implementation is instantiated with such types.
///
/// - Optionally giving a custom allocator.  This could impose limits on the memory consumption.
/// Fallible allocations are supported for which an error type is returned.
///
/// - TODO: Additional aspects I hope to achieve...
pub mod robust;


/// Alternative variations of the algorithm that do not have all the properties of the robust
/// algorithm, for uses where only some of the properties are needed and better efficiency can be
/// gained.
pub mod alt
{
    /// Variation that uses the normal function call stack, and so cannot handle very-deep graphs
    /// because stack overflow can happen, and that does not load/unload nodes (TODO: actually it
    /// might be possible to load/unload), and so cannot handle graphs larger than available
    /// memory. For inputs that are not too deep and that can fit in memory at once, this is
    /// faster (TODO: benchmarks to confirm).
    ///
    /// It supports:
    /// - Graphs: cyclic, directed-acyclic, degenerate, tree.
    /// - Generic node types.
    /// - TODO?: Optionally giving a custom allocator?
    pub mod basic;
}
