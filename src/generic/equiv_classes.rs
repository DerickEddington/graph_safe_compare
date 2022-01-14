//! The essential technique of the algorithm, that enables it to handle cyclic and degenerate
//! graphs efficiently, is determining whether nodes, at the same position/path in the input
//! graphs, have already been seen and are already known to be equivalent, before descending their
//! edges.
//!
//! The mechanism that acheives this is a table that associates nodes by ID with a representation
//! of equivalence classes that is optimized for its needed operations of making a union of
//! classes by merging the links to it and of checking if nodes are members of the same class by
//! following the links to it.  (It is not needed to be able to iterate the members of a given
//! class, and the type cannot do that.)
//!
//! For the internal representation, interior mutability is required due to requiring shared
//! ownership and multiple borrows.  [`Cell`] is used because failures are impossible with it
//! (unlike [`RefCell`](core::cell::RefCell)).

use {
    crate::Node,
    core::{
        cell::Cell,
        ops::Deref,
        ptr,
    },
};


/// Allows being generic over the type that provides the needed shared ownership of [`Class`].
pub trait Rc: Deref<Target = Cell<Class<Self>>> + Clone
{
    /// Create a new shared-ownership allocation for `value`.
    fn new(value: Cell<Class<Self>>) -> Self;
}

/// Membership in an equivalence class.
///
/// Optimized for merging with other classes and for checking membership in the same class.
#[derive(Clone)]
#[non_exhaustive]
pub enum Class<R>
{
    /// An equivalence class.
    ///
    /// Distinct objects of this variant represent distinct equivalence classes.
    #[non_exhaustive]
    Representative
    {
        /// Determines which `Representative` object is merged when nodes are recorded as
        /// equivalent when both had already been seen separately.
        weight: usize,
    },
    /// A merger with another equivalence class.
    ///
    /// This makes all nodes in our class be equivalent to all nodes in the other class, by
    /// transitivity.
    #[non_exhaustive]
    Link
    {
        /// The other class that we merged with.
        next: R,
    },
}

impl<R> Default for Class<R>
{
    #[inline]
    fn default() -> Self
    {
        Self::Representative { weight: 1 }
    }
}

impl<R: Rc> Class<R>
{
    /// Create a distinct [`Representative`](Self::Representative) object that represents a
    /// distinct equivalence class, with initial weight.
    #[allow(clippy::new_ret_no_self)] // It actually does return a type that "contains" `Self`.
    fn new() -> R
    {
        R::new(Cell::new(Self::default()))
    }

    /// This type uses `Cell`, instead of `RefCell`, so that failures are impossible, which
    /// requires the approach of this function because our type cannot be `Copy`.
    fn clone_inner(it: &R) -> Self
    {
        let dummy = Self::default();
        let inner = it.replace(dummy);
        let result = inner.clone();
        it.set(inner);
        result
    }

    /// Get the representative, and its weight, of the equivalence class `it` is a member of
    /// currently.
    ///
    /// Follows [`Link`](Self::Link) chains to the distinct
    /// [`Representative`](Self::Representative) object.  Long chains are shortened, to improve
    /// efficiency for subsequent traversals.
    fn get_rep_and_weight(it: &R) -> (R, usize)
    {
        let it_inner = Self::clone_inner(it);
        match it_inner {
            Self::Representative { weight } => (R::clone(it), weight),

            Self::Link { mut next } => {
                let mut cur = R::clone(it);
                loop {
                    let next_inner = Self::clone_inner(&next);
                    match next_inner {
                        Self::Representative { weight } => break (next, weight),

                        Self::Link { next: next_next } => {
                            cur.set(Self::Link { next: R::clone(&next_next) });
                            cur = next;
                            next = next_next;
                        },
                    }
                }
            },
        }
    }

    /// Like [`get_rep_and_weight`](Self::get_rep_and_weight) but only returns the representative.
    fn get_rep(it: &R) -> R
    {
        Self::get_rep_and_weight(it).0
    }

    /// Use [`ptr::eq`] to compare references to [`Representative`](Self::Representative) objects,
    /// so that distinct objects represent distinct equivalence classes.
    fn eq_rep(
        it: &R,
        other: &R,
    ) -> bool
    {
        debug_assert!(matches!(
            (Self::clone_inner(it), Self::clone_inner(other)),
            (Self::Representative { .. }, Self::Representative { .. })
        ));

        ptr::eq(&**it, &**other)
    }

