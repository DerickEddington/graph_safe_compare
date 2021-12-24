//! Package features select which (pseudo)random-number generator(s) is/are available, and the
//! highest-priority one is the default used with the premade items.
//!
//! It was not decided which would always be the best for all users, so instead support a variety.
//! If you add one that was not already here, consider whether it can be no-std, if you want to
//! use this crate without its "std" feature.

use {
    cfg_if::cfg_if,
    core::num::NonZeroU16,
};

// The (P)RNG chosen as default, by priority.
cfg_if! {
    if #[cfg(feature = "fastrand")] {
        pub use self::fastrand as default;
    }
    else if #[cfg(feature = "wyrng")] {
        pub use self::wyrng as default;
    }
    else if #[cfg(feature = "oorandom")] {
        pub use self::oorandom as default;
    }
}


#[cfg(all(feature = "fastrand", feature = "std"))]
pub mod fastrand;

#[cfg(feature = "oorandom")]
pub mod oorandom;

#[cfg(feature = "wyrng")]
pub mod wyrng;


/// What the [`Interleave`](super::Interleave) type requires from a (pseudo)random-number
/// generator type.
///
/// The implementation of [`Default`] may or may not initialize each instance with a constant
/// seed.  Our use does not require random seeding.
pub trait NumberGenerator: Default
{
    /// Return a (pseudo)random number in the range `0 .. exclusive_end`.
    fn rand_upto(
        &mut self,
        exclusive_end: NonZeroU16,
    ) -> u16;
}
