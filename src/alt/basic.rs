// TODO(#9): Reorganize into sub modules

use std::{
    cell::Cell,
    collections::HashMap,
    hash::Hash,
    ops::AddAssign,
    ptr,
    rc::Rc,
};


// TODO: These values are from the paper, which is for Scheme.  Other values
// might be more optimal for this Rust variation?
const PRE_LIMIT: u16 = 400;
const FAST_LIMIT: u16 = 2 * PRE_LIMIT;
#[allow(clippy::integer_division)]
const SLOW_LIMIT: u16 = PRE_LIMIT / 10;
#[allow(clippy::as_conversions)]
const SLOW_LIMIT_NEG: i32 = -(SLOW_LIMIT as i32);


/// What the main equivalence algorithm needs from a type.
pub trait Node
{
    /// Determines when nodes are the same identical node and so can immediately
    /// be considered equivalent without checking their values, edges, nor
    /// descendents.  The size of and methods on this type should be small and
    /// very cheap.
    ///
    /// For types where only nodes that are the same allocation in memory can be
    /// considered identical, pointer/address equality and hashing should be
    /// used by defining this type to be [`*const Self`].  (Unfortunately,
    /// trying to use `&Self` would result in too many difficulties with
    /// lifetimes.  Using `*const Self` is valid for the equivalence algorithm
    /// because the lifetimes of the `&Self` borrows for the entry-point
    /// function calls outlive the raw pointers used internally, and so the
    /// `Self` objects cannot move during that lifetime and so the pointers
    /// remain valid.)
    ///
    /// For types where different `Self` allocations can represent the same
    /// identical node, a different implementation must be provided.
    type Id: Eq + Hash + Clone;

    type Index: Eq + Ord + AddAssign + From<u8>;

    type Edge: Node<Id = Self::Id>;

    fn id(&self) -> Self::Id;

    fn amount_edges(&self) -> Self::Index;

    fn get_edge(
        &self,
        idx: &Self::Index,
    ) -> Self::Edge;

    /// Check if the nodes are equivalent in their own directly-contained values
    /// ignoring their edges and ignoring their descendent nodes.  This is
    /// intended to be used by
    /// [`Self::equiv_modulo_descendents_then_amount_edges`].
    ///
    /// TODO: This is not the best most precise statement of the issue.
    /// For types that do not directly contain values, i.e. when only the
    /// structure of edges and descendents matters, the implementation should
    /// just return `true`.
    fn equiv_modulo_edges(
        &self,
        other: &Self,
    ) -> bool;

    /// Check if the nodes are equivalent in their own directly-contained values
    /// ignoring their descendent nodes and check if their amounts of edges are
    /// similar enough that their descendents will need to be checked for
    /// equivalence.  If both conditions are true, return the amount of edges
    /// that the main equivalence algorithm should descend, else return `None`.
    ///
    /// The implementation must use [`Self::equiv_modulo_edges`] and
    /// [`Self::amount_edges`] to check the conditions, but it may do so in any
    /// order.  This allows the implementation to optimize the order to be the
    /// most efficient for its type.
    ///
    /// The implementation must ensure that a `Some(result)` upholds:
    /// `self.amount_edges() >= result && other.amount_edges() >= result`, so
    /// that there are enough descendents of each to descend into.
    ///
    /// The default implementation checks that `self.amount_edges() ==
    /// other.amount_edges()` and `self.equiv_modulo_edges(other)`, in that
    /// order, and, when true, returns the amount of edges.  This is intended
    /// for types where [`Self::amount_edges`] is cheaper than
    /// [`Self::equiv_modulo_edges`] and so should be checked first, and where
    /// the nodes should be considered unequivalent if their amounts of edges
    /// are not the same, and where all the edges should be descended.  For
    /// types that do not want all of those aspects, a custom implementation
    /// will need to be provided, and it must fulfill all the above
    /// requirements.
    #[inline]
    fn equiv_modulo_descendents_then_amount_edges(
        &self,
        other: &Self,
    ) -> Option<Self::Index>
    {
        let (az, bz) = (self.amount_edges(), other.amount_edges());
        (az == bz && self.equiv_modulo_edges(other)).then(|| az)
    }
}


/// The main equivalence algorithm which can be used for [`PartialEq`]
/// implementations.  TODO(#10): more about...
pub fn precheck_interleave_equiv<N: Node + ?Sized>(
    a: &N,
    b: &N,
) -> bool
{
    match precheck(a, b, PRE_LIMIT.into())
    {
        EquivResult::Equiv(_) => true,
        EquivResult::Unequiv => false,
        EquivResult::Abort => interleave(a, b, -1),
    }
}


