pub use premade::*;

mod premade
{
    use {
        super::{
            modes::{
                limited::{
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
                Aborted,
                Equiv,
            },
            Node,
        },
        core::marker::PhantomData,
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
    ) -> bool
    {
        struct Args<N>(PhantomData<N>);

        impl<N: Node> equiv::Params for Args<N>
        {
            type DescendMode = Unlimited;
            type Node = N;
            type RecurStack = CallStack;
        }

        let mut e = Equiv::<Args<N>>::default();
        e.is_equiv(a, b)
    }


    /// Equivalence predicate that limits how many node edges are descended, and that aborts early
    /// if the limit is reached.  Like [`equiv`](equiv()), this cannot handle cyclic nor very-deep
    /// graphs and has minimal overhead.
    ///
    /// # Errors
    /// If the limit is reached before completing, return `Err(Aborted)`.
    #[inline]
    pub fn limited_equiv<N: Node, L: Ticker>(
        limit: L,
        a: &N,
        b: &N,
    ) -> Result<bool, Aborted>
    {
        struct Args<N, L>(PhantomData<(N, L)>);

        impl<N: Node, L: Ticker> equiv::Params for Args<N, L>
        {
            type DescendMode = Limited<L>;
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
        use crate::generic::equiv::{
            self,
            Aborted,
            Equiv,
            RecurStack,
        };

        /// Specifies use of the normal call-stack.
        #[derive(Default)]
        #[non_exhaustive]
        pub struct CallStack;

        /// Enables [`CallStack`] to be used with the algorithm.
        impl<P> RecurStack<P> for CallStack
        where P: equiv::Params<RecurStack = Self>
        {
            #[inline]
            fn recur(
                it: &mut Equiv<P>,
                a: P::Node,
                b: P::Node,
            ) -> Result<bool, Aborted>
            {
                it.equiv_main(&a, &b)
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
    /// Do not limit the algorithm in how many node edges are descended, and never abort early.
    pub mod unlimited
    {
        use crate::generic::equiv::{
            self,
            DescendMode,
        };

        /// Specifies not limiting the amount of node edges descended.
        #[derive(Default)]
        #[allow(clippy::exhaustive_structs)]
        pub struct Unlimited;

        /// Enables [`Unlimited`] to be used with the algorithm.
        impl<P> DescendMode<P> for Unlimited
        where P: equiv::Params<DescendMode = Self>
        {
            /// Always start handling node edges.
            #[inline]
            fn do_edges(
                &mut self,
                _a: &P::Node,
                _b: &P::Node,
            ) -> bool
            {
                true
            }

            /// Always descend into edges, without limit.
            #[inline]
            fn do_recur(&mut self) -> bool
            {
                true
            }
        }
    }

    /// Limit the algorithm in how many node edges it is allowed to descend before aborting early.
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
        }

        pub(in super::super) use sealed::Ticker;
        use {
            crate::generic::equiv::{
                self,
                DescendMode,
                Equiv,
            },
            core::ops::ControlFlow,
        };

        /// Specifies limiting the amount of node edges descended.  The inner value is the limit.
        #[allow(clippy::exhaustive_structs)]
        pub struct Limited<T>(pub T);

        impl<T, P> Equiv<P>
        where P: equiv::Params<DescendMode = Limited<T>>
        {
            /// Intended for uses where early abort due to reaching the limit should cause control
            /// to continue on to some other attempt.
            #[inline]
            pub fn precheck_equiv(
                &mut self,
                a: &P::Node,
                b: &P::Node,
            ) -> ControlFlow<bool, ()>
            {
                self.equiv(a, b).map_or(ControlFlow::Continue(()), ControlFlow::Break)
            }
        }

        /// Enables [`Limited`] to be used with the algorithm.
        impl<T, P> DescendMode<P> for Limited<T>
        where
            T: Ticker,
            P: equiv::Params<DescendMode = Self>,
        {
            /// Always start handling node edges.
            #[inline]
            fn do_edges(
                &mut self,
                _a: &P::Node,
                _b: &P::Node,
            ) -> bool
            {
                true
            }

            /// Enforce the limit on the amount of edges descended into.
            #[inline]
            fn do_recur(&mut self) -> bool
            {
                if self.0 > 0.into() {
                    self.0 -= 1.into();
                    true
                }
                else {
                    false
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
            modes::limited::Limited,
            recursion::callstack::CallStack,
        },
        crate::{
            generic::equiv::{
                self,
                Aborted,
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
                type Node = &'l Datum;
                type RecurStack = CallStack;
            }

            let mut e = Equiv::<LimitedArgs<'_>>::new(Limited(limit));

            match e.equiv(a, b) {
                Ok(true) => True(e.descend_mode.0),
                Ok(false) => False(e.descend_mode.0),
                Err(Aborted) => Abort(e.descend_mode.0),
            }
        }

        use ResultLimit::*;

        assert_eq!(eqv(&leaf(), &leaf(), 42), True(42));
        assert_eq!(eqv(&leaf(), &leaf(), 0), True(0));
        assert_eq!(eqv(&leaf(), &end_pair(), 42), False(42));
        assert_eq!(eqv(&end_pair(), &leaf(), 42), False(42));
        assert_eq!(eqv(&end_pair(), &end_pair(), 7), True(5));
        assert_eq!(eqv(&pair(leaf(), end_pair()), &pair(leaf(), end_pair()), 7), True(3));
        assert_eq!(eqv(&end_pair(), &end_pair(), 0), Abort(0));
        assert_eq!(eqv(&end_pair(), &end_pair(), 1), Abort(0));
        assert_eq!(eqv(&pair(leaf(), end_pair()), &pair(leaf(), end_pair()), 1), Abort(0));
        assert_eq!(eqv(&pair(leaf(), leaf()), &pair(leaf(), end_pair()), 42), False(40));
        assert_eq!(
            {
                let x = pair(end_pair(), leaf());
                eqv(&x, &x, 0)
            },
            True(0)
        );
    }
}
