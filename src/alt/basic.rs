use std::{
    cell::{
        Cell,
        Ref,
        RefCell,
    },
    collections::HashMap,
    hash::{
        Hash,
        Hasher,
    },
    ops::Deref,
    ptr,
    rc::Rc,
};


const PRE_LIMIT: u16 = 400;
const FAST_LIMIT: u16 = 2 * PRE_LIMIT;
#[allow(clippy::integer_division)]
const SLOW_LIMIT: u16 = PRE_LIMIT / 10;
#[allow(clippy::as_conversions)]
const SLOW_LIMIT_NEG: i32 = -(SLOW_LIMIT as i32);


#[derive(Debug)]
pub struct Datum<DR>(RefCell<DatumInner<DR>>);

#[derive(Debug)]
pub enum DatumInner<DR>
{
    Leaf,
    Pair(DR, DR),
}

impl<DR> Datum<DR>
{
    #[must_use]
    pub fn leaf() -> Self
    {
        Self(RefCell::new(DatumInner::Leaf))
    }

    #[must_use]
    pub fn pair(
        a: DR,
        b: DR,
    ) -> Self
    {
        Self(RefCell::new(DatumInner::Pair(a, b)))
    }

    fn inner(&self) -> Ref<'_, DatumInner<DR>>
    {
        self.0.borrow()
    }

    pub fn set(
        &self,
        inner: DatumInner<DR>,
    )
    {
        *self.0.borrow_mut() = inner;
    }
}

// TODO?: To have this, would require:
//          Datum<DR>: Clone
//          DatumRef::new(Datum<DR>)
//        Which seems undesirable to require of all users.
// impl<DR> PartialEq for Datum<DR>
// {
//     fn eq(
//         &self,
//         other: &Self,
//     ) -> bool
//     {
//         precheck_interleave_equiv(self, other)
//     }
// }
// impl<DR> Eq for Datum<DR>
// {
// }

impl<DR: Clone> Clone for Datum<DR>
{
    fn clone(&self) -> Self
    {
        Self(RefCell::new(self.inner().clone()))
    }
}

impl<DR: Clone> Clone for DatumInner<DR>
{
    fn clone(&self) -> Self
    {
        match self
        {
            Self::Leaf => Self::Leaf,
            Self::Pair(a, b) => Self::Pair(a.clone(), b.clone()),
        }
    }
}

pub trait DatumRef: Deref<Target = Datum<Self>> + Clone + Sized
{
}

impl<T: Deref<Target = Datum<T>> + Clone> DatumRef for T
{
}


pub fn precheck_interleave_equiv<DR: DatumRef>(
    ar: &DR,
    br: &DR,
) -> bool
{
    precheck(ar, br, PRE_LIMIT.into())
        .map_or(false, |lim| if lim >= 0 { true } else { interleave(ar, br, -1) })
}


fn precheck<DR: DatumRef>(
    ar: &DR,
    br: &DR,
    limit: i32,
) -> Option<i32>
{
    equiv(ar, br, limit, |_, _, lim, _| lim >= 0, |a, b, lim, _| precheck(a, b, lim), &mut ())
}


fn equiv<
    DR: DatumRef,
    D: FnMut(&DR, &DR, i32, &mut S) -> bool,
    R: FnMut(&DR, &DR, i32, &mut S) -> Option<i32>,
    S,
>(
    ar: &DR,
    br: &DR,
    mut limit: i32,
    mut do_descend: D,
    mut recur: R,
    state: &mut S,
) -> Option<i32>
{
    let (ad, bd): (&Datum<DR>, &Datum<DR>) = (&**ar, &**br);

    if ptr::eq(ad, bd)
    {
        Some(limit)
    }
    else
    {
        #[allow(clippy::enum_glob_use)]
        use DatumInner::*;

        let (ai, bi): (&DatumInner<DR>, &DatumInner<DR>) = (&*ar.inner(), &*br.inner());

        match (ai, bi)
        {
            (Leaf, Leaf) => Some(limit),

            (Pair(ap0, ap1), Pair(bp0, bp1)) =>
            {
                limit = limit.saturating_sub(1);

                if do_descend(ar, br, limit, state)
                {
                    recur(ap0, bp0, limit, state).and_then(|lim| recur(ap1, bp1, lim, state))
                }
                else
                {
                    Some(limit)
                }
            },

            (_, _) => None,
        }
    }
}