    /// Set `it` to be a [`Representative`](Self::Representative) with the given `weight`.
    fn set_rep(
        it: &R,
        weight: usize,
    )
    {
        it.set(Self::Representative { weight });
    }

    /// Set `it` to be a [`Link`](Self::Link) to the given `next`.
    fn set_link(
        it: &R,
        next: R,
    )
    {
        it.set(Self::Link { next });
    }
}


/// Allows being generic over the type that provides the table that associates nodes by ID with
/// their equivalence classes.
pub trait Table: Default
{
    /// The node type that a `Self` table handles.
    type Node: Node;
    /// Allows customizing the type that provides the needed shared ownership of equivalence
    /// classes.
    type Rc: Rc;

    /// Lookup a node ID and return its equivalence class if associated.
    fn get(
        &self,
        k: &<Self::Node as Node>::Id,
    ) -> Option<&Self::Rc>;

    /// Associate a node ID with an equivalence class.
    fn insert(
        &mut self,
        k: <Self::Node as Node>::Id,
        v: Self::Rc,
    );
}

/// The classes of the nodes that are known to be equivalent, for an invocation of the algorithm.
#[derive(Default)]
pub(crate) struct EquivClasses<T>
{
    /// Table that associates nodes by ID with their equivalence class.
    table: T,
}

impl<T: Table> EquivClasses<T>
{
    /// First time both nodes are seen.
    ///
    /// Immediately record them as being in the same equivalence class, before checking their
    /// descendents, by associating their IDs with a new equivalence class.
    fn none_seen(
        &mut self,
        ak: &<T::Node as Node>::Id,
        bk: &<T::Node as Node>::Id,
    )
    {
        let ac = Class::new();
        let bc = T::Rc::clone(&ac);
        self.table.insert(ak.clone(), ac);
        self.table.insert(bk.clone(), bc);
    }

    /// First time one node is seen but the other has already been seen.
    ///
    /// Immediately record them as being in the same equivalence class, before checking their
    /// descendents, by associating the ID of the unseen with the equivalence class of the seen.
    ///
    /// This also causes any further nodes that were already members of the class to now be
    /// transitively equivalent to the unseen, which can improve efficiency for some shapes.
    fn some_seen(
        &mut self,
        oc: &T::Rc,
        k: &<T::Node as Node>::Id,
    )
    {
        let r = Class::get_rep(oc);
        self.table.insert(k.clone(), r);
    }

    /// Both nodes have already been seen, but maybe not already known to be equivalent.
    ///
    /// Return `true` if already recorded as being in the same equivalence class.
    ///
    /// Else return `false`, and immediately record them as being in the same equivalence class,
    /// before checking their descendents, by merging their classes into a union.  This also
    /// causes any further nodes that were already members of the classes to now be transitively
    /// equivalent to each other, which can improve efficiency for some shapes.
    fn all_seen(
        ac: &T::Rc,
        bc: &T::Rc,
    ) -> bool
    {
        let (ar, aw) = Class::get_rep_and_weight(ac);
        let (br, bw) = Class::get_rep_and_weight(bc);

        // Already same class.
        if Class::eq_rep(&ar, &br) {
            true
        }
        // Merge classes, according to the "weighted union rule" as prescribed by the paper.
        else {
            let (larger_rep, smaller_rep);

            if aw >= bw {
                larger_rep = ar;
                smaller_rep = br;
            }
            else {
                larger_rep = br;
                smaller_rep = ar;
            }
            Class::set_rep(&larger_rep, aw.saturating_add(bw));
            Class::set_link(&smaller_rep, larger_rep);

            false
        }
    }

