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
            utils::IntoOk as _,
            Node,
        },
        core::{
            convert::Infallible,
            marker::PhantomData,
        },
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
    ) -> N::Cmp
    {
        impl<N: Node> equiv::Params for Args<N>
        {
            type DescendMode = Interleave<Self>;
            type Error = Infallible;
            type Node = N;
            type RecurStack = CallStack;
        }

        let mut e = Equiv::<Args<N>>::default();
        e.equiv(a, b).into_ok()
    }


    /// Like [`equiv`](equiv()) but first tries the precheck that is faster for small acyclic
    /// graphs.
    #[inline]
    pub fn precheck_equiv<N: Node>(
        a: &N,
        b: &N,
    ) -> N::Cmp
    {
        impl<N: Node> precheck_interleave::Params<N> for Args<N>
        {
            type Error = Infallible;
            type InterleaveParams = Self;
            type InterleaveRecurStack = CallStack;
            type PrecheckRecurStack = CallStack;
        }

        precheck_interleave::equiv::<N, Args<N>>(a, b).into_ok()
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
            core::{
                convert::Infallible,
                num::NonZeroU16,
            },
            random::NumberGenerator as _,
        };


        /// Generic parameters of [`Interleave`] and its operations.
        ///
        /// The default values for the associated constants are from the paper, which is for
        /// Scheme.  You may choose different values for your particular application, by `impl`ing
        /// [`Params`] for your own type and using it with the [`generic`](crate::generic) API.
        pub trait Params
        {
            /// How many nodes are traversed by a separate precheck before it aborts.
            ///
            /// Only directly used by a separate precheck when [`Interleave`] is used after that
            /// (e.g. [`precheck_interleave::equiv`](crate::generic::precheck_interleave::equiv)).
            /// Also used to derive the default values of the other constants, following the
            /// paper.  Included here in [`Params`] because it often makes sense for the other
            /// constants to be defined in terms of it.  You may redefine only this and the others
            /// will be derived from that, or you may redefine the others.
            const PRECHECK_LIMIT: u16 = 400;
            /// Maximum of randomized limiting of how many nodes are traversed by the "fast" phase
            /// before switching to the "slow" phase.
            const FAST_LIMIT_MAX: u16 = 2 * Self::PRECHECK_LIMIT;
            /// How many nodes, consecutively, that have not already been seen, are traversed by
            /// the "slow" phase before switching to the "fast" phase.
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
            /// Decremented for every node traversed, and reset when the phase is changed.
            ticker:        i32,
            /// Table of nodes that have already been seen and recorded as equivalent, for use by
            /// the "slow" phase.
            equiv_classes: EquivClasses<P::Table>,
            /// State of the (P)RNG that is used to vary the limit of the "fast" phase.
            rng:           P::RNG,
        }

        #[allow(clippy::as_conversions, clippy::integer_division)]
        impl<P: Params> Interleave<P>
        {
            /// How many additional leaf nodes may be traversed, after the "slow" phase limit was
            /// exceeded, and still reset to "fast" phase.  If this is reached, reset to "slow"
            /// phase instead, because fast traversal of leaf nodes was already just done for a
            /// period.
            pub const BELOW_SLOW_LIMIT: i32 =
                Self::SLOW_LIMIT_NEG - ((P::FAST_LIMIT_MAX / 4) as i32);
            /// Exclusive end of range, derived from
            /// [`P::FAST_LIMIT_MAX`](Params::FAST_LIMIT_MAX).
            pub const FAST_LIMIT_MAX_RANGE_END: NonZeroU16 =
                match NonZeroU16::new(P::FAST_LIMIT_MAX + 1) {
                    Some(v) => v,
                    #[allow(clippy::panic)]
                    None => panic!(),
                };
            /// Negated [`P::SLOW_LIMIT`](Params::SLOW_LIMIT).
            pub const SLOW_LIMIT_NEG: i32 = -(P::SLOW_LIMIT as i32);
        }

        impl<P: Params> Default for Interleave<P>
        {
            #[inline]
            fn default() -> Self
            {
                Self {
                    ticker:        0,
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
            Infallible: Into<E::Error>,
        {
            type Error = Infallible;

            /// Determine whether to use "slow" or "fast" phase, based on our limits.  When "slow"
            /// phase, if the nodes are already known to be equivalent then do not check their
            /// descendents.
            #[inline]
            fn do_edges(
                &mut self,
                a: &E::Node,
                b: &E::Node,
            ) -> Result<bool, Self::Error>
            {
                // "fast" phase
                let r = if self.ticker >= 0 {
                    true
                }
                // "slow" limit reached, change to "fast" phase
                else if self.ticker < Self::SLOW_LIMIT_NEG
                // When `ticker <= BELOW_SLOW_LIMIT`, that means leaf nodes (possibly many) were
                // traversed without calling this method, which was fast, and so "slow" phase
                // should now be done (this is critical for balancing performance for both
                // very-wide degenerate and very-long narrow leafy shapes).
                    && Self::BELOW_SLOW_LIMIT < self.ticker
                {
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
                    self.ticker = 0;
                    false
                }
                else {
                    true
                };
                Ok(r)
            }

            /// Decrement the ticker before [`Self::do_edges`] is called, and traverse nodes
            /// without limit.
            #[inline]
            fn do_traverse(&mut self) -> Result<bool, Self::Error>
            {
                self.ticker = self.ticker.saturating_sub(1);
                Ok(true)
            }
        }
    }
}