fn interleave<DR: DatumRef>(
    ar: &DR,
    br: &DR,
    limit: i32,
) -> bool
{
    fn slow_or_fast<DR: DatumRef>(
        ar: &DR,
        br: &DR,
        limit: i32,
        equiv_classes: &mut EquivClasses<DR>,
    ) -> Option<i32>
    {
        if limit < 0
        {
            if limit > SLOW_LIMIT_NEG
            {
                slow(ar, br, limit, equiv_classes)
            }
            else
            {
                fn rand_limit(max: u16) -> i32
                {
                    fastrand::i32(0 ..= max.into())
                }

                fast(ar, br, rand_limit(FAST_LIMIT), equiv_classes)
            }
        }
        else
        {
            fast(ar, br, limit, equiv_classes)
        }
    }

    fn slow<DR: DatumRef>(
        ar: &DR,
        br: &DR,
        limit: i32,
        equiv_classes: &mut EquivClasses<DR>,
    ) -> Option<i32>
    {
        equiv(
            ar,
            br,
            limit,
            |a, b, _, eqv_cls| !eqv_cls.same_class(a, b),
            slow_or_fast,
            equiv_classes,
        )
    }

    fn fast<DR: DatumRef>(
        ar: &DR,
        br: &DR,
        limit: i32,
        equiv_classes: &mut EquivClasses<DR>,
    ) -> Option<i32>
    {
        equiv(ar, br, limit, |_, _, _, _| true, slow_or_fast, equiv_classes)
    }

    slow_or_fast(ar, br, limit, &mut EquivClasses::new()).is_some()
}


struct EquivClasses<DR>
{
    map: HashMap<DatumRefKey<DR>, Rc<Cell<EquivClassChain>>>,
}

struct DatumRefKey<DR>(DR);

impl<DR: DatumRef> PartialEq for DatumRefKey<DR>
{
    fn eq(
        &self,
        other: &Self,
    ) -> bool
    {
        let (ar, br): (&Datum<DR>, &Datum<DR>) = (&*self.0, &*other.0);
        ptr::eq(ar, br)
    }
}
impl<DR: DatumRef> Eq for DatumRefKey<DR>
{
}

impl<DR: DatumRef> Hash for DatumRefKey<DR>
{
    fn hash<H: Hasher>(
        &self,
        state: &mut H,
    )
    {
        let r: &Datum<DR> = &*self.0;
        let ptr: *const Datum<DR> = r;
        ptr.hash(state);
    }
}

impl<DR: Clone> From<&DR> for DatumRefKey<DR>
{
    fn from(r: &DR) -> Self
    {
        Self(r.clone())
    }
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

impl<DR: DatumRef> EquivClasses<DR>
{
    fn new() -> Self
    {
        Self { map: HashMap::new() }
    }

    fn none_seen(
        &mut self,
        ar: &DR,
        br: &DR,
    )
    {
        let (ak, bk): (DatumRefKey<DR>, DatumRefKey<DR>) = (ar.into(), br.into());
        let aec = EquivClassChain::default();
        let bec = Rc::clone(&aec);
        let _ignored1 = self.map.insert(ak, aec);
        let _ignored2 = self.map.insert(bk, bec);
    }

    fn some_seen(
        &mut self,
        oec: &Rc<Cell<EquivClassChain>>,
        r: &DR,
    )
    {
        let rk: DatumRefKey<DR> = r.into();
        let rep = EquivClassChain::rep_of(oec);
        let ec = EquivClassChain::new(rep);
        let _ignored = self.map.insert(rk, ec);
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
        ar: &DR,
        br: &DR,
    ) -> bool
    {
        let (ak, bk): (&DatumRefKey<DR>, &DatumRefKey<DR>) = (&ar.into(), &br.into());

        match (self.map.get(ak), self.map.get(bk))
        {
            #![allow(clippy::shadow_reuse)]
            (None, None) =>
            {
                self.none_seen(ar, br);
                false
            },
            (Some(aec), None) =>
            {
                let aec = &Rc::clone(aec); // To end borrow of `self`.
                self.some_seen(aec, br);
                false
            },
            (None, Some(bec)) =>
            {
                let bec = &Rc::clone(bec); // To end borrow of `self`.
                self.some_seen(bec, ar);
                false
            },
            (Some(aec), Some(bec)) => Self::all_seen(aec, bec),
        }
    }
}


#[cfg(test)]
mod tests
{
    use std::collections::hash_map::DefaultHasher;

    use super::*;

    #[derive(Clone)]
    struct RcDatum(Datum<DatumRc>);
    #[derive(Clone, Eq)]
    struct DatumRc(Rc<RcDatum>);

    impl PartialEq for RcDatum
    {
        fn eq(
            &self,
            other: &Self,
        ) -> bool
        {
            DatumRc::eq(&self.into(), &other.into())
        }
    }
    impl Eq for RcDatum
    {
    }

    impl From<&RcDatum> for DatumRc
    {
        fn from(d: &RcDatum) -> Self
        {
            Self(Rc::new(d.clone()))
        }
    }

    impl PartialEq for DatumRc
    {
        fn eq(
            &self,
            other: &Self,
        ) -> bool
        {
            precheck_interleave_equiv(self, other)
        }
    }

    impl Deref for DatumRc
    {
        type Target = Datum<Self>;

        fn deref(&self) -> &Self::Target
        {
            &(*self.0).0
        }
    }

    fn leaf() -> DatumRc
    {
        DatumRc(Rc::new(RcDatum(Datum::leaf())))
    }

    fn pair(
        a: DatumRc,
        b: DatumRc,
    ) -> DatumRc
    {
        DatumRc(Rc::new(RcDatum(Datum::pair(a, b))))
    }

