//! A package feature selects which (pseudo)random-number generator is chosen.
//!
//! It was not decided which would always be the best for all users, so instead support a variety.
//! If you add one that was not already here, consider whether it can be no-std, if you want to
//! use this crate without its "std" feature.

use core::num::NonZeroU16;


// These features are mutually exclusive.  If more than one are enabled, the compiler will error
// about there being multiple definitions of the `chosen` module.

#[cfg(all(feature = "fastrand", feature = "std"))]
#[path = "fastrand.rs"]
pub(super) mod chosen;

#[cfg(feature = "oorandom")]
#[path = "oorandom.rs"]
pub(super) mod chosen;

#[cfg(feature = "wyrng")]
#[path = "wyrng.rs"]
pub(super) mod chosen;


/// What the [`Interleave`](super::Interleave) type requires from a (pseudo)random-number
/// generator type.
///
/// The implementation of [`Default`] may or may not initialize each instance with a constant
/// seed.  Our use does not require random seeding.
pub(super) trait NumberGenerator: Default
{
    /// Return a (pseudo)random number in the range `0 .. exclusive_end`.
    fn rand_upto(
        &mut self,
        exclusive_end: NonZeroU16,
    ) -> u16;
}
