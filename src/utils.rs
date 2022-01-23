pub use {
    into_ok::IntoOk,
    range_iter::RangeIter,
    ref_id::RefId,
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

mod ref_id
{
    use core::{
        cmp::Ordering,
        hash::Hash,
        ops::Deref,
        ptr,
    };

    /// Compare and hash references by pointer.
    ///
    /// This should be used as the [`Node::Id`](crate::Node::Id), instead of `*const U`, where
    /// possible, where `T` is some type of reference to the primary inner type of an `N: Node`
    /// (and must not be a reference to an `N` itself), because it keeps any lifetimes when `T` is
    /// a `&` type, which is safer for avoiding logic bugs.  While `*const U` can be safely used
    /// for some types, where `U` is the primary inner type, it is not guaranteed that some
    /// refactoring does not invalidate the (imaginary) lifetime of such a pointer.  (This crate
    /// is `forbid(unsafe_code)` but since such pointers would only be used as identifiers (and
    /// never dereferenced), such lifetime logic bugs could become hypothetically possible).
    ///
    /// If the `T` type is a "fat" reference, the additional metadata is compared and hashed,
    /// because such values with differing metadata could have different behavior and should be
    /// considered distinct.
    #[derive(Copy, Clone)]
    #[allow(clippy::exhaustive_structs)]
    pub struct RefId<T>(pub T);

    impl<T: Deref> RefId<T>
    {
        #[inline]
        fn as_ptr(&self) -> *const T::Target
        {
            let p: *const T::Target = {
                let r: &T::Target = &*self.0;
                r
            };
            p
        }
    }

    impl<T: Deref> PartialEq for RefId<T>
    {
        #[inline]
        fn eq(
            &self,
            other: &Self,
        ) -> bool
        {
            ptr::eq(self.as_ptr(), other.as_ptr())
        }
    }
    impl<T: Deref> Eq for RefId<T> {}

    impl<T: Deref> Hash for RefId<T>
    {
        #[inline]
        fn hash<H: core::hash::Hasher>(
            &self,
            state: &mut H,
        )
        {
            ptr::hash(self.as_ptr(), state);
        }
    }

    impl<T: Deref> PartialOrd for RefId<T>
    {
        #[inline]
        fn partial_cmp(
            &self,
            other: &Self,
        ) -> Option<Ordering>
        {
            Some(Ord::cmp(self, other))
        }

        #[inline]
        fn lt(
            &self,
            other: &Self,
        ) -> bool
        {
            self.as_ptr() < other.as_ptr()
        }

        #[inline]
        fn le(
            &self,
            other: &Self,
        ) -> bool
        {
            self.as_ptr() <= other.as_ptr()
        }

        #[inline]
        fn gt(
            &self,
            other: &Self,
        ) -> bool
        {
            self.as_ptr() > other.as_ptr()
        }

        #[inline]
        fn ge(
            &self,
            other: &Self,
        ) -> bool
        {
            self.as_ptr() >= other.as_ptr()
        }
    }

    impl<T: Deref> Ord for RefId<T>
    {
        #[inline]
        fn cmp(
            &self,
            other: &Self,
        ) -> Ordering
        {
            Ord::cmp(&self.as_ptr(), &other.as_ptr())
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
