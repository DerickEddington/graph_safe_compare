//! The essential technique of the algorithm, that enables it to handle cyclic and degenerate
//! graphs efficiently, is determining whether nodes, at the same position/path in the input
//! graphs, have already been seen and are already known to be equivalent, before descending their
//! edges.
//!
//! The mechanism that acheives this is a table that associates nodes by ID with a representation
//! of equivalence classes that is optimized for its needed operations of making a union of
//! classes by merging the paths to it and of checking if nodes are members of the same class by
//! following the paths to it.  (It is not needed to be able to iterate the members of a given
//! class, and the type cannot do that.)
//!
//! For the internal representations, interior mutability is required due to requiring shared
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


/// An equivalence class.
///
/// Distinct values of this type represent distinct equivalence classes.
#[derive(Copy, Clone, Debug)]
struct Class
{
    /// Determines which `Class` a node is merged into when being recorded as equivalent to
    /// another node when both nodes had already been seen separately.
    weight: usize,
}

impl Class
{
    fn default() -> Rc<Cell<Self>>
    {
        Rc::new(Cell::new(Class { weight: 1 }))
    }
}

/// Using [`ptr::eq`] for this makes distinct values of this type represent distinct equivalence
/// classes.
impl PartialEq for Class
{
    fn eq(
        &self,
        other: &Self,
    ) -> bool
    {
        ptr::eq(self, other)
    }
}
impl Eq for Class {}


/// Membership in an equivalence class.
///
/// Optimized for merging with other classes and for checking membership in the same class.
#[derive(Clone)]
enum Membership
{
    End(Rc<Cell<Class>>),
    Chain(Rc<Cell<Self>>),
}

impl Membership
{
    fn default() -> Rc<Cell<Self>>
    {
        Membership::new(Class::default())
    }

    fn new(class: Rc<Cell<Class>>) -> Rc<Cell<Self>>
    {
        Rc::new(Cell::new(Self::End(class)))
    }

    /// This type uses `Cell`, instead of `RefCell`, so that failures are impossible, which
    /// requires the approach of this function because our type cannot be `Copy`.
    fn clone_inner(it: &Rc<Cell<Self>>) -> Self
    {
        let dummy = Self::Chain(Rc::clone(it));
        let inner = it.replace(dummy);
        let result = inner.clone();
        it.set(inner);
        result
    }

    /// Get the equivalence class `it` is a member of currently.
    ///
    /// Follows the chain path to the distinct `Class` value.  Long paths are shortened, to
    /// improve efficiency for subsequent traversals.
    fn class(it: &Rc<Cell<Self>>) -> Rc<Cell<Class>>
    {
        let it_inner = Self::clone_inner(it);
        match it_inner {
            Self::End(class) => class,

            Self::Chain(mut next) => {
                let mut cur = Rc::clone(it);
                loop {
                    let next_inner = Self::clone_inner(&next);
                    match next_inner {
                        Self::End(class) => break class,

                        Self::Chain(next_next) => {
                            cur.set(Self::Chain(Rc::clone(&next_next)));
                            cur = next;
                            next = next_next;
                        },
                    }
                }
            },
        }
    }
}

/// The classes of the nodes that are known to be equivalent, for an invocation of the algorithm.
pub(super) struct EquivClasses<K>
{
    /// Table that associates nodes by ID with their equivalence class.
    map: HashMap<K, Rc<Cell<Membership>>>,
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
        let am = Membership::default();
        let bm = Rc::clone(&am);
        let _ignored1 = self.map.insert(ak.clone(), am);
        let _ignored2 = self.map.insert(bk.clone(), bm);
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
        om: &Rc<Cell<Membership>>,
        k: &K,
    )
    {
        let c = Membership::class(om);
        let m = Membership::new(c);
        let _ignored = self.map.insert(k.clone(), m);
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
        am: &Rc<Cell<Membership>>,
        bm: &Rc<Cell<Membership>>,
    ) -> bool
    {
        let ac = Membership::class(am);
        let bc = Membership::class(bm);

        // Already same class.
        if ac == bc {
            true
        }
        // Merge classes, according to the "weighted union rule" as prescribed by the paper.
        else {
            let (aw, bw) = (ac.get().weight, bc.get().weight);
            let (larger_memb, larger_class, smaller_memb);

            if aw >= bw {
                larger_memb = am;
                larger_class = ac;
                smaller_memb = bm;
            }
            else {
                larger_memb = bm;
                larger_class = bc;
                smaller_memb = am;
            }
            smaller_memb.set(Membership::Chain(Rc::clone(larger_memb)));
            larger_class.set(Class { weight: aw.saturating_add(bw) });
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
            (Some(am), None) => {
                let am = &Rc::clone(am); // To end borrow of `self`.
                self.some_seen(am, bk);
                false
            },
            (None, Some(bm)) => {
                let bm = &Rc::clone(bm); // To end borrow of `self`.
                self.some_seen(bm, ak);
                false
            },
            (Some(am), Some(bm)) => Self::all_seen(am, bm),
        }
    }
}


#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn class_eq()
    {
        {
            #![allow(clippy::eq_op)]
            let r = &Class { weight: 0 };
            assert!(r == r);
        };
        {
            let r1 = &Class { weight: 1 };
            let r2 = r1;
            assert_eq!(r1, r2);
        };
        {
            let r1 = Rc::new(Class { weight: 2 });
            let r2 = Rc::clone(&r1);
            assert_eq!(r1, r2);
        };
        {
            let r1 = Rc::new(Class { weight: 3 });
            let r2 = Rc::new(Class { weight: 3 });
            assert_ne!(r1, r2);
        };
    }

    #[test]
    fn class_of()
    {
        let class1 = Class::default();
        let memb1 = Membership::new(Rc::clone(&class1));
        let memb2 = Membership::new(Rc::clone(&class1));
        let memb3 = Rc::new(Cell::new(Membership::Chain(Rc::clone(&memb1))));
        let memb4 = Rc::new(Cell::new(Membership::Chain(Rc::clone(&memb2))));
        let memb5 = Rc::new(Cell::new(Membership::Chain(Rc::clone(&memb4))));
        let memb6 = Rc::new(Cell::new(Membership::Chain(Rc::clone(&memb5))));

        assert_eq!(Membership::class(&memb1), class1);
        assert_eq!(Membership::class(&memb2), class1);
        assert_eq!(Membership::class(&memb3), class1);
        assert_eq!(Membership::class(&memb4), class1);
        assert_eq!(Membership::class(&memb5), class1);
        assert_eq!(Membership::class(&memb6), class1);

        let class2 = Class::default();
        let memb7 = Membership::new(Rc::clone(&class2));
        assert_ne!(Membership::class(&memb7), class1);
    }

    #[test]
    fn same_class()
    {
        let mut ec = EquivClasses::new();
        let keys = ['a', 'b', 'c', 'd', 'e', 'f'];

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
        assert!(!ec.same_class(&keys[1], &keys[4]));

        for a in &keys {
            for b in &keys {
                assert!(ec.same_class(a, b));
            }
        }
    }
}
