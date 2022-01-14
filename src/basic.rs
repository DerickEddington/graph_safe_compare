pub use premade::*;

mod premade
{
    use {
        super::{
            modes::{
                limited::{
                    LimitReached,
                    Limited,
                    Ticker,
                },
                unlimited::Unlimited,
            },
            recursion::callstack::CallStack,
        },
        crate::{
            generic::equiv::{
                self,
                Equiv,
            },
            utils::IntoOk as _,
            Node,
        },
        core::{
            convert::Infallible,
            marker::PhantomData,
        },
    };


    /// Equivalence predicate that cannot handle cyclic nor very-deep graphs, and that has the
    /// least overhead.
    ///
    /// This is similar to `#[derive(PartialEq)]` and so usually is not better than that, but
    /// could be useful for types where implementing [`Node`] in peculiar ways is useful for
    /// having different behavior than derived `PartialEq`.
    #[inline]
    pub fn equiv<N: Node>(
        a: &N,
        b: &N,
    ) -> N::Cmp
    {
        struct Args<N>(PhantomData<N>);

        impl<N: Node> equiv::Params for Args<N>
        {
            type DescendMode = Unlimited;
            type Error = Infallible;
            type Node = N;
            type RecurStack = CallStack;
        }

        let mut e = Equiv::<Args<N>>::default();
        e.equiv(a, b).into_ok()
    }


    /// Equivalence predicate that limits how many nodes are traversed, and that aborts early if
    /// the limit is reached.  Like [`equiv`](equiv()), this cannot handle cyclic nor very-deep
    /// graphs and has minimal overhead.
    ///
    /// # Errors
    /// If the limit is reached before completing, return `Err(LimitReached)`.
    #[inline]
    pub fn limited_equiv<N: Node, L: Ticker>(
        limit: L,
        a: &N,
        b: &N,
    ) -> Result<N::Cmp, LimitReached>
    {
        struct Args<N, L>(PhantomData<(N, L)>);

        impl<N: Node, L: Ticker> equiv::Params for Args<N, L>
        {
            type DescendMode = Limited<L>;
            type Error = LimitReached;
            type Node = N;
            type RecurStack = CallStack;
        }

        let mut e = Equiv::<Args<N, L>>::new(Limited(limit));
        e.equiv(a, b)
    }
}


/// Basic support, that cannot handle very-deep graphs, for recursion of the algorithm.
pub mod recursion
{
    /// Use the normal call-stack for the recursions done to descend node edges.
    pub mod callstack
    {
        use crate::{
            generic::equiv::{
                self,
                EdgesIter,
                Equiv,
                RecurStack,
            },
            Cmp,
            Node,
        };

        /// Specifies use of the normal call-stack.
        ///
        /// Does depth-first preorder traversals.
        #[derive(Default)]
        #[non_exhaustive]
        pub struct CallStack;

        /// Enables [`CallStack`] to be used with the algorithm.
        impl<P> RecurStack<P> for CallStack
        where P: equiv::Params<RecurStack = Self>
        {
            type Error = P::Error;

            #[inline]
            fn recur(
                it: &mut Equiv<P>,
                edges_iter: EdgesIter<P::Node>,
            ) -> Result<<P::Node as Node>::Cmp, Self::Error>
            {
                for (a, b) in edges_iter {
                    match it.equiv_main(&a, &b) {
                        Ok(cmp) if cmp.is_equiv() => (),
                        result => return result,
                    }
                }
                Ok(Cmp::new_equiv())
            }

            #[inline]
            fn next(&mut self) -> Option<(P::Node, P::Node)>
            {
                None
            }

            /// Only for compatibility with generic uses of recursion-stack types.  The call-stack
            /// does not need to be reset, so this is a no-op for this type.
            #[inline]
            fn reset(self) -> Self
            {
                self
            }
        }
    }
}

/// Modes of the algorithm that are useful in basic ways.
pub mod modes
{
    /// Do not limit the algorithm in how many nodes are traversed, and never abort early.
    pub mod unlimited
    {
        use {
            crate::generic::equiv::{
                self,
                DescendMode,
            },
            core::convert::Infallible,
        };

        /// Specifies not limiting the amount of nodes traversed.
        #[derive(Default)]
        #[allow(clippy::exhaustive_structs)]
        pub struct Unlimited;

        /// Enables [`Unlimited`] to be used with the algorithm.
        impl<P> DescendMode<P> for Unlimited
        where
            P: equiv::Params<DescendMode = Self>,
            Infallible: Into<P::Error>,
        {
            type Error = Infallible;

            /// Always start handling node edges.
            #[inline]
            fn do_edges(
                &mut self,
                _a: &P::Node,
                _b: &P::Node,
            ) -> Result<bool, Self::Error>
            {
                Ok(true)
            }

            /// Always traverse nodes, without limit.
            #[inline]
            fn do_traverse(&mut self) -> Result<bool, Self::Error>
            {
                Ok(true)
            }
        }
    }