fn precheck<N: Node + ?Sized>(
    a: &N,
    b: &N,
    limit: i32,
) -> EquivResult
{
    equiv(
        a,
        b,
        limit,
        |_, _, lim, _| if lim > 0 { DoDescend::Yes } else { DoDescend::NoAbort },
        |a, b, lim, _| precheck(a, b, lim),
        &mut (),
    )
}


enum DoDescend
{
    Yes,
    NoContinue(i32),
    NoAbort,
}

#[derive(PartialEq, Eq, Debug)]
enum EquivResult
{
    Unequiv,
    Abort,
    Equiv(i32),
}

// TODO(#1): Might be cleaner to instead have a trait for this with methods instead
// of passing the closures, and the different uses can impl that for their own
// internal types.
fn equiv<
    N: Node + ?Sized,
    D: FnMut(&N, &N, i32, &mut S) -> DoDescend,
    R: FnMut(&N::Edge, &N::Edge, i32, &mut S) -> EquivResult,
    S,
>(
    a: &N,
    b: &N,
    mut limit: i32,
    mut do_descend: D,
    mut recur: R,
    state: &mut S,
) -> EquivResult
{
    use EquivResult::{
        Abort,
        Equiv,
        Unequiv,
    };

    if a.id() == b.id()
    {
        Equiv(limit)
    }
    else if let Some(amount_edges) = a.equiv_modulo_descendents_then_amount_edges(b)
    {
        let mut i = 0.into();

        if i < amount_edges
        {
            match do_descend(a, b, limit, state)
            {
                DoDescend::Yes =>
                {
                    while i < amount_edges
                    {
                        let (ae, be) = (a.get_edge(&i), b.get_edge(&i));

                        limit = limit.saturating_sub(1);

                        match recur(&ae, &be, limit, state)
                        {
                            Equiv(lim) => limit = lim,
                            result @ (Unequiv | Abort) => return result,
                        }

                        i += 1.into();
                    }
                },
                DoDescend::NoContinue(lim) => limit = lim,
                DoDescend::NoAbort => return Abort,
            }
        }

        Equiv(limit)
    }
    else
    {
        Unequiv
    }
}


// TODO(#2): Could this be refactored to be cleaner. Hopefully refactoring equiv
// per above will help do that for this.
fn interleave<N: Node + ?Sized>(
    a: &N,
    b: &N,
    limit: i32,
) -> bool
{
    fn slow_or_fast<N: Node + ?Sized>(
        a: &N,
        b: &N,
        limit: i32,
        equiv_classes: &mut EquivClasses<N::Id>,
    ) -> EquivResult
    {
        if limit < 0
        {
            if limit >= SLOW_LIMIT_NEG
            {
                slow(a, b, limit, equiv_classes)
            }
            else
            {
                fn rand_limit(max: u16) -> i32
                {
                    fastrand::i32(0 ..= max.into())
                }

                fast(a, b, rand_limit(FAST_LIMIT), equiv_classes)
            }
        }
        else
        {
            fast(a, b, limit, equiv_classes)
        }
    }

    fn slow<N: Node + ?Sized>(
        a: &N,
        b: &N,
        limit: i32,
        equiv_classes: &mut EquivClasses<N::Id>,
    ) -> EquivResult
    {
        equiv(
            a,
            b,
            limit,
            |a, b, _, eqv_cls| {
                if eqv_cls.same_class(&a.id(), &b.id())
                {
                    // This is what prevents traversing descendents that have
                    // already been checked, which prevents infinite loops on
                    // cycles and is more efficient on shared structure.  Reset
                    // the counter so that `slow` will be used for longer, on
                    // the theory that further equivalences in descendents are
                    // more likely since we found an equivalence (which is
                    // critical for avoiding stack overflow with shapes like
                    // "degenerate cyclic".).
                    DoDescend::NoContinue(-1)
                }
                else
                {
                    DoDescend::Yes
                }
            },
            slow_or_fast,
            equiv_classes,
        )
    }

    fn fast<N: Node + ?Sized>(
        a: &N,
        b: &N,
        limit: i32,
        equiv_classes: &mut EquivClasses<N::Id>,
    ) -> EquivResult
    {
        equiv(a, b, limit, |_, _, _, _| DoDescend::Yes, slow_or_fast, equiv_classes)
    }

    matches!(slow_or_fast(a, b, limit, &mut EquivClasses::new()), EquivResult::Equiv(_))
}


