pub use step::Step;
pub(crate) use {
    core::convert::Infallible,
    into_ok::IntoOk,
    range_iter::RangeIter,
};


mod into_ok
{
    //! FUTURE: This should be removed and its use should be replaced with the `unwrap_infallible`
    //! feature, if that is ever stabilized.  And the `#[allow(unstable_name_collisions)]` that
    //! are applied to calls of `IntoOk::into_ok` should also be removed.

    use core::convert::Infallible;

    /// Only for certain multi-variant types that can only be their "ok" variant.
    #[cfg_attr(rust_lib_feature = "unwrap_infallible", deprecated)]
    pub(crate) trait IntoOk
    {
        /// Contained in the "ok" variant.
        type T;

        /// Convert an "ok" variant into its contained value.
        ///
        /// Panics must be truly impossible.
        fn into_ok(self) -> Self::T;
    }

    #[cfg(not(rust_lib_feature = "unwrap_infallible"))]
    impl<T> IntoOk for Result<T, Infallible>
    {
        type T = T;

        #[inline]
        fn into_ok(self) -> Self::T
        {
            #![allow(clippy::expect_used)] // Truly infallible.
            self.expect("infallible")
        }
    }
}


mod step
{
    //! FUTURE: This should be removed, if the unstable `step_trait` feature is stabilized.  Doing
    //! that will be a breaking change involving a version increase.

    /// Increments [`Node::Index`](crate::Node::Index) types.
    ///
    /// Will be deprecated in favor of the `step_trait` feature if that is stabilized.  The `Step`
    /// trait of that feature is different, and so when this crate changes to require that it will
    /// be a breaking change involving a version increase of this crate.
    #[cfg_attr(rust_lib_feature = "step_trait", deprecated)]
    pub trait Step
    {
        /// Return the incremented value of `self`.
        ///
        /// Only called when it is guaranteed to not overflow, i.e. when it is already known that
        /// there is a greater value.
        #[must_use]
        fn increment(&self) -> Self;
    }

    macro_rules! provided_impls
    {
        { $($t:ty)* } => {
            $(
                impl Step for $t
                {
                    #[allow(clippy::integer_arithmetic)]
                    #[inline]
                    fn increment(&self) -> Self
                    {
                        self + 1
                    }
                }
            )*
        }
    }

    provided_impls! { u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize }
}


mod range_iter
{
    //! FUTURE: This should be removed and its use should be replaced with the `impl<A: Step>
    //! Iterator for Range<A>` of `core`, if the unstable `step_trait` feature is stabilized, so
    //! that our generic custom index types can `impl` `core::iter::Step` so that `Range<Index>`
    //! can be used instead of `RangeIter`.

    use {
        super::Step,
        core::{
            borrow::Borrow,
            mem,
            ops::Range,
        },
    };

    /// Enables iteration of a generic [`Range`].
    #[cfg_attr(rust_lib_feature = "step_trait", deprecated)]
    #[allow(clippy::exhaustive_structs)]
    pub(crate) struct RangeIter<T>(Range<T>);

    impl<T> From<Range<T>> for RangeIter<T>
    {
        #[inline]
        fn from(range: Range<T>) -> Self
        {
            Self(range)
        }
    }

    impl<T> Borrow<Range<T>> for RangeIter<T>
    {
        #[inline]
        fn borrow(&self) -> &Range<T>
        {
            &self.0
        }
    }

    /// Yields in increasing order.
    impl<T> Iterator for RangeIter<T>
    where T: Step + Ord
    {
        type Item = T;

        #[inline]
        fn next(&mut self) -> Option<Self::Item>
        {
            (self.0.start < self.0.end).then(|| {
                let incr = self.0.start.increment();
                mem::replace(&mut self.0.start, incr)
            })
        }
    }
}
