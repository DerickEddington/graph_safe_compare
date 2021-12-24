#[cfg(feature = "std")]
pub use premade::*;

#[cfg(feature = "std")]
mod premade
{
    use {
        super::modes::interleave::{
            self,
            random::default,
            Interleave,
        },
        crate::{
            basic::recursion::callstack::CallStack,
            generic::{
                equiv::{
                    self,
                    Equiv,
                },
                equiv_classes::premade::hash_map,
                precheck_interleave,
            },
            Node,
        },
        core::marker::PhantomData,
    };

    struct Args<N>(PhantomData<N>);

    impl<N: Node> interleave::Params for Args<N>
    {
        type Node = N;
        type RNG = default::RandomNumberGenerator;
        type Table = hash_map::Table<Self>;
    }

    impl<N: Node> hash_map::Params for Args<N>
    {
        type Node = N;
    }

    /// Equivalence predicate that can handle cyclic graphs but not very-deep graphs.
    #[inline]
    pub fn equiv<N: Node>(
        a: &N,
        b: &N,
    ) -> bool
    {
        impl<N: Node> equiv::Params for Args<N>
        {
            type DescendMode = Interleave<Self>;
            type Node = N;
            type RecurStack = CallStack;
        }

        let mut e = Equiv::<Args<N>>::default();
        e.is_equiv(a, b)
    }


    /// Like [`equiv`](equiv()) but first tries the precheck that is faster for small acyclic
    /// graphs.
    #[inline]
    pub fn precheck_equiv<N: Node>(
        a: &N,
        b: &N,
    ) -> bool
    {
        impl<N: Node> precheck_interleave::Params<N> for Args<N>
        {
            type InterleaveParams = Self;
            type InterleaveRecurStack = CallStack;
            type PrecheckRecurStack = CallStack;
        }

        precheck_interleave::equiv::<N, Args<N>>(a, b)
    }
}


/// Modes of the algorithm that enable handling cyclic and degenerate graphs.
pub mod modes
{
    /// Make the algorithm interleave a shared-structure-detecting "slow" phase with a basic
    /// "fast" phase.
    pub mod interleave
    {
        /// The "interleave" mode requires (pseudo)random numbers, to randomly vary the limit of
        /// the "fast" phase.
        pub mod random;

        use {
            crate::{
                generic::{
                    equiv::{
                        self,
                        DescendMode,
                    },
                    equiv_classes::{
                        EquivClasses,
                        Table,
                    },
                },
                Node,
            },
            core::num::NonZeroU16,
            random::NumberGenerator as _,
        };


        /// Generic parameters of [`Interleave`] and its operations.
        ///
        /// The default values for the associated constants are from the paper, which is for
        /// Scheme.  You may choose different values for your particular application, by `impl`ing
        /// [`Params`] for your own type and using it with the [`generic`](crate::generic) API.
        pub trait Params
        {
            /// How many edges are descended by a separate precheck before it aborts.
            ///
            /// Only directly used by a separate precheck when [`Interleave`] is used after that
            /// (e.g. [`precheck_interleave::equiv`](crate::generic::precheck_interleave::equiv)).
            /// Also used to derive the default values of the other constants, following the
            /// paper.  Included here in [`Params`] because it often makes sense for the other
            /// constants to be defined in terms of it.  You may redefine only this and the others
            /// will be derived from that, or you may redefine the others.
            const PRECHECK_LIMIT: u16 = 400;
            /// Maximum of randomized limiting of how many edges are descended by the "fast" phase
            /// before switching to the "slow" phase.
            const FAST_LIMIT_MAX: u16 = 2 * Self::PRECHECK_LIMIT;
            /// How many node edges, consecutively, that have not already been seen, are descended
            /// by the "slow" phase before switching to the "fast" phase.
            #[allow(clippy::integer_division)]
            const SLOW_LIMIT: u16 = Self::PRECHECK_LIMIT / 10;

            /// Type of node that is recorded as equivalent in the [`Self::Table`].  Must be the
            /// same as used with the corresponding [`equiv::Params`].
            type Node: Node;
            /// Type that records nodes as equivalent.
            type Table: Table<Node = Self::Node>;
            /// Type that provides a sequence of (pseudo)random numbers, used to vary the limit of
            /// the "fast" phase.
            type RNG: random::NumberGenerator;
        }

        /// Specifies use of the "interleave" mode.
        pub struct Interleave<P: Params>
        {
            /// Decremented for every node edge descended into, and reset when the phase is
            /// changed.
            ticker:        i32,
            /// Table of nodes that have already been seen and recorded as equivalent, for use by
            /// the "slow" phase.
            equiv_classes: EquivClasses<P::Table>,
            /// State of the (P)RNG that is used to vary the limit of the "fast" phase.
            rng:           P::RNG,
        }

        impl<P: Params> Interleave<P>
        {
            const FAST_LIMIT_MAX_RANGE_END: NonZeroU16 =
                match NonZeroU16::new(P::FAST_LIMIT_MAX + 1) {
                    Some(v) => v,
                    #[allow(clippy::panic)]
                    None => panic!(),
                };
            #[allow(clippy::as_conversions)]
            const SLOW_LIMIT_NEG: i32 = -(P::SLOW_LIMIT as i32);
        }

        impl<P: Params> Default for Interleave<P>
        {
            #[inline]
            fn default() -> Self
            {
                Self {
                    ticker:        -1,
                    equiv_classes: EquivClasses::default(),
                    rng:           P::RNG::default(),
                }
            }
        }

        /// Enables [`Interleave`] to be used with the algorithm.
        impl<E, I, T> DescendMode<E> for Interleave<I>
        where
            E: equiv::Params<DescendMode = Self>,
            I: Params<Table = T>,
            T: Table<Node = E::Node>,
        {
            /// Determine whether to use "slow" or "fast" phase, based on our limits.  When "slow"
            /// phase, if the nodes are already known to be equivalent then do not check their
            /// descendents.
            #[inline]
            fn do_edges(
                &mut self,
                a: &E::Node,
                b: &E::Node,
            ) -> bool
            {
                // "fast" phase
                if self.ticker >= 0 {
                    true
                }
                // "slow" limit reached, change to "fast" phase
                else if self.ticker < Self::SLOW_LIMIT_NEG {
                    // Random limits for "fast" "reduce the likelihood of repeatedly tripping on
                    // worst-case behavior in cases where the sizes of the input graphs happen to
                    // be related to the chosen bounds in a bad way".
                    self.ticker = self.rng.rand_upto(Self::FAST_LIMIT_MAX_RANGE_END).into();
                    true
                }
                // "slow" phase
                else if self.equiv_classes.same_class(&a.id(), &b.id()) {
                    // This is what prevents traversing descendents that have already been
                    // checked, which prevents infinite loops on cycles and is more efficient on
                    // shared structure.
                    // Reset the ticker so that "slow" will be used for longer, "on the theory
                    // that if one equivalence is found, more are likely to be found" (which is
                    // critical for avoiding stack overflow with shapes like "degenerate cyclic").
                    self.ticker = -1;
                    false
                }
                else {
                    true
                }
            }

            /// When [`Self::do_edges`] returns `true`, descend into edges without limit.
            #[inline]
            fn do_recur(&mut self) -> bool
            {
                self.ticker = self.ticker.saturating_sub(1);
                true
            }
        }
    }
}
