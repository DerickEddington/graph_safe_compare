use {
    self::modes::interleave::Interleave,
    crate::{
        basic::recursion::callstack::CallStack,
        generic::{
            equiv::Equiv,
            precheck_interleave_equiv,
        },
        Node,
    },
};


/// Equivalence predicate that can handle cyclic graphs but not very-deep graphs.
#[inline]
pub fn equiv<N: Node>(
    a: &N,
    b: &N,
) -> bool
{
    let mut e = Equiv::<Interleave<N>, _>::new(CallStack);
    e.is_equiv(a, b)
}


/// Like [`equiv`] but first tries the precheck that is faster for small acyclic graphs.
#[inline]
pub fn precheck_equiv<N: Node>(
    a: &N,
    b: &N,
) -> bool
{
    precheck_interleave_equiv::<N, CallStack, CallStack>(a, b)
}


pub(super) mod modes
{
    /// Make the algorithm interleave a shared-structure-detecting "slow" phase with a basic
    /// "fast" phase.
    pub(crate) mod interleave
    {
        use crate::{
            basic::modes::limited::Limited,
            equiv_classes::EquivClasses,
            generic::{
                equiv::{
                    Descend,
                    Equiv,
                },
                recursion::Reset,
            },
            Node,
        };

        // TODO: These values are from the paper, which is for Scheme.  Other values might be more
        // optimal for this Rust variation?
        pub(crate) const PRE_LIMIT: u16 = 400;
        pub(crate) const FAST_LIMIT: u16 = 2 * PRE_LIMIT;
        #[allow(clippy::integer_division)]
        pub(crate) const SLOW_LIMIT: u16 = PRE_LIMIT / 10;
        #[allow(clippy::as_conversions)]
        pub(crate) const SLOW_LIMIT_NEG: i32 = -(SLOW_LIMIT as i32);

        /// Specifies use of the "interleave" mode.
        pub struct Interleave<N: Node>
        {
            /// Table of nodes that have already been seen and recorded as equivalent, for use by
            /// the "slow" phase.
            equiv_classes: EquivClasses<N::Id>,
        }

        impl<N: Node, S> Equiv<Interleave<N>, S>
        {
            #[inline]
            pub fn new(recur_stack: S) -> Self
            {
                Self {
                    ticker: -1,
                    mode: Interleave { equiv_classes: EquivClasses::new() },
                    recur_stack,
                }
            }
        }

        /// Enables the same recursion-stack value to be reused across the precheck and the
        /// interleave, which is more efficient for some types since this avoids dropping it and
        /// creating another.
        impl<N, SP, SI> From<Equiv<Limited<N>, SP>> for Equiv<Interleave<N>, SI>
        where
            N: Node,
            SP: Reset + Into<SI>,
        {
            fn from(prechecker: Equiv<Limited<N>, SP>) -> Self
            {
                Self::new(prechecker.recur_stack.reset().into())
            }
        }

        /// Enables [`Interleave`] to be used with the algorithm.
        impl<N: Node, S> Descend for Equiv<Interleave<N>, S>
        {
            type Node = N;

            /// Determine whether to use "slow" or "fast" phase, based on our limits.  When "slow"
            /// phase, if the nodes are already known to be equivalent then do not check their
            /// descendents.
            fn do_edges(
                &mut self,
                a: &Self::Node,
                b: &Self::Node,
            ) -> bool
            {
                fn rand(max: u16) -> i32
                {
                    fastrand::i32(0 ..= max.into())
                }

                match self.ticker {
                    // "fast" phase
                    0 .. => true,

                    // "slow" phase
                    SLOW_LIMIT_NEG ..= -1 =>
                        if self.mode.equiv_classes.same_class(&a.id(), &b.id()) {
                            // This is what prevents traversing descendents that have already been
                            // checked, which prevents infinite loops on cycles and is more
                            // efficient on shared structure.  Reset the ticker so that "slow"
                            // will be used for longer, "on the theory that if one equivalence is
                            // found, more are likely to be found" (which is critical for avoiding
                            // stack overflow with shapes like "degenerate cyclic").
                            self.ticker = -1;
                            false
                        }
                        else {
                            true
                        },

                    // "slow" limit reached, change to "fast" phase
                    _ => {
                        // Random limits for "fast" "reduce the likelihood of repeatedly tripping
                        // on worst-case behavior in cases where the sizes of the input graphs
                        // happen to be related to the chosen bounds in a bad way".
                        self.ticker = rand(FAST_LIMIT);
                        true
                    },
                }
            }

            /// When [`Self::do_edges`] returns `true`, descend into edges without limit.
            fn do_recur(&mut self) -> bool
            {
                true
            }
        }
    }
}
