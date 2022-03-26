pub use premade::*;

pub mod equiv_classes;


mod premade
{
    /// The primary variation of the algorithm, like that of the paper.
    pub mod precheck_interleave
    {
        #[allow(unreachable_pub)]
        mod sealed
        {
            use {
                super::{
                    super::super::equiv,
                    errors::{
                        InterleaveError,
                        PrecheckError,
                    },
                    Params,
                },
                crate::{
                    basic::modes::limited::Limited,
                    cycle_safe::modes::interleave::Interleave,
                    Node,
                },
                core::marker::PhantomData,
            };

            pub struct PrecheckArgs<N, P>(PhantomData<(N, P)>);

            impl<N: Node, P: Params<N>> equiv::Params for PrecheckArgs<N, P>
            {
                type DescendMode = Limited<u16>;
                type Error = PrecheckError<P::Error>;
                type Node = N;
                type RecurMode = P::PrecheckRecurMode;
            }

            pub struct InterleaveArgs<N, P>(PhantomData<(N, P)>);

            impl<N: Node, P: Params<N>> equiv::Params for InterleaveArgs<N, P>
            {
                type DescendMode = Interleave<P::InterleaveParams>;
                type Error = InterleaveError<P::Error>;
                type Node = N;
                type RecurMode = P::InterleaveRecurMode;
            }
        }

        mod errors
        {
            #[cfg(doc)]
            use super::super::precheck_interleave;
            use crate::{
                anticipated_or_like::Infallible,
                basic::modes::limited::LimitReached,
            };

            /// Variants of errors that can occur while doing a precheck.
            #[allow(clippy::exhaustive_enums)]
            pub enum PrecheckError<E>
            {
                /// [`LimitReached`] occurred.  Abort the precheck.
                LimitReached,
                /// The [`precheck_interleave::Params::PrecheckRecurMode`] errored.
                RecurError(E),
            }

            impl<E> From<LimitReached> for PrecheckError<E>
            {
                #[inline]
                fn from(_: LimitReached) -> Self
                {
                    PrecheckError::LimitReached
                }
            }

            impl<E> From<Infallible> for PrecheckError<E>
            {
                #[inline]
                fn from(_: Infallible) -> Self
                {
                    #![allow(clippy::unreachable)] // Truly unreachable.
                    unreachable!()
                }
            }

            /// The [`precheck_interleave::Params::InterleaveRecurMode`] errored while doing an
            /// interleave.
            #[allow(clippy::exhaustive_structs)]
            pub struct InterleaveError<E>(pub E);

            impl<E> From<Infallible> for InterleaveError<E>
            {
                #[inline]
                fn from(_: Infallible) -> Self
                {
                    #![allow(clippy::unreachable)] // Truly unreachable.
                    unreachable!()
                }
            }
        }

        pub use errors::{
            InterleaveError,
            PrecheckError,
        };
        use {
            super::super::equiv::{
                Equiv,
                RecurMode,
            },
            crate::{
                basic::modes::limited::Limited,
                cycle_safe::modes::interleave,
                Node,
            },
            sealed::{
                InterleaveArgs,
                PrecheckArgs,
            },
        };


        /// Generic parameters of [`equiv`].
        pub trait Params<N: Node>: Sized
        {
            /// Type of recursion mode for the precheck.
            type PrecheckRecurMode: RecurMode<PrecheckArgs<N, Self>>
                + Into<Self::InterleaveRecurMode>;
            /// Type of recursion mode for the interleave.
            type InterleaveRecurMode: RecurMode<InterleaveArgs<N, Self>>;
            /// Type that `impl`s the arguments for the generic parameters for the interleave.
            type InterleaveParams: interleave::Params<Node = N>;
            /// Type that represents the errors that can occur from [`Self::PrecheckRecurMode`]
            /// and [`Self::InterleaveRecurMode`].
            type Error;
        }