    /// Check if the given node IDs are already known to be equivalent.  If not, they will be made
    /// members of the same equivalence class, merging classes if needed.
    ///
    /// Returns `true` if they were already recorded as equivalent, and their descendents do not
    /// need to be checked.  Returns `false` if not already recorded as equivalent, and their
    /// descendents do need to be checked, and they will be recorded as equivalent for if they are
    /// seen again.
    ///
    /// After returning `false`, their descendents will be checked for equivalence, which might
    /// lead the traversal to these same nodes (cyclic) or to other nodes (DAG) that have been or
    /// will be merged into the same equivalence class.
    pub(crate) fn same_class(
        &mut self,
        ak: &<T::Node as Node>::Id,
        bk: &<T::Node as Node>::Id,
    ) -> bool
    {
        match (self.table.get(ak), self.table.get(bk)) {
            (None, None) => {
                self.none_seen(ak, bk);
                false
            },
            (Some(ac), None) => {
                let ac = &T::Rc::clone(ac); // To end borrow of `self`.
                self.some_seen(ac, bk);
                false
            },
            (None, Some(bc)) => {
                let bc = &T::Rc::clone(bc); // To end borrow of `self`.
                self.some_seen(bc, ak);
                false
            },
            (Some(ac), Some(bc)) => Self::all_seen(ac, bc),
        }
    }
}


#[cfg(any(feature = "alloc", feature = "std"))]
/// Items made for ready use as specific choices for the generic types of the equivalence classes
/// mechanisms.
pub mod premade
{
    #[cfg(feature = "alloc")]
    pub use alloc::*;
    #[cfg(feature = "std")]
    pub use std::*;

    #[cfg(feature = "alloc")]
    mod alloc
    {
        //! Support for [`alloc`] things.

        extern crate alloc;

        /// Support for standard [`Rc`](alloc::rc::Rc).
        pub mod rc
        {
            use {
                super::{
                    super::super::{
                        Class,
                        Rc as RcTrait,
                    },
                    alloc,
                },
                core::{
                    cell::Cell,
                    ops::Deref,
                },
            };

            /// Enables standard [`Rc`](alloc::rc::Rc) to be used as an
            /// [`equiv_classes::Rc`](RcTrait), which requires this recursive type.
            #[derive(Clone)]
            pub struct Rc(alloc::rc::Rc<Cell<Class<Self>>>);

            impl Deref for Rc
            {
                type Target = Cell<Class<Self>>;

                #[inline]
                fn deref(&self) -> &Self::Target
                {
                    &*self.0
                }
            }

            impl RcTrait for Rc
            {
                #[inline]
                fn new(val: Cell<Class<Self>>) -> Self
                {
                    Self(alloc::rc::Rc::new(val))
                }
            }
        }
    }

    #[cfg(feature = "std")]
    mod std
    {
        //! Support for [`std`] things.

        extern crate std;

        /// Support for standard [`HashMap`](std::collections::HashMap).
        pub mod hash_map
        {
            use {
                super::{
                    super::{
                        super::Table as TableTrait,
                        rc::Rc,
                    },
                    std,
                },
                crate::Node,
                std::collections::HashMap,
            };

            /// Generic parameters of [`Table`] and its operations.
            pub trait Params
            {
                /// Amount of elements (i.e. branch (non-leaf) nodes) that a table can grow to
                /// contain initially before reallocating.
                ///
                /// The default value is a balance between being somewhat large to avoid excessive
                /// reallocations and not being too huge that it often consumes excessive memory.
                /// If the default is not good for your use case, a custom `impl` of [`Params`]
                /// may be made with a more-appropriate value - either smaller or larger.  Note
                /// that the default only affects the initial capacity of the underlying
                /// [`HashMap`], and it will still grow as large as needed regardless by
                /// reallocating.
                const INITIAL_CAPACITY: usize = 2_usize.pow(12);
                /// Type of node that is recorded in the table.  Must be the same as used with the
                /// corresponding [`equiv::Params`](crate::generic::equiv::Params).
                type Node: Node;
            }

            /// Eases using standard [`HashMap`] as an [`equiv_classes::Table`](TableTrait) that
            /// uses [`Rc`].
            pub struct Table<P: Params>(HashMap<<P::Node as Node>::Id, Rc>);

            impl<P: Params> Default for Table<P>
            {
                /// Create a new instance with capacity
                /// [`P::INITIAL_CAPACITY`](Params::INITIAL_CAPACITY).
                #[inline]
                fn default() -> Self
                {
                    Self(HashMap::with_capacity(P::INITIAL_CAPACITY))
                }
            }

            impl<P: Params> TableTrait for Table<P>
            {
                type Node = P::Node;
                type Rc = Rc;

                #[inline]
                fn get(
                    &self,
                    k: &<Self::Node as Node>::Id,
                ) -> Option<&Self::Rc>
                {
                    HashMap::get(&self.0, k)
                }

