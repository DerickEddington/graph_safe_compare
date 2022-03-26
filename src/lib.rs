#![cfg_attr(unix, doc = include_str!("../README.md"))]
#![cfg_attr(windows, doc = include_str!("..\\README.md"))]
#![cfg_attr(
    not(feature = "std"),
    doc = "\n",
    doc = "Note: This crate was built without its `std` feature and some premade items are \
           unavailable, and so custom types must be provided and used with the items of the \
           [`generic`] module, to have cycle-safety and/or deep-safety."
)]
#![cfg_attr(
    all(not(feature = "std"), feature = "alloc"),
    doc = "\n",
    doc = "Note: This crate was built with its `alloc` feature, and so some premade items, \
           that use the [`alloc`](https://doc.rust-lang.org/alloc/) crate, are available."
)]
// Apply the `no_std` attribute unconditionally, to require explicit `use` of non-`core` items.
#![no_std]
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
    clippy::cargo,
    // Individual lints not included in above groups and desired.
    macro_use_extern_crate,
    meta_variable_misuse,
    // missing_copy_implementations,
    // missing_debug_implementations,
    missing_docs,
    // // missing_doc_code_examples, // maybe someday
    noop_method_call,
    pointer_structural_match,
    single_use_lifetimes, // annoying hits on invisible derived impls
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    variant_size_differences,
)]
// Warn about this one but avoid annoying hits for dev-dependencies.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
// Exclude (re-allow) undesired lints included in above groups.
#![allow(
    clippy::implicit_return,
    clippy::blanket_clippy_restriction_lints,
    clippy::default_numeric_fallback,
    clippy::separated_literal_suffix,
    clippy::missing_docs_in_private_items,
    clippy::pattern_type_mismatch,
    clippy::shadow_reuse
)]
// When our package-feature "anticipate" is activated, cause breaking changes to our API that use
// Rust-features that our crate anticipates adopting in a future version if they become stable.
// While unstable, they must be enabled here; or if some become stable, they will already be
// enabled.
#![cfg_attr(
    all(feature = "anticipate", not(rust_lib_feature = "step_trait")),
    feature(step_trait)
)]
#![cfg_attr(
    all(feature = "anticipate", not(rust_lib_feature = "unwrap_infallible")),
    feature(unwrap_infallible)
)]
#![cfg_attr(
    all(feature = "anticipate", not(rust_lang_feature = "never_type")),
    feature(never_type)
)]


#[cfg(feature = "std")]
/// Items that are safe for cyclic, degenerate, and very-deep graphs.
pub mod robust;

/// Items that are safe for cyclic and degenerate graphs, but not, by themselves, for very-deep
/// graphs.
pub mod cycle_safe;

#[cfg(feature = "alloc")]
/// Items that are safe for very-deep graphs, but not, by themselves, for cyclic nor degenerate
/// graphs.
pub mod deep_safe;

#[cfg(feature = "alloc")]
/// Items that are safe for very-wide graphs, but not, by themselves, for cyclic nor degenerate
/// graphs.
pub mod wide_safe;

/// Items that are not safe for cyclic, degenerate, nor very-deep graphs.
pub mod basic;

/// Items that require choosing specific instantiations, which allows customizability beyond the
/// premade functions of the other modules.  Can be used to achieve cycle-safety and/or
/// deep-safety.
pub mod generic;

/// Miscellaneous utilities that are sometimes useful.
pub mod utils;

cfg_if::cfg_if! {
    if #[cfg(feature = "anticipate")] {
        /// Use of anticipated Rust features.
        mod anticipated;
        use anticipated as anticipated_or_like;
        use core::iter::Step;
    }
    else {
        /// Workarounds, that work with stable versions of Rust, that provide functionality
        /// similar to unstable features that this crate anticipates using once stable.
        mod like_anticipated;
        use like_anticipated as anticipated_or_like;
        pub use like_anticipated::Step;
    }
}


use core::{
    cmp::Ordering,
    hash::Hash,
};


/// What the algorithm requires from a type, to be applied to it.
///
/// The `Self` type is passed by value because that simplifies this crate.  It is possible, and
/// sometimes recommended, to `impl` this trait for reference types (e.g. `&N`, `Rc<N>`, etc.)
/// which often can be passed by value more readily.
pub trait Node: Sized
{
    /// Result of comparing nodes.  Common choices are [`bool`] or [`Ordering`], but it may be
    /// anything that satisfies the trait bounds.
    ///
    /// The result value of the algorithm is that of the first node traversed that compares as
    /// inequivalent, or it is the value that represents equivalence.  E.g. this enables using
    /// types like [`Ordering`] to achieve giving an ordering to graphs that are compared, in
    /// addition to determining equivalence.
    ///
    /// For types where only boolean equivalence is appropriate, [`bool`] should be used.
    type Cmp: Cmp;