        /// Equivalence predicate that can handle cyclic graphs, but first tries the precheck that
        /// is faster for small acyclic graphs, and that requires choosing specific type arguments
        /// that determine the implementations of internal dynamic data structures.  Safe for
        /// very-deep graphs only when the interleave recursion-mode type is.
        ///
        /// # Errors
        /// If the [`P::PrecheckRecurMode`](Params::PrecheckRecurMode) or
        /// [`P::InterleaveRecurMode`](Params::InterleaveRecurMode) error, return an `Err` with
        /// a [`P::Error`](Params::Error) that represents the error.
        #[inline]
        pub fn equiv<N, P>(
            a: N,
            b: N,
        ) -> Result<N::Cmp, P::Error>
        where
            N: Node + Clone,
            P: Params<N>,
        {
            use interleave::Params as _;

            let mut e =
                Equiv::<PrecheckArgs<N, P>>::new(Limited(P::InterleaveParams::PRECHECK_LIMIT));

            match e.equiv(a.clone(), b.clone()) {
                Ok(cmp) => Ok(cmp),
                Err(PrecheckError::RecurError(e)) => Err(e),
                Err(PrecheckError::LimitReached) => {
                    let mut e: Equiv<InterleaveArgs<N, P>> = e.into();
                    e.equiv(a, b).map_err(|InterleaveError(error)| error)
                },
            }
        }
    }
}


mod modes
{
    use super::equiv::Params;

    /// Controls if node edges are descended into.
    pub trait DescendMode<P: Params>
    {
        /// Type of error that can occur.
        type Error: Into<P::Error>;

        /// Controls if all the descendents of a pair of nodes being compared should be
        /// immediately skipped.
        ///
        /// Returning `true` causes all the descendents to begin to be compared, individually
        /// under the control of [`Self::do_traverse`].
        ///
        /// Returning `false` causes all the descendents to be skipped, and assumes they are
        /// already known to satisfy equivalence with their counterparts, and causes the
        /// comparison traversal to immediately continue on to the next non-descendent
        /// (i.e. sibling or ancestor) nodes.
        ///
        /// # Errors
        /// Returning `Err` causes the invocation of the algorithm to abort early and immediately
        /// return the converted error.
        fn do_edges(
            &mut self,
            a: &P::Node,
            b: &P::Node,
        ) -> Result<bool, Self::Error>;

        /// Controls if each node-counterparts will be traversed.
        ///
        /// Returning `true` causes the next counterparts to be compared.
        ///
        /// Returning `false` causes them to be skipped, and assumes they are already known to
        /// satisfy equivalence, and causes the comparison traversal to immediately continue on to
        /// the next nodes.
        ///
        /// # Errors
        /// Returning `Err` causes the invocation of the algorithm to abort early and immediately
        /// return the converted error.
        fn do_traverse(&mut self) -> Result<bool, Self::Error>;
    }
}


mod recursion
{
    use {
        super::equiv::{
            EdgesIter,
            Equiv,
            Params,
        },
        crate::Node,
    };

    /// [`Node`]s at the same position in the input graphs to compare.
    pub type Counterparts<N> = [N; 2];
    /// `Ok` when there are [`Counterparts`] descendents from ancestors.  `Err` when ancestors do
    /// not have the same amount of edges.
    pub type CounterpartsResult<N> = Result<Counterparts<N>, <N as Node>::Cmp>;

    /// Abstraction of recursion continuations.
    pub trait RecurMode<P: Params>: Default
    {
        /// Type of error that can occur.
        type Error: Into<P::Error>;

        /// Arrange for the given nodes to be recurred on, either immediately or later.
        ///
        /// The `it` parameter enables accessing the entire [`Equiv`] value.
        ///
        /// When recurred on immediately, the result must be that of comparing the given nodes
        /// (`Ok`) or attempting to (`Err`).  When saved for later, the result must be `Ok(cmp)`
        /// where `cmp.is_equiv()` is true, and [`Self::next`] must supply these nodes at some
        /// point, or the result must be `Err` if an error occurred.
        ///
        /// Returning `Ok(cmp)` where `cmp.is_equiv()` is false causes the invocation of the
        /// algorithm to immediately return `cmp` that represents inequivalence.
        ///
        /// # Errors
        /// Returning `Err` causes the invocation of the algorithm to abort early and immediately
        /// return the converted error.
        fn recur(
            it: &mut Equiv<P>,
            edges_iter: EdgesIter<P::Node>,
        ) -> Result<<P::Node as Node>::Cmp, Self::Error>;

        /// Supply the next counterpart nodes for the algorithm to compare, if any were saved for
        /// later by [`Self::recur`].
        ///
        /// # Errors
        /// If there are not the same amount of counterparts (i.e. their ancestor nodes have
        /// different amounts of edges), then `Err(cmp)` indicates which ancestor has less.
        fn next(&mut self) -> Option<CounterpartsResult<P::Node>>;

