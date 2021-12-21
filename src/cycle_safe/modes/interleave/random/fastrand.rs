//! Use [fastrand](https://crates.io/crates/fastrand) as the choice of PRNG.

use core::num::NonZeroU16;

/// Use the thread-local-state ability of [`fastrand`], represented as this zero-sized unit struct
/// so that our traits can be `impl`ed on it.
#[derive(Default)]
pub(in super::super) struct RandomNumberGenerator;

impl super::NumberGenerator for RandomNumberGenerator
{
    fn rand_upto(
        &mut self,
        exclusive_end: NonZeroU16,
    ) -> u16
    {
        fastrand::u16(0 .. exclusive_end.get())
    }
}