    fn end_pair() -> DatumRc
    {
        pair(leaf(), leaf())
    }

    #[test]
    fn precheck_basic()
    {
        assert_eq!(precheck(&leaf(), &leaf(), 42), Some(42));
        assert_eq!(precheck(&leaf(), &leaf(), -1), Some(-1));
        assert_eq!(precheck(&leaf(), &end_pair(), 42), None);
        assert_eq!(precheck(&end_pair(), &leaf(), 42), None);
        assert_eq!(precheck(&end_pair(), &end_pair(), 7), Some(6));
        assert_eq!(precheck(&pair(leaf(), end_pair()), &pair(leaf(), end_pair()), 7), Some(5));
        assert_eq!(precheck(&end_pair(), &end_pair(), 0), Some(-1));
        assert_eq!(precheck(&pair(leaf(), end_pair()), &pair(leaf(), end_pair()), 1), Some(-1));
        assert_eq!(
            {
                let x = pair(end_pair(), leaf());
                precheck(&x, &x, 0)
            },
            Some(0)
        );
    }

    #[test]
    fn datum_ref_key()
    {
        fn hash(d: &DatumRefKey<DatumRc>) -> u64
        {
            let mut hasher = DefaultHasher::new();
            d.hash(&mut hasher);
            hasher.finish()
        }

        {
            #![allow(clippy::eq_op)]
            let l = leaf();
            let dr = DatumRefKey(l);
            assert!(dr == dr);
            assert_eq!(hash(&dr), hash(&dr));
        };
        {
            let d1 = leaf();
            let d2 = d1.clone();
            let dr1 = DatumRefKey(d1);
            let dr2 = DatumRefKey(d2);
            assert!(dr1 == dr2);
            assert_eq!(hash(&dr1), hash(&dr2));
        };
        {
            let d1 = leaf();
            let d2 = leaf();
            let dr1 = DatumRefKey(d1);
            let dr2 = DatumRefKey(d2);
            assert!(dr1 != dr2);
        };
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
        fn p() -> DatumRc
        {
            pair(leaf(), leaf())
        }

        fn check_all(
            g: &[DatumRc],
            ec: &mut EquivClasses<DatumRc>,
        )
        {
            for a in g
            {
                for b in g
                {
                    assert!(ec.same_class(a, b));
                }
            }
        }

        let mut ec = EquivClasses::new();
        let g = [p(), p(), p(), p(), p(), p()];

        assert!(!ec.same_class(&g[0], &g[1]));
        assert!(ec.same_class(&g[0], &g[1]));

        assert!(!ec.same_class(&g[0], &g[2]));
        assert!(ec.same_class(&g[0], &g[2]));
        assert!(ec.same_class(&g[1], &g[2]));

        assert!(!ec.same_class(&g[3], &g[2]));
        assert!(ec.same_class(&g[3], &g[2]));
        assert!(ec.same_class(&g[3], &g[1]));
        assert!(ec.same_class(&g[3], &g[0]));

        assert!(!ec.same_class(&g[4], &g[5]));
        assert!(ec.same_class(&g[4], &g[5]));
        assert!(!ec.same_class(&g[1], &g[4]));
        check_all(&g, &mut ec);
    }

    mod precheck_interleave_equiv
    {
        use super::*;
        use crate::tests::{
            self,
            make_degenerate_cycle,
            make_degenerate_dag,
            make_list,
            DEGEN_DAG_TEST_DEPTH,
            LONG_LIST_TEST_LENGTH,
        };

        impl tests::Pair for DatumRc
        {
            fn new(
                a: Self,
                b: Self,
            ) -> Self
            {
                pair(a, b)
            }

            fn set(
                &self,
                a: Self,
                b: Self,
            )
            {
                self.0.0.set(DatumInner::Pair(a, b));
            }
        }

        impl tests::Leaf for DatumRc
        {
            fn new() -> Self
            {
                leaf()
            }
        }

        #[test]
        fn rudimentary()
        {
            assert!(leaf() == leaf());
        }

        #[test]
        fn degenerate_dag_fast()
        {
            let ddag1 = make_degenerate_dag::<DatumRc>(DEGEN_DAG_TEST_DEPTH);
            let ddag2 = make_degenerate_dag::<DatumRc>(DEGEN_DAG_TEST_DEPTH);
            // dbg!(&ddag1);
            assert!(ddag1 == ddag2);
        }

        #[test]
        fn degenerate_cycle_works_and_fast()
        {
            let dcyc1 = make_degenerate_cycle::<DatumRc>(1);
            let dcyc2 = make_degenerate_cycle::<DatumRc>(1);
            assert!(dcyc1 == dcyc2);
        }

        #[test]
        #[ignore]
        fn long_list_stack_overflow()
        {
            let list1 = make_list::<DatumRc>(LONG_LIST_TEST_LENGTH);
            let list2 = make_list::<DatumRc>(LONG_LIST_TEST_LENGTH);
            assert!(list1 == list2);
        }
    }
}
