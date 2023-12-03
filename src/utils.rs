#[cfg(feature = "alloc")]
pub(crate) use lazy_collections::{
    LazierIterator,
    LazyVecQueue,
    LazyVecStack,
};
pub(crate) use non_advancing_iterator::NonAdvancingIterator;
pub use ref_id::RefId;


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
    /// possible, where `T` is some type of reference to the primary inner type `U` of an `N:
    /// Node` (and must not be a reference to an `N` itself).  This is safer for avoiding logic
    /// bugs, because it keeps any lifetimes of `T`.  While `*const U` can be safely used for some
    /// types, it is not guaranteed that some refactoring does not invalidate the (imaginary)
    /// lifetime of such a pointer.  (This crate is `forbid(unsafe_code)` but since such pointers
    /// would only be used as identifiers (and never dereferenced), such lifetime logic bugs could
    /// become hypothetically possible).
    ///
    /// If the `T` type is a "fat" reference, the additional metadata is compared and hashed,
    /// because such values with differing metadata could have different behavior and should be
    /// considered distinct.
    #[derive(Copy, Clone)]
    #[allow(clippy::exhaustive_structs)]
    pub struct RefId<T>(pub T);

    impl<T: Deref> RefId<T>
    {
        #[allow(clippy::explicit_auto_deref)]
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


#[cfg(feature = "alloc")]
mod lazy_collections
{
    extern crate alloc;
    use {
        super::NonAdvancingIterator,
        alloc::{
            collections::VecDeque,
            vec::Vec,
        },
    };

    /// A logical stack of items yielded by an internal stack of [`Iterator`]s.  Enables lazily
    /// generating items to reduce memory usage.
    pub(crate) struct LazyVecStack<I>(Vec<I>);

    /// A logical queue of items yielded by an internal queue of [`Iterator`]s.  Enables lazily
    /// generating items to reduce memory usage.
    pub(crate) struct LazyVecQueue<I>(VecDeque<I>);

    /// Somewhat similar to [`Flatten`](core::iter::Flatten) but designed to not consume ownership
    /// and not hold a "current" sub-iterator so that mutating to `extend` with further items may
    /// be done between `next` calls.
    pub(crate) trait LazierIterator
    {
        type SubIter: NonAdvancingIterator;

        fn extend(
            &mut self,
            subiter: Self::SubIter,
        );

        fn next_subiter_as_mut(&mut self) -> Option<&mut Self::SubIter>;

        fn next_subiter(&mut self) -> Option<Self::SubIter>;

        #[allow(clippy::inline_always)]
        #[inline(always)] // Actually makes a big difference.
        fn next(&mut self) -> Option<<Self::SubIter as Iterator>::Item>
        {
            while let Some(subiter) = self.next_subiter_as_mut() {
                let next = subiter.next();
                if !subiter.has_next() {
                    drop(self.next_subiter()); // Remove empty iterators, before returning `Some`.
                }
                if next.is_some() {
                    return next;
                }
            }
            None
        }
    }

    macro_rules! provided_impls {
        {
            $($t:ty {
                $with_capacity:path,
                $extend:ident,
                $next_subiter_as_mut:ident,
                $next_subiter:ident
            },)*
        } => {
            $(
                impl<I> $t
                {
                    pub(crate) fn with_capacity(capacity: usize) -> Self
                    {
                        Self($with_capacity(capacity))
                    }

                    pub(crate) fn clear(&mut self)
                    {
                        self.0.clear()
                    }
                }

                impl<I: NonAdvancingIterator> LazierIterator for $t
                {
                    type SubIter = I;

                    fn extend(
                        &mut self,
                        subiter: Self::SubIter,
                    )
                    {
                        self.0.$extend(subiter);
                    }

                    fn next_subiter_as_mut(&mut self) -> Option<&mut Self::SubIter>
                    {
                        self.0.$next_subiter_as_mut()
                    }

                    fn next_subiter(&mut self) -> Option<Self::SubIter>
                    {
                        self.0.$next_subiter()
                    }
                }
            )*
        };
    }

    provided_impls! {
        LazyVecStack<I> { Vec::with_capacity, push, last_mut, pop },
        LazyVecQueue<I> { VecDeque::with_capacity, push_back, front_mut, pop_front },
    }
}


mod non_advancing_iterator
{
    /// An `Iterator` that can repeatedly yield the same next item without advancing.
    ///
    /// Unlike [`Peekable`](core::iter::Peekable), this is not required to store and repeatedly
    /// yield the same value.  The logically-same next item may be dynamically regenerated as
    /// distinct equivalent values.
    pub(crate) trait NonAdvancingIterator: Iterator
    {
        /// Returns the next item without advancing the iterator.
        ///
        /// Multiple calls return the logically-same item, when the iterator is not advanced by
        /// some other means.  In such case, this may or may not be the same value.
        fn next_no_adv(&mut self) -> Option<Self::Item>;

        /// Returns `true` if the iterator has a next item, without advancing the iterator.
        fn has_next(&mut self) -> bool
        {
            self.next_no_adv().is_some()
        }
    }
}