        /// Reset to be empty while preserving capacity, if relevant.
        ///
        /// An aborted precheck, that uses particular types of recursion-modes, might leave some
        /// elements on such a structure, in which case it needs to be reset before doing the
        /// interleave using the same structure.
        ///
        /// Also, some things that take ownership of a `Self` might call this to ensure a
        /// structure is in a fresh state.
        ///
        /// When it is more efficient, the `self` value should be reset and then returned.  But a
        /// newly-created value may be returned if desired.
        #[must_use]
        fn reset(self) -> Self;
    }
}


/// The central parts of the algorithm.
pub mod equiv
{
    use {
        crate::{
            Cmp,
            Node,
            Step,
        },
        cfg_if::cfg_if,
        core::{
            cmp::Ordering,
            iter,
        },
    };

    pub use super::{
        modes::DescendMode,
        recursion::{
            Counterparts,
            CounterpartsResult,
            RecurMode,
        },
    };

    /// Generic parameters of [`Equiv`] and its operations.
    pub trait Params: Sized
    {
        /// Type of node to handle.
        type Node: Node;
        /// Type that controls descending node edges.
        type DescendMode: DescendMode<Self>;
        /// Type that provides recursion continuations.
        type RecurMode: RecurMode<Self>;
        /// Type that represents the errors that can occur from [`Self::DescendMode`] and
        /// [`Self::RecurMode`].
        type Error;
    }

    /// The state for an invocation of a variation of the algorithm.
    #[non_exhaustive]
    pub struct Equiv<P: Params>
    {
        /// Controls if node edges are descended into.
        pub(crate) descend_mode: P::DescendMode,
        /// Representation of recursion continuations.
        pub recur_mode:          P::RecurMode,
    }

    impl<P: Params> Equiv<P>
    {
        /// Create a new instance to use once for a single invocation of the algorithm.
        ///
        /// For use with [`DescendMode`] types that cannot implement `Default`.
        #[inline]
        pub fn new(descend_mode: P::DescendMode) -> Self
        {
            Self { descend_mode, recur_mode: P::RecurMode::default() }
        }
    }

    impl<P: Params> Default for Equiv<P>
    where P::DescendMode: Default
    {
        /// Create a new instance to use once for a single invocation of the algorithm.
        ///
        /// For use with [`DescendMode`] types that implement `Default`.
        #[inline]
        fn default() -> Self
        {
            Self {
                descend_mode: P::DescendMode::default(),
                recur_mode:   P::RecurMode::default(),
            }
        }
    }

    /// Enables the same recursion-mode value to be reused across the precheck and the interleave,
    /// which is more efficient for some types since this avoids dropping it and creating another.
    ///
    /// [`From`] or [`Into`] cannot be `impl`ed for this, because that would conflict with the
    /// blanket implementations provided by the `core` library.
    impl<PT: Params> Equiv<PT>
    where PT::DescendMode: Default
    {
        /// Like [`From::from`].
        #[inline]
        pub fn from<PF: Params>(e: Equiv<PF>) -> Self
        where PF::RecurMode: Into<PT::RecurMode>
        {
            Self {
                descend_mode: PT::DescendMode::default(),
                recur_mode:   e.recur_mode.reset().into(),
            }
        }
    }

    impl<PF: Params> Equiv<PF>
    {
        /// Like [`Into::into`].
        #[inline]
        pub fn into<PT: Params>(self) -> Equiv<PT>
        where
            PF::RecurMode: Into<PT::RecurMode>,
            PT::DescendMode: Default,
        {
            Equiv::from(self)
        }
    }

    /// The primary logic of the algorithm.
    ///
    /// This generic design works with the [`Node`], [`DescendMode`], and [`RecurMode`] traits to
    /// enable variations.
    impl<P: Params> Equiv<P>
    {
        /// The entry-point of the algorithm.
        ///
        /// Returns `Ok` with a value that represents the result of comparison, according to the
        /// trait implementations that define the variation of the logic.
        ///
        /// # Errors
        /// If a [`DescendMode`] or [`RecurMode`] method gives an error, returns that error
        /// converted.
        #[inline]
        pub fn equiv(
            &mut self,
            mut a: P::Node,
            mut b: P::Node,
        ) -> Result<<P::Node as Node>::Cmp, P::Error>
        {
            // This loop, when used in conjunction with certain `RecurMode::recur` and
            // `RecurMode::next` implementations, is what prevents growing the call-stack, and so
            // prevents the possibility of stack overflow, when traversing descendents.  For other
            // implementations where the `RecurMode::recur` does grow the call-stack, the
            // `RecurMode::next` always returns `None` and so this loop should be optimized away.
            loop {
                match self.equiv_main(a, b) {
                    Ok(cmp) if cmp.is_equiv() => match self.recur_mode.next() {
                        Some(Ok([an, bn])) => {
                            a = an;
                            b = bn;
                        },
                        Some(Err(cmp_amount_edges)) => return Ok(cmp_amount_edges),
                        None => return Ok(cmp),
                    },
                    result => return result,
                }
            }
        }