struct EquivClasses<Key>
{
    map: HashMap<Key, Rc<Cell<EquivClassChain>>>,
}

#[derive(Clone)]
enum EquivClassChain
{
    End(Rc<Cell<Representative>>),
    Next(Rc<Cell<Self>>),
}

#[derive(Copy, Clone, Debug)]
struct Representative
{
    weight: usize,
}

impl Representative
{
    fn default() -> Rc<Cell<Self>>
    {
        Rc::new(Cell::new(Self { weight: 1 }))
    }
}

impl PartialEq for Representative
{
    fn eq(
        &self,
        other: &Self,
    ) -> bool
    {
        ptr::eq(self, other)
    }
}
impl Eq for Representative
{
}

impl EquivClassChain
{
    fn default() -> Rc<Cell<Self>>
    {
        Self::new(Representative::default())
    }

    fn new(rep: Rc<Cell<Representative>>) -> Rc<Cell<Self>>
    {
        Rc::new(Cell::new(Self::End(rep)))
    }

    /// This type uses `Cell`, instead of `RefCell`, so that panics are
    /// impossible, which requires the approach of this function because our
    /// type cannot be `Copy`.
    fn clone_inner(it: &Rc<Cell<Self>>) -> Self
    {
        let dummy = Self::Next(Rc::clone(it));
        let inner = it.replace(dummy);
        let result = inner.clone();
        it.set(inner);
        result
    }

    fn rep_of(it: &Rc<Cell<Self>>) -> Rc<Cell<Representative>>
    {
        let it_inner = Self::clone_inner(it);
        match it_inner
        {
            Self::End(rep) => rep,

            Self::Next(mut next) =>
            {
                let mut cur = Rc::clone(it);
                loop
                {
                    let next_inner = Self::clone_inner(&next);
                    match next_inner
                    {
                        Self::End(rep) => break rep,

                        Self::Next(next_next) =>
                        {
                            cur.set(Self::Next(Rc::clone(&next_next)));
                            cur = next;
                            next = next_next;
                        },
                    }
                }
            },
        }
    }
}

impl<K: Eq + Hash + Clone> EquivClasses<K>
{
    fn new() -> Self
    {
        Self { map: HashMap::new() }
    }

    fn none_seen(
        &mut self,
        ak: &K,
        bk: &K,
    )
    {
        let aec = EquivClassChain::default();
        let bec = Rc::clone(&aec);
        let _ignored1 = self.map.insert(ak.clone(), aec);
        let _ignored2 = self.map.insert(bk.clone(), bec);
    }

    fn some_seen(
        &mut self,
        oec: &Rc<Cell<EquivClassChain>>,
        k: &K,
    )
    {
        let rep = EquivClassChain::rep_of(oec);
        let ec = EquivClassChain::new(rep);
        let _ignored = self.map.insert(k.clone(), ec);
    }

    fn all_seen(
        aec: &Rc<Cell<EquivClassChain>>,
        bec: &Rc<Cell<EquivClassChain>>,
    ) -> bool
    {
        let arep = EquivClassChain::rep_of(aec);
        let brep = EquivClassChain::rep_of(bec);

        if arep == brep
        {
            true
        }
        else
        {
            let (aw, bw) = (arep.get().weight, brep.get().weight);
            let (larger_chain, larger_rep, smaller_chain);

            if aw >= bw
            {
                larger_chain = aec;
                larger_rep = arep;
                smaller_chain = bec;
            }
            else
            {
                larger_chain = bec;
                larger_rep = brep;
                smaller_chain = aec;
            }
            smaller_chain.set(EquivClassChain::Next(Rc::clone(larger_chain)));
            larger_rep.set(Representative { weight: aw.saturating_add(bw) });
            false
        }
    }

    fn same_class(
        &mut self,
        ak: &K,
        bk: &K,
    ) -> bool
    {
        match (self.map.get(ak), self.map.get(bk))
        {
            (None, None) =>
            {
                self.none_seen(ak, bk);
                false
            },
            (Some(aec), None) =>
            {
                let aec = &Rc::clone(aec); // To end borrow of `self`.
                self.some_seen(aec, bk);
                false
            },
            (None, Some(bec)) =>
            {
                let bec = &Rc::clone(bec); // To end borrow of `self`.
                self.some_seen(bec, ak);
                false
            },
            (Some(aec), Some(bec)) => Self::all_seen(aec, bec),
        }
    }
}


#[cfg(test)]
mod tests
{
    use super::*;