    /// Limit the algorithm in how many nodes it is allowed to traverse before aborting early.
    pub mod limited
    {
        mod sealed
        {
            use core::ops::SubAssign;

            /// Any type of unsigned integer may be used with [`Limited`](super::Limited).
            pub trait Ticker: Ord + From<u8> + SubAssign {}
            impl Ticker for u8 {}
            impl Ticker for u16 {}
            impl Ticker for u32 {}
            impl Ticker for u64 {}
            impl Ticker for u128 {}
            impl Ticker for usize {}
        }

        pub(in super::super) use sealed::Ticker;

        use crate::generic::equiv::{
            self,
            DescendMode,
        };

        /// Specifies limiting the amount of nodes traversed.  The inner value is the limit.
        #[allow(clippy::exhaustive_structs)]
        pub struct Limited<T>(pub T);

        /// [`Err`] type returned when aborting early because a limit was reached.
        #[derive(Debug)]
        #[allow(clippy::exhaustive_structs)]
        pub struct LimitReached;

        /// Enables [`Limited`] to be used with the algorithm.
        impl<T, P> DescendMode<P> for Limited<T>
        where
            T: Ticker,
            P: equiv::Params<DescendMode = Self>,
            LimitReached: Into<P::Error>,
        {
            type Error = LimitReached;

            /// Always start handling node edges.
            #[inline]
            fn do_edges(
                &mut self,
                _a: &P::Node,
                _b: &P::Node,
            ) -> Result<bool, Self::Error>
            {
                Ok(true)
            }

            /// Enforce the limit on the amount of nodes traversed.
            #[inline]
            fn do_traverse(&mut self) -> Result<bool, Self::Error>
            {
                if self.0 > 0.into() {
                    self.0 -= 1.into();
                    Ok(true)
                }
                else {
                    Err(LimitReached)
                }
            }
        }
    }
}


#[cfg(test)]
#[allow(clippy::enum_glob_use)]
mod tests
{
    extern crate alloc;

    use {
        super::{
            modes::limited::{
                LimitReached,
                Limited,
            },
            recursion::callstack::CallStack,
        },
        crate::{
            generic::equiv::{
                self,
                Equiv,
            },
            Node,
        },
        alloc::boxed::Box,
        core::marker::PhantomData,
    };

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
        type Cmp = bool;
        type Id = *const Datum;
        type Index = u8;

        fn id(&self) -> Self::Id
        {
            *self
        }

        fn amount_edges(&self) -> Self::Index
        {
            match self {
                Datum::Leaf => 0,
                Datum::Pair(_, _) => 2,
            }
        }

        fn get_edge(
            &self,
            idx: &Self::Index,
        ) -> Self
        {
            match (idx, self) {
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
    fn limiting()
    {
        #[derive(PartialEq, Eq, Debug)]
        enum ResultLimit
        {
            True(u32),
            False(u32),
            Abort(u32),
        }

        fn eqv(
            a: &Datum,
            b: &Datum,
            limit: u32,
        ) -> ResultLimit
        {
            struct LimitedArgs<'l>(PhantomData<&'l ()>);

            impl<'l> equiv::Params for LimitedArgs<'l>
            {
                type DescendMode = Limited<u32>;
                type Error = LimitReached;
                type Node = &'l Datum;
                type RecurStack = CallStack;
            }

            let mut e = Equiv::<LimitedArgs<'_>>::new(Limited(limit));

            match e.equiv(a, b) {
                Ok(true) => True(e.descend_mode.0),
                Ok(false) => False(e.descend_mode.0),
                Err(LimitReached) => Abort(e.descend_mode.0),
            }
        }

        use ResultLimit::*;

        assert_eq!(eqv(&leaf(), &leaf(), 42), True(41));
        assert_eq!(eqv(&leaf(), &leaf(), 1), True(0));
        assert_eq!(eqv(&leaf(), &leaf(), 0), Abort(0));
        assert_eq!(eqv(&leaf(), &end_pair(), 42), False(41));
        assert_eq!(eqv(&end_pair(), &leaf(), 42), False(41));
        assert_eq!(eqv(&end_pair(), &end_pair(), 7), True(4));
        assert_eq!(eqv(&pair(leaf(), end_pair()), &pair(leaf(), end_pair()), 7), True(2));
        assert_eq!(eqv(&end_pair(), &end_pair(), 0), Abort(0));
        assert_eq!(eqv(&end_pair(), &end_pair(), 2), Abort(0));
        assert_eq!(eqv(&pair(leaf(), end_pair()), &pair(leaf(), end_pair()), 1), Abort(0));
        assert_eq!(eqv(&pair(leaf(), leaf()), &pair(leaf(), end_pair()), 42), False(39));
        assert_eq!(
            {
                let x = pair(end_pair(), leaf());
                eqv(&x, &x, 1)
            },
            True(0)
        );
    }
}
