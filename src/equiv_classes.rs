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

// TODO: Make these conditional on new feature(s), and/or move when the table is made generic.
extern crate alloc;
extern crate std;

use {
    alloc::rc::Rc,
    core::{
        cell::Cell,
        hash::Hash,
        ptr,
    },
    std::collections::HashMap,
};


/// Membership in an equivalence class.
///
/// Optimized for merging with other classes and for checking membership in the same class.
#[derive(Clone)]
enum Class
{
    /// An equivalence class.
    ///
    /// Distinct objects of this variant represent distinct equivalence classes.
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
    Link
    {
        /// The other class that we merged with.
        next: Rc<Cell<Self>>,
    },
}

impl Default for Class
{
    fn default() -> Self
    {
        Self::Representative { weight: 1 }
    }
}

impl Class
{
    /// Create a distinct [`Representative`](Self::Representative) object that represents a
    /// distinct equivalence class, with initial weight.
    fn new() -> Rc<Cell<Self>>
    {
        Rc::new(Cell::new(Self::default()))
    }

    /// This type uses `Cell`, instead of `RefCell`, so that failures are impossible, which
    /// requires the approach of this function because our type cannot be `Copy`.
    fn clone_inner(it: &Rc<Cell<Self>>) -> Self
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
    fn get_rep_and_weight(it: &Rc<Cell<Self>>) -> (Rc<Cell<Self>>, usize)
    {
        let it_inner = Self::clone_inner(it);
        match it_inner {
            Self::Representative { weight } => (Rc::clone(it), weight),

            Self::Link { mut next } => {
                let mut cur = Rc::clone(it);
                loop {
                    let next_inner = Self::clone_inner(&next);
                    match next_inner {
                        Self::Representative { weight } => break (next, weight),

                        Self::Link { next: next_next } => {
                            cur.set(Self::Link { next: Rc::clone(&next_next) });
                            cur = next;
                            next = next_next;
                        },
                    }
                }
            },
        }
    }

    /// Like [`get_rep_and_weight`](Self::get_rep_and_weight) but only returns the representative.
    fn get_rep(it: &Rc<Cell<Self>>) -> Rc<Cell<Self>>
    {
        Self::get_rep_and_weight(it).0
    }

    /// Use [`ptr::eq`] to compare references to [`Representative`](Self::Representative) objects,
    /// so that distinct objects represent distinct equivalence classes.
    fn eq_rep(
        it: &Rc<Cell<Self>>,
        other: &Rc<Cell<Self>>,
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
        it: &Rc<Cell<Self>>,
        weight: usize,
    )
    {
        it.set(Self::Representative { weight });
    }

    /// Set `it` to be a [`Link`](Self::Link) to the given `next`.
    fn set_link(
        it: &Rc<Cell<Self>>,
        next: Rc<Cell<Self>>,
    )
    {
        it.set(Self::Link { next });
    }
}


/// The classes of the nodes that are known to be equivalent, for an invocation of the algorithm.
pub(super) struct EquivClasses<K>
{
    /// Table that associates nodes by ID with their equivalence class.
    map: HashMap<K, Rc<Cell<Class>>>,
}

impl<K: Eq + Hash + Clone> EquivClasses<K>
{
    pub(super) fn new() -> Self
    {
        Self { map: HashMap::new() }
    }

    /// First time both nodes are seen.
    ///
    /// Immediately record them as being in the same equivalence class, before checking their
    /// descendents, by associating their IDs with a new equivalence class.
    fn none_seen(
        &mut self,
        ak: &K,
        bk: &K,
    )
    {
        let ac = Class::new();
        let bc = Rc::clone(&ac);
        let _ignored1 = self.map.insert(ak.clone(), ac);
        let _ignored2 = self.map.insert(bk.clone(), bc);
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
        oc: &Rc<Cell<Class>>,
        k: &K,
    )
    {
        let r = Class::get_rep(oc);
        let _ignored = self.map.insert(k.clone(), r);
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
        ac: &Rc<Cell<Class>>,
        bc: &Rc<Cell<Class>>,
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
    pub(super) fn same_class(
        &mut self,
        ak: &K,
        bk: &K,
    ) -> bool
    {
        match (self.map.get(ak), self.map.get(bk)) {
            (None, None) => {
                self.none_seen(ak, bk);
                false
            },
            (Some(ac), None) => {
                let ac = &Rc::clone(ac); // To end borrow of `self`.
                self.some_seen(ac, bk);
                false
            },
            (None, Some(bc)) => {
                let bc = &Rc::clone(bc); // To end borrow of `self`.
                self.some_seen(bc, ak);
                false
            },
            (Some(ac), Some(bc)) => Self::all_seen(ac, bc),
        }
    }
}


#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn eq_rep()
    {
        fn rep(weight: usize) -> Rc<Cell<Class>>
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

    #[test]
    fn get_rep()
    {
        fn link(next: &Rc<Cell<Class>>) -> Rc<Cell<Class>>
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

    #[test]
    fn same_class()
    {
        let mut ec = EquivClasses::new();
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
