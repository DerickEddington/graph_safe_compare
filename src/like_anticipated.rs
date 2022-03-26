pub use step::Step;
pub(crate) use {
    core::convert::Infallible,
    into_ok::IntoOk,
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
    pub trait Step: Sized
    {
        /// Return the incremented value of `self`.
        ///
        /// If this would overflow the range of values supported by `Self`, returns None.
        #[must_use]
        fn increment(&self) -> Option<Self>;
    }

    macro_rules! provided_impls
    {
        { $($t:ty)* } => {
            $(
                impl Step for $t
                {
                    #[inline]
                    fn increment(&self) -> Option<Self>
                    {
                        self.checked_add(1)
                    }
                }
            )*
        }
    }

    provided_impls! { u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 usize isize }
}
