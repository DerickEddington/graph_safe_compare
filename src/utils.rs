pub use {
    into_ok::IntoOk,
    range_iter::RangeIter,
};

mod into_ok
{
    // FUTURE: This should be removed and its use should be replaced with the unstable
    // `unwrap_infallible` feature, if that is ever stabilized.

    use core::convert::Infallible;

    /// Only for certain multi-variant types that can only be their "ok" variant.
    pub trait IntoOk
    {
        /// Contained in the "ok" variant.
        type T;

        /// Convert an "ok" variant into its contained value.
        ///
        /// Panics must be truly impossible.
        fn into_ok(self) -> Self::T;
    }

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

mod range_iter
{
    // FUTURE: This should be removed and its use should be replaced with the `impl<A: Step>
    // DoubleEndedIterator for Range<A>` (along with `Iterator::rev`) of `core`, if the unstable
    // `step_trait` feature is stabilized so that our generic custom index types can `impl` `Step`
    // to be used with it in `Range<Index>`.

    use core::ops::{
        AddAssign,
        Range,
        SubAssign,
    };

    /// Enables iteration of a generic [`Range`].
    #[allow(clippy::exhaustive_structs)]
    pub struct RangeIter<T>(pub Range<T>);

    /// Yields in increasing order.
    impl<T> Iterator for RangeIter<T>
    where T: Ord + Clone + From<u8> + AddAssign
    {
        type Item = T;

        #[inline]
        fn next(&mut self) -> Option<Self::Item>
        {
            (self.0.start < self.0.end).then(|| {
                let next = self.0.start.clone();
                self.0.start += 1.into();
                next
            })
        }
    }

    /// Yields in decreasing order.
    impl<T> DoubleEndedIterator for RangeIter<T>
    where T: Ord + From<u8> + SubAssign + Clone + AddAssign
    {
        #[inline]
        fn next_back(&mut self) -> Option<Self::Item>
        {
            (self.0.start < self.0.end).then(|| {
                self.0.end -= 1.into();
                self.0.end.clone()
            })
        }
    }
}
