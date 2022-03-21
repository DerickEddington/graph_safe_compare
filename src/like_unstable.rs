pub(crate) use {
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

#[rustfmt::skip] // This unusual formatting preserves lines for cleaner diffs.
mod range_iter
{
    //! FUTURE: This should be removed and its use should be replaced with the `impl<A: Step>
    //! DoubleEndedIterator for Range<A>` (along with `Iterator::rev`) of `core`, if the unstable
    //! `step_trait` feature is stabilized so that our generic custom index types can `impl`
    //! `Step` to be used with it in `Range<Index>`.

    use cfg_if::cfg_if;

cfg_if!
{
if #[cfg(rust_lib_feature = "step_trait")]
{
    use core::ops::Range;

    /// Alias of generic [`Range`] that can be iterated with the new "step_trait" feature of the
    /// Standard Library, for backwards compatibility with the code that uses our `RangeIter`
    /// workaround type.
    #[deprecated]
    pub(crate) type RangeIter<T> = Range<T>;
}
else
{
    use core::{
        borrow::Borrow,
        ops::{
            AddAssign,
            Range,
            SubAssign,
        }
    };

    /// Enables iteration of a generic [`Range`].
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
}
}
