#[cfg(feature = "alloc")]
pub(crate) use lazy_collections::{
    LazierIterator,
    LazyVecQueue,
    LazyVecStack,
};
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
        alloc::{
            collections::VecDeque,
            vec::Vec,
        },
        core::iter::Peekable,
    };

    /// A logical stack of items yielded by an internal stack of [`Iterator`]s.  Enables lazily
    /// generating items to reduce memory usage.
    pub(crate) struct LazyVecStack<I: Iterator>(pub(crate) Vec<Peekable<I>>);

    /// A logical queue of items yielded by an internal queue of [`Iterator`]s.  Enables lazily
    /// generating items to reduce memory usage.
    pub(crate) struct LazyVecQueue<I: Iterator>(pub(crate) VecDeque<Peekable<I>>);

    /// Somewhat similar to [`Flatten`](core::iter::Flatten) but designed to not consume ownership
    /// and not hold a "current" sub-iterator so that mutating to `extend` with further items may
    /// be done between `next` calls.
    pub(crate) trait LazierIterator
    {
        type SubIter: Iterator;

        fn extend(
            &mut self,
            subiter: Self::SubIter,
        );

        fn next_subiter_as_mut(&mut self) -> Option<&mut Peekable<Self::SubIter>>;

        fn next_subiter(&mut self) -> Option<Peekable<Self::SubIter>>;

        #[allow(clippy::inline_always)]
        #[inline(always)] // Actually makes a big difference.
        fn next(&mut self) -> Option<<Self::SubIter as Iterator>::Item>
        {
            while let Some(subiter) = self.next_subiter_as_mut() {
                let next = subiter.next();
                if subiter.peek().is_none() {
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
        { $($t:ty { $extend:ident, $next_subiter_as_mut:ident, $next_subiter:ident },)* } => {
            $(
                impl<I: Iterator> LazierIterator for $t
                {
                    type SubIter = I;

                    fn extend(
                        &mut self,
                        subiter: Self::SubIter,
                    )
                    {
                        self.0.$extend(subiter.peekable());
                    }

                    fn next_subiter_as_mut(&mut self) -> Option<&mut Peekable<Self::SubIter>>
                    {
                        self.0.$next_subiter_as_mut()
                    }

                    fn next_subiter(&mut self) -> Option<Peekable<Self::SubIter>>
                    {
                        self.0.$next_subiter()
                    }
                }
            )*
        };
    }

    provided_impls! {
        LazyVecStack<I> { push, last_mut, pop },
        LazyVecQueue<I> { push_back, front_mut, pop_front },
    }
}