    enum Datum
    {
        Leaf,
        Pair(Box<Self>, Box<Self>),
    }

    fn leaf() -> Datum
    {
        Datum::Leaf
    }

    fn pair(
        a: Datum,
        b: Datum,
    ) -> Datum
    {
        Datum::Pair(Box::new(a), Box::new(b))
    }

    fn end_pair() -> Datum
    {
        pair(leaf(), leaf())
    }

    impl Node for &Datum
    {
        type Edge = Self;
        type Id = *const Datum;
        type Index = u8;

        fn id(&self) -> Self::Id
        {
            *self
        }

        fn amount_edges(&self) -> Self::Index
        {
            match self
            {
                Datum::Leaf => 0,
                Datum::Pair(_, _) => 2,
            }
        }

        fn get_edge(
            &self,
            idx: &Self::Index,
        ) -> Self::Edge
        {
            match (idx, self)
            {
                #![allow(clippy::panic)]
                (0, Datum::Pair(a, _)) => a,
                (1, Datum::Pair(_, b)) => b,
                _ => panic!("invalid"),
            }
        }

        fn equiv_modulo_edges(
            &self,
            _other: &Self,
        ) -> bool
        {
            true
        }
    }

    #[test]
    fn precheck_basic()
    {
        use EquivResult::{
            Abort,
            Equiv,
            Unequiv,
        };

        assert_eq!(precheck(&&leaf(), &&leaf(), 42), Equiv(42));
        assert_eq!(precheck(&&leaf(), &&leaf(), -1), Equiv(-1));
        assert_eq!(precheck(&&leaf(), &&end_pair(), 42), Unequiv);
        assert_eq!(precheck(&&end_pair(), &&leaf(), 42), Unequiv);
        assert_eq!(precheck(&&end_pair(), &&end_pair(), 7), Equiv(5));
        assert_eq!(precheck(&&pair(leaf(), end_pair()), &&pair(leaf(), end_pair()), 7), Equiv(3));
        assert_eq!(precheck(&&end_pair(), &&end_pair(), 0), Abort);
        assert_eq!(precheck(&&pair(leaf(), end_pair()), &&pair(leaf(), end_pair()), 1), Abort);
        assert_eq!(
            {
                let x = pair(end_pair(), leaf());
                precheck(&&x, &&x, 0)
            },
            Equiv(0)
        );
    }

    #[test]
    fn representative()
    {
        {
            #![allow(clippy::eq_op)]
            let r = &Representative { weight: 0 };
            assert!(r == r);
        };
        {
            let r1 = &Representative { weight: 1 };
            let r2 = r1;
            assert_eq!(r1, r2);
        };
        {
            let r1 = Rc::new(Representative { weight: 2 });
            let r2 = Rc::clone(&r1);
            assert_eq!(r1, r2);
        };
        {
            let r1 = Rc::new(Representative { weight: 3 });
            let r2 = Rc::new(Representative { weight: 3 });
            assert_ne!(r1, r2);
        };
    }

    #[test]
    fn rep_of()
    {
        let rep1 = Representative::default();
        let ecc1 = EquivClassChain::new(Rc::clone(&rep1));
        let ecc2 = EquivClassChain::new(Rc::clone(&rep1));
        let ecc3 = Rc::new(Cell::new(EquivClassChain::Next(Rc::clone(&ecc1))));
        let ecc4 = Rc::new(Cell::new(EquivClassChain::Next(Rc::clone(&ecc2))));
        let ecc5 = Rc::new(Cell::new(EquivClassChain::Next(Rc::clone(&ecc4))));
        let ecc6 = Rc::new(Cell::new(EquivClassChain::Next(Rc::clone(&ecc5))));

        assert_eq!(EquivClassChain::rep_of(&ecc1), rep1);
        assert_eq!(EquivClassChain::rep_of(&ecc2), rep1);
        assert_eq!(EquivClassChain::rep_of(&ecc3), rep1);
        assert_eq!(EquivClassChain::rep_of(&ecc4), rep1);
        assert_eq!(EquivClassChain::rep_of(&ecc5), rep1);
        assert_eq!(EquivClassChain::rep_of(&ecc6), rep1);

        let rep2 = Representative::default();
        let ecc7 = EquivClassChain::new(Rc::clone(&rep2));
        assert_ne!(EquivClassChain::rep_of(&ecc7), rep1);
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

        for a in &keys
        {
            for b in &keys
            {
                assert!(ec.same_class(a, b));
            }
        }
    }
}
