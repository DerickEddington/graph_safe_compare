#[cfg(feature = "std")]
pub use premade::*;

#[cfg(feature = "std")]
mod premade
{
    use {
        super::modes::interleave::Interleave,
        crate::{
            basic::recursion::callstack::CallStack,
            generic::{
                equiv::Equiv,
                equiv_classes::premade::HashMap,
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
        let mut e = Equiv::<Interleave<HashMap<N>>, _>::new(CallStack);
        e.is_equiv(a, b)
    }


    /// Like [`equiv`] but first tries the precheck that is faster for small acyclic graphs.
    #[inline]
    pub fn precheck_equiv<N: Node>(
        a: &N,
        b: &N,
    ) -> bool
    {
        precheck_interleave_equiv::<N, HashMap<N>, CallStack, CallStack>(a, b)
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
        mod random;

        use {
            crate::{
                basic::modes::limited::Limited,
                generic::{
                    equiv::{
                        Descend,
                        Equiv,
                    },
                    equiv_classes::{
                        EquivClasses,
                        Table,
                    },
                    recursion::Reset,
                },
                Node,
            },
            core::num::NonZeroU16,
            random::chosen::RandomNumberGenerator,
        };

        // TODO: These values are from the paper, which is for Scheme.  Other values might be more
        // optimal for this Rust variation?
        pub(crate) const PRE_LIMIT: u16 = 400;
        pub(crate) const FAST_LIMIT_RANGE_END: NonZeroU16 =
            match NonZeroU16::new(2 * PRE_LIMIT + 1) {
                Some(v) => v,
                #[allow(clippy::panic)]
                None => panic!(),
            };
        #[allow(clippy::integer_division)]
        pub(crate) const SLOW_LIMIT: u16 = PRE_LIMIT / 10;
        #[allow(clippy::as_conversions)]
        pub(crate) const SLOW_LIMIT_NEG: i32 = -(SLOW_LIMIT as i32);

        /// Specifies use of the "interleave" mode.
        ///
        /// The chosen `T` type must implement [`Table`].
        pub struct Interleave<T>
        {
            /// Table of nodes that have already been seen and recorded as equivalent, for use by
            /// the "slow" phase.
            equiv_classes: EquivClasses<T>,
            /// State of the (P)RNG that is used to vary the limit of the "fast" phase.
            rng:           RandomNumberGenerator,
        }

        impl<T: Default, S: Reset> Equiv<Interleave<T>, S>
        {
            /// Create a new state for an invocation of the [`Interleave`] mode of the algorithm.
            ///
            /// The given `recur_stack` type determines how the algorithm will do its recursions.
            #[inline]
            pub fn new(recur_stack: S) -> Self
            {
                Self {
                    ticker:      -1,
                    mode:        Interleave {
                        equiv_classes: EquivClasses::new(),
                        rng:           RandomNumberGenerator::default(),
                    },
                    recur_stack: recur_stack.reset(),
                }
            }
        }

        /// Enables the same recursion-stack value to be reused across the precheck and the
        /// interleave, which is more efficient for some types since this avoids dropping it and
        /// creating another.
        impl<SP, SI, N, T> From<Equiv<Limited<N>, SP>> for Equiv<Interleave<T>, SI>
        where
            SP: Into<SI>,
            SI: Reset,
            T: Default,
        {
            #[inline]
            fn from(prechecker: Equiv<Limited<N>, SP>) -> Self
            {
                Self::new(prechecker.recur_stack.into())
            }
        }

        /// Enables [`Interleave`] to be used with the algorithm.
        impl<T: Table, S> Descend for Equiv<Interleave<T>, S>
        {
            type Node = T::Node;

            /// Determine whether to use "slow" or "fast" phase, based on our limits.  When "slow"
            /// phase, if the nodes are already known to be equivalent then do not check their
            /// descendents.
            #[inline]
            fn do_edges(
                &mut self,
                a: &Self::Node,
                b: &Self::Node,
            ) -> bool
            {
                match self.ticker {
                    // "fast" phase
                    0 .. => true,

                    // "slow" phase
                    SLOW_LIMIT_NEG ..= -1 =>
                        if self.mode.equiv_classes.same_class(&a.id(), &b.id()) {
                            // This is what prevents traversing descendents that have already been
                            // checked, which prevents infinite loops on cycles and is more
                            // efficient on shared structure.

                            // Reset the ticker so that "slow" will be used for longer, "on the
                            // theory that if one equivalence is found, more are likely to be
                            // found" (which is critical for avoiding stack overflow with shapes
                            // like "degenerate cyclic").
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

                        self.ticker = random::NumberGenerator::rand_upto(
                            // Call the trait method as a direct function call (instead of method
                            // call) so that the `RandomNumberGenerator` type (chosen by package
                            // feature) is required to `impl` the trait (instead of working if it
                            // accidentally has only an inherent implementation of a method with
                            // the same signature).
                            &mut self.mode.rng,
                            FAST_LIMIT_RANGE_END,
                        )
                        .into();

                        true
                    },
                }
            }

            /// When [`Self::do_edges`] returns `true`, descend into edges without limit.
            #[inline]
            fn do_recur(&mut self) -> bool
            {
                true
            }
        }
    }
}
