//! Use [oorandom](https://crates.io/crates/oorandom) as the choice of PRNG.

use {
    core::num::NonZeroU16,
    oorandom::Rand32,
};

/// Wrap [`Rand32`] so that our traits can be `impl`ed on it.
pub(in super::super) struct RandomNumberGenerator(Rand32);

impl Default for RandomNumberGenerator
{
    fn default() -> Self
    {
        /// The result of `Rand32::new(0).state()`.
        #[allow(clippy::unreadable_literal)]
        const STATE: (u64, u64) = (10116158231463745938, 2885390081777926815);

        Self(Rand32::from_state(STATE))
    }
}

impl super::NumberGenerator for RandomNumberGenerator
{
    fn rand_upto(
        &mut self,
        exclusive_end: NonZeroU16,
    ) -> u16
    {
        #![allow(clippy::as_conversions, clippy::cast_possible_truncation)]

        let exclusive_end: u32 = exclusive_end.get().into();

        // This `as` conversion cast is ok because the result will always be in the intended range
        // because the `exclusive_end` argument is `u16` and so the greatest possible result is
        // `u16::MAX - 1`.
        self.0.rand_range(0 .. exclusive_end) as u16
    }
}