    /// Determines when nodes are the same identical node and so can immediately be considered
    /// equivalent without checking their values, edges, nor descendents.  The size of and methods
    /// on this type should be small and very cheap.
    ///
    /// For types where only nodes that are the same object in memory can be considered identical,
    /// pointer equality and hashing should be used by defining this type to be
    /// [`RefId<T>`](crate::utils::RefId) where `T` is some reference type to the primary inner
    /// object type and must not be a reference or pointer to the `Self` type.  The `Self` values
    /// are moved around during the algorithm, and so references or pointers to them would be
    /// invalid as identifiers (because they would not be unique nor consistent).
    ///
    /// For other types where separate objects can represent the same identical node, some
    /// approach following that should be provided, and `RefId` should not be used.
    type Id: Eq + Hash + Clone;

    /// Determines what is used to index descendent nodes and to represent the amount of them.
    /// The primitive unsigned integer types, like `usize`, are a common choice, but it may be
    /// anything that satisfies the trait bounds.
    ///
    /// E.g. for graphs with nodes whose amounts of edges are always smaller than some limit, it
    /// might be desirable, for efficiency, to use an index type smaller than `usize`.  Or for
    /// other node types, it might be more logical or convenient to use an index type that is not
    /// a number.
    ///
    /// This type's implementation of `Default` must give the "zero" value of this type.
    type Index: Step + Default + Ord;

    /// Get the identity of the `self` node.  The result must only be `==` to another node's when
    /// the nodes should be considered identical.
    fn id(&self) -> Self::Id;

    /// Get descendent node by index, if `index` is within the range of the `self` node.  The
    /// algorithm calls this method, until it returns `None`, to descend into each edge.
    #[must_use]
    fn get_edge(
        &self,
        index: &Self::Index,
    ) -> Option<Self>;

    /// Check if the nodes are equivalent in their own directly-contained semantically-significant
    /// values ignoring their edges and ignoring their descendent nodes.
    ///
    /// E.g. a node type like:
    ///
    /// ```rust
    /// struct My {
    ///   value: i32,
    ///   next: Box<My>,
    /// }
    /// ```
    ///
    /// Requires that the implementor decide whether the value of the `value` field should affect
    /// comparison.  Either way is supported.  The implementor could decide to always return
    /// "equivalent" to ignore the field and allow the algorithm to just compare the descendent,
    /// or the implementor could make the result correspond to whether the values of the field are
    /// the same or not.
    ///
    /// Or, e.g. a node type like:
    ///
    /// ```rust
    /// enum My {
    ///   A(Box<My>, Box<My>),
    ///   B(Box<My>, Box<My>),
    /// }
    /// ```
    ///
    /// Requires that the implementor decide whether the difference between the `A` and `B`
    /// variants should affect comparison.  Either way is supported.  Since both variants have the
    /// same amount of edges (assuming [`Self::get_edge`] is implemented like that), the
    /// implementor could decide to always return "equivalent" to ignore differences in the
    /// variants and allow the algorithm to just compare the descendents, or the implementor could
    /// make the result correspond to whether the variants are the same or not.
    ///
    /// Or, e.g. a node type like:
    ///
    /// ```rust
    /// enum My {
    ///   A,
    ///   B(Box<My>),
    /// }
    /// ```
    ///
    /// It is sufficient to always return "equivalent", when [`Self::get_edge`] returns `None` for
    /// the `A` variant or `Some` for the `B` variant, because the algorithm will detect the
    /// unequivalence that way instead.
    fn equiv_modulo_edges(
        &self,
        other: &Self,
    ) -> Self::Cmp;
}


/// Represents comparison of nodes.
///
/// Node types may have richer multi-way comparison than boolean equivalence.
pub trait Cmp
{
    /// Create a new value that represents equivalence is true.
    fn new_equiv() -> Self;

    /// Return `true` if the value represents equivalence, `false` if not.
    fn is_equiv(&self) -> bool;

    /// Create a new value that most-accurately represents the given `Ordering` value.
    ///
    /// Intended for representing comparisons of the amounts of edges of nodes, as checked by
    /// [`EdgesIter::next`](crate::generic::equiv::EdgesIter::next).
    fn from_ord(ord: Ordering) -> Self;
}

impl Cmp for bool
{
    #[inline]
    fn new_equiv() -> Self
    {
        true
    }

    #[inline]
    fn is_equiv(&self) -> bool
    {
        *self
    }

    #[inline]
    fn from_ord(ord: Ordering) -> Self
    {
        ord.is_eq()
    }
}

impl Cmp for Ordering
{
    #[inline]
    fn new_equiv() -> Self
    {
        Ordering::Equal
    }

    #[inline]
    fn is_equiv(&self) -> bool
    {
        self.is_eq()
    }

    #[inline]
    fn from_ord(ord: Ordering) -> Self
    {
        ord
    }
}
