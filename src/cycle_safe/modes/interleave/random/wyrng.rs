//! Use the `wyrng` of [wyhash](https://crates.io/crates/wyhash/) as the choice of PRNG.

use {
    core::num::NonZeroU16,
    wyhash::wyrng,
};

/// Wrap a [`wyrng`] state value so that our traits can be `impl`ed on it.
#[derive(Default)]
pub struct RandomNumberGenerator(u64);

impl super::NumberGenerator for RandomNumberGenerator
{
    #[inline]
    fn rand_upto(
        &mut self,
        exclusive_end: NonZeroU16,
    ) -> u16
    {
        #![allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        // This `as` conversion cast is ok because we want truncation.
        wyrng(&mut self.0) as u16 % exclusive_end
    }
}