                #[inline]
                fn insert(
                    &mut self,
                    k: <Self::Node as Node>::Id,
                    v: Self::Rc,
                )
                {
                    drop(HashMap::insert(&mut self.0, k, v));
                }
            }
        }
    }
}


#[cfg(test)]
mod tests
{
    #[allow(unused_imports)]
    use super::*;

    #[cfg(feature = "alloc")]
    #[test]
    fn eq_rep()
    {
        use premade::rc::Rc;

        fn rep(weight: usize) -> Rc
        {
            Rc::new(Cell::new(Class::Representative { weight }))
        }

        {
            let r = &rep(0);
            assert!(Class::eq_rep(r, r));
        }
        {
            let r1 = rep(2);
            let r2 = Rc::clone(&r1);
            assert!(Class::eq_rep(&r1, &r2));
        }
        {
            let r1 = rep(3);
            let r2 = rep(3);
            assert!(!Class::eq_rep(&r1, &r2));
        }
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn get_rep()
    {
        use premade::rc::Rc;

        fn link(next: &Rc) -> Rc
        {
            let new = Class::new();
            Class::set_link(&new, Rc::clone(next));
            new
        }

        let rep1 = Class::new();
        let link1 = link(&rep1);
        let link2 = link(&rep1);
        let link3 = link(&link1);
        let link4 = link(&link2);
        let link5 = link(&link4);
        let link6 = link(&link5);

        assert!(Class::eq_rep(&Class::get_rep(&link1), &rep1));
        assert!(Class::eq_rep(&Class::get_rep(&link2), &rep1));
        assert!(Class::eq_rep(&Class::get_rep(&link3), &rep1));
        assert!(Class::eq_rep(&Class::get_rep(&link4), &rep1));
        assert!(Class::eq_rep(&Class::get_rep(&link5), &rep1));
        assert!(Class::eq_rep(&Class::get_rep(&link6), &rep1));

        let rep2 = Class::new();
        let link7 = link(&rep2);
        assert!(!Class::eq_rep(&Class::get_rep(&link7), &rep1));
    }

    #[cfg(feature = "std")]
    #[test]
    fn same_class()
    {
        use premade::hash_map::{
            Params,
            Table,
        };

        struct Args;

        impl Params for Args
        {
            type Node = CharKeyed;
        }

        #[derive(Clone)]
        struct CharKeyed;

        #[allow(clippy::unreachable)]
        impl Node for CharKeyed
        {
            type Cmp = bool;
            type Id = char;
            type Index = u8;

            fn id(&self) -> Self::Id
            {
                unreachable!()
            }

            fn amount_edges(&self) -> Self::Index
            {
                unreachable!()
            }

            fn get_edge(
                &self,
                _index: &Self::Index,
            ) -> Self
            {
                unreachable!()
            }

            fn equiv_modulo_edges(
                &self,
                _other: &Self,
            ) -> bool
            {
                unreachable!()
            }
        }

        let mut ec = EquivClasses::<Table<Args>>::default();
        let keys = ['a', 'b', 'c', 'd', 'e', 'f', 'g'];

        assert!(!ec.same_class(&keys[0], &keys[1]));
        assert!(ec.same_class(&keys[0], &keys[1]));

        assert!(!ec.same_class(&keys[0], &keys[2]));
        assert!(ec.same_class(&keys[0], &keys[2]));
        assert!(ec.same_class(&keys[1], &keys[2]));

        assert!(!ec.same_class(&keys[3], &keys[2]));
        assert!(ec.same_class(&keys[3], &keys[2]));
        assert!(ec.same_class(&keys[3], &keys[1]));
        assert!(ec.same_class(&keys[3], &keys[0]));

        assert!(!ec.same_class(&keys[4], &keys[5]));
        assert!(ec.same_class(&keys[4], &keys[5]));

        assert!(!ec.same_class(&keys[5], &keys[6]));
        assert!(ec.same_class(&keys[5], &keys[6]));
        assert!(ec.same_class(&keys[4], &keys[6]));

        assert!(!ec.same_class(&keys[1], &keys[4]));
        assert!(ec.same_class(&keys[1], &keys[4]));
        assert!(ec.same_class(&keys[1], &keys[5]));
        assert!(ec.same_class(&keys[1], &keys[6]));

        for a in &keys {
            for b in &keys {
                assert!(ec.same_class(a, b));
            }
        }
    }
}
