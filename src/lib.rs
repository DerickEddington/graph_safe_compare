//! TODO

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
    unused_crate_dependencies,
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
    clippy::pattern_type_mismatch
)]

pub mod proto;

#[cfg(test)]
mod tests;
