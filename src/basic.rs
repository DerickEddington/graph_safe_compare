use {
    self::{
        modes::{
            limited::Limited,
            unlimited::Unlimited,
        },
        recursion::callstack::CallStack,
    },
    crate::{
        generic::equiv::{
            Aborted,
            Equiv,
        },
        Node,
    },
};


/// Equivalence predicate that cannot handle cyclic nor very-deep graphs, and that has the least
/// overhead.
///
/// This is similar to `#[derive(PartialEq)]` and so usually is not better than that, but could be
/// useful for types where implementing [`Node`] in peculiar ways is useful for having different
/// behavior than derived `PartialEq`.
#[inline]
pub fn equiv<N: Node>(
    a: &N,
    b: &N,
) -> bool
{
    let mut e = Equiv::<Unlimited<N>, _>::new(CallStack);
    e.is_equiv(a, b)
}


// TODO: The `limit` should not be `i32`, and instead should be unsigned and generic, which will
// require changes to `Equiv`.
/// Equivalence predicate that limits how many node edges are descended, and that aborts early if
/// the limit is reached.  Like [`equiv`], this cannot handle cyclic nor very-deep graphs and has
/// minimal overhead.
///
/// # Errors
/// If the limit is reached before completing, return `Err(Aborted)`.
#[inline]
pub fn limited_equiv<N: Node>(
    limit: i32,
    a: &N,
    b: &N,
) -> Result<bool, Aborted>
{
    let mut e = Equiv::<Limited<N>, _>::new(limit, CallStack);
    e.equiv(a, b)
}


/// Basic support, that cannot handle very-deep graphs, for recursion of the algorithm.
pub mod recursion
{
    /// Use the normal call-stack for the recursions done to descend node edges.
    pub mod callstack
    {
        use crate::{
            generic::{
                equiv::{
                    Aborted,
                    Descend,
                    Equiv,
                    Recur,
                },
                recursion::Reset,
            },
            Node,
        };

        /// Specifies use of the normal call-stack.
        #[derive(Default)]
        #[non_exhaustive]
        pub struct CallStack;

        /// Only for compatibility with generic uses of recursion-stack types.  The call-stack
        /// does not need to be reset, so this is a no-op for this type.
        impl Reset for CallStack
        {
            #[inline]
            fn reset(self) -> Self
            {
                self
            }
        }

        /// Enables [`CallStack`] to be used with the algorithm.
        impl<N: Node, M> Recur<CallStack> for Equiv<M, CallStack>
        where Self: Descend<Node = N>
        {
            type Node = N;

            #[inline]
            fn recur(
                &mut self,
                a: Self::Node,
                b: Self::Node,
            ) -> Result<bool, Aborted>
            {
                self.equiv_main(&a, &b)
            }

            #[inline]
            fn next(&mut self) -> Option<(Self::Node, Self::Node)>
            {
                None
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
        use {
            crate::{
                generic::equiv::{
                    Descend,
                    Equiv,
                },
                Node,
            },
            core::marker::PhantomData,
        };

        /// Specifies not limiting the amount of node edges descended.
        pub struct Unlimited<N>
        {
            /// Some `impl`s need `N` to be constrained by `Self`.
            _node_type: PhantomData<N>,
        }

        impl<N, S> Equiv<Unlimited<N>, S>
        {
            /// Create a new state for an invocation of the [`Unlimited`] mode of the algorithm.
            ///
            /// The given `recur_stack` type determines how the algorithm will do its recursions,
            /// and the value must be new or [`Reset`](crate::generic::recursion::Reset).
            #[inline]
            pub fn new(recur_stack: S) -> Self
            {
                Self { ticker: 0, mode: Unlimited { _node_type: PhantomData }, recur_stack }
            }
        }

        impl<N, S: Default> Default for Equiv<Unlimited<N>, S>
        {
            #[inline]
            fn default() -> Self
            {
                Self::new(S::default())
            }
        }

        /// Enables [`Unlimited`] to be used with the algorithm.
        impl<N: Node, S> Descend for Equiv<Unlimited<N>, S>
        {
            type Node = N;

            /// Always start handling node edges.
            #[inline]
            fn do_edges(
                &mut self,
                _a: &Self::Node,
                _b: &Self::Node,
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
        use {
            crate::{
                generic::equiv::{
                    Descend,
                    Equiv,
                    Recur,
                },
                Node,
            },
            core::{
                marker::PhantomData,
                ops::ControlFlow,
            },
        };

        /// Specifies limiting the amount of node edges descended.
        pub struct Limited<N>
        {
            /// Some `impl`s need `N` to be constrained by `Self`.
            _node_type: PhantomData<N>,
        }

        impl<N, S> Equiv<Limited<N>, S>
        {
            /// Create a new state for an invocation of the [`Limited`] mode of the algorithm.
            ///
            /// The given `recur_stack` type determines how the algorithm will do its recursions,
            /// and the value must be new or [`Reset`](crate::generic::recursion::Reset).
            #[inline]
            pub fn new(
                limit: i32,
                recur_stack: S,
            ) -> Self
            {
                Self { ticker: limit, mode: Limited { _node_type: PhantomData }, recur_stack }
            }
        }

        impl<N: Node, S> Equiv<Limited<N>, S>
        where Self: Descend<Node = N> + Recur<S, Node = N>
        {
            /// Intended for uses where early abort due to reaching the limit should cause control
            /// to continue on to some other attempt.
            #[inline]
            pub fn precheck_equiv(
                &mut self,
                a: &N,
                b: &N,
            ) -> ControlFlow<bool, ()>
            {
                self.equiv(a, b).map_or(ControlFlow::Continue(()), ControlFlow::Break)
            }
        }

        /// Enables [`Limited`] to be used with the algorithm.
        impl<N: Node, S> Descend for Equiv<Limited<N>, S>
        {
            type Node = N;

            /// Always start handling node edges.
            #[inline]
            fn do_edges(
                &mut self,
                _a: &Self::Node,
                _b: &Self::Node,
            ) -> bool
            {
                true
            }

            /// Enforce the limit on the amount of edges descended into.
            #[inline]
            fn do_recur(&mut self) -> bool
            {
                self.ticker >= 0
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
                Aborted,
                Equiv,
            },
            Node,
        },
        alloc::boxed::Box,
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
            True(i32),
            False(i32),
            Abort(i32),
        }

        fn eqv(
            a: &Datum,
            b: &Datum,
            limit: i32,
        ) -> ResultLimit
        {
            let mut e = Equiv::<Limited<&Datum>, CallStack>::new(limit, CallStack);

            match e.equiv(a, b) {
                Ok(true) => True(e.ticker),
                Ok(false) => False(e.ticker),
                Err(Aborted) => Abort(e.ticker),
            }
        }

        use ResultLimit::*;

        assert_eq!(eqv(&leaf(), &leaf(), 42), True(42));
        assert_eq!(eqv(&leaf(), &leaf(), -1), True(-1));
        assert_eq!(eqv(&leaf(), &end_pair(), 42), False(42));
        assert_eq!(eqv(&end_pair(), &leaf(), 42), False(42));
        assert_eq!(eqv(&end_pair(), &end_pair(), 7), True(5));
        assert_eq!(eqv(&pair(leaf(), end_pair()), &pair(leaf(), end_pair()), 7), True(3));
        assert_eq!(eqv(&end_pair(), &end_pair(), 0), Abort(-1));
        assert_eq!(eqv(&end_pair(), &end_pair(), 1), Abort(-1));
        assert_eq!(eqv(&pair(leaf(), end_pair()), &pair(leaf(), end_pair()), 1), Abort(-1));
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