        /// The main logic of the algorithm.
        ///
        /// Must not be used as the initial entry-point, but may be called by
        /// [`RecurMode::recur`] implementations.
        ///
        /// Returns same as [`Self::equiv`].
        ///
        /// # Errors
        /// Same as [`Self::equiv`].
        #[inline]
        pub fn equiv_main(
            &mut self,
            a: P::Node,
            b: P::Node,
        ) -> Result<<P::Node as Node>::Cmp, P::Error>
        {
            // Needed only because the `?` operator uses `From` instead of `Into`.
            macro_rules! try_into {
                ($e:expr) => {
                    ($e).map_err(Into::into)?
                };
            }

            let mut cmp = Cmp::new_equiv();

            // For trait method implementations that always return the same constant, dead
            // branches should be eliminated by the optimizer.  For the other methods, inlining
            // should be doable by the optimizer.

            if try_into!(self.descend_mode.do_traverse()) && a.id() != b.id() {
                cmp = a.equiv_modulo_edges(&b);
                if cmp.is_equiv() {
                    let mut edges_iter = EdgesIterTail::new([a, b]);
                    match edges_iter.next() {
                        Some(Ok(first)) => {
                            #[allow(clippy::shadow_unrelated)]
                            let [a, b] = &edges_iter.counterparts;
                            if try_into!(self.descend_mode.do_edges(a, b)) {
                                let edges_iter = EdgesIter::new(first, edges_iter);
                                cmp = try_into!(P::RecurMode::recur(self, edges_iter));
                            }
                        },
                        Some(Err(cmp_amount_edges)) => cmp = cmp_amount_edges,
                        None => (),
                    }
                }
            }

            Ok(cmp)
        }
    }

    /// [`Node::Index`] types must give their "zero" value, for their implementation of
    /// [`Default`].
    fn zero<T: Default>() -> T
    {
        T::default()
    }

    /// Get edges lazily, in increasing-index order.
    ///
    /// Enables avoiding consuming excessive space for `RecurMode` types like `VecStack`.
    pub struct EdgesIter<N: Node>(
        iter::Chain<iter::Once<<Self as Iterator>::Item>, EdgesIterTail<N>>,
    );

    impl<N: Node> EdgesIter<N>
    {
        fn new(
            first_edges: Counterparts<N>,
            tail_iter: EdgesIterTail<N>,
        ) -> Self
        {
            debug_assert!({
                let one = increment_index(&zero());
                tail_iter.next_index == one
            });
            Self(iter::once(Ok(first_edges)).chain(tail_iter))
        }
    }

    impl<N: Node> Iterator for EdgesIter<N>
    {
        type Item = CounterpartsResult<N>;

        #[inline]
        fn next(&mut self) -> Option<Self::Item>
        {
            self.0.next()
        }
    }

    struct EdgesIterTail<N: Node>
    {
        counterparts: Counterparts<N>,
        next_index:   Option<N::Index>,
    }

    impl<N: Node> EdgesIterTail<N>
    {
        /// Prepare to get the edges from `counterparts`.
        fn new(counterparts: Counterparts<N>) -> Self
        {
            Self { counterparts, next_index: Some(zero()) }
        }
    }

    impl<N: Node> Iterator for EdgesIterTail<N>
    {
        type Item = CounterpartsResult<N>;

        fn next(&mut self) -> Option<Self::Item>
        {
            if let Some(i) = &self.next_index {
                let [a, b] = &self.counterparts;
                match [a.get_edge(i), b.get_edge(i)] {
                    [Some(ae), Some(be)] => {
                        self.next_index = increment_index(i);
                        Some(Ok([ae, be]))
                    },
                    [None, None] => {
                        self.next_index = None;
                        None
                    },
                    [None, Some(_)] => Some(Err(N::Cmp::from_ord(Ordering::Less))),
                    [Some(_), None] => Some(Err(N::Cmp::from_ord(Ordering::Greater))),
                }
            }
            else {
                None
            }
        }
    }

    fn increment_index<T: Step>(i: &T) -> Option<T>
    {
        cfg_if! {
            if #[cfg(feature = "anticipate")] {
                Step::forward_checked(i.clone(), 1)
            }
            else {
                i.increment()
            }
        }
    }
}
