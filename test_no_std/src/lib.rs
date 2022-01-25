//! Integration test of using the parent package as `no_std` from a library.

#![no_std]


pub use my::My;

mod my
{
    use graph_safe_compare::{
        basic::equiv,
        Node,
    };

    #[derive(Copy, Clone)]
    pub struct My;

    impl Node for My
    {
        type Cmp = bool;
        type Id = ();
        type Index = u8;

        fn id(&self) -> Self::Id {}

        fn amount_edges(&self) -> Self::Index
        {
            0
        }

        fn get_edge(
            &self,
            _index: &Self::Index,
        ) -> Self
        {
            unreachable!()
        }

        fn equiv_modulo_edges(
            &self,
            _other: &Self,
        ) -> bool
        {
            true
        }
    }

    /// The usual reason for using `graph_safe_compare` is to impl this trait, but usually
    /// `basic::equiv` would not be used and instead the more-capable functionality would be, but
    /// using it suffices for this test where the premade more-capable functions are not present
    /// (due to no "std") (would have to go to more effort to provide custom types for the generic
    /// more-capable functions that are present).
    impl PartialEq for My
    {
        fn eq(
            &self,
            other: &Self,
        ) -> bool
        {
            equiv(*self, *other)
        }
    }
    impl Eq for My {}
}


#[cfg(not(any(feature = "std", test)))]
mod for_dylib_and_bin
{
    use libc::abort;

    // When `cfg(test)` is true, i.e. when building this crate as a `--test` or bench harness,
    // `libtest` is linked which links `libstd` which already provides a `panic_handler`, and that
    // would conflict.  Without conditional compilation, the following error would happen:
    //     error[E0152]: found duplicate lang item `panic_impl`
    //
    // This also serves to detect, and fail the build, if `libstd` does somehow end up being
    // linked in.
    #[panic_handler]
    fn panic(_info: &core::panic::PanicInfo) -> !
    {
        // Abort by calling the standard C `_Noreturn void abort(void)`.  Not strictly necessary
        // for this crate to build, since `loop {}` could be used instead to satisfy the `!`
        // return type, but `abort` causes the program to actually terminate if a panic occurs
        // (instead of looping infinitely).
        unsafe {
            abort();
        }
    }
}


/// When using the parent package with its "std" feature enabled, check that we can use its items
/// that are only available with that feature.
#[macro_export]
macro_rules! static_assert_dep_is_std {
    () => {
        mod static_assert_dep_is_std
        {
            #[allow(unused_imports)]
            use graph_safe_compare::{
                cycle_safe::{
                    equiv,
                    precheck_equiv,
                },
                wide_safe,
                generic::equiv_classes::premade::{
                    hash_map::Table,
                    rc::Rc,
                },
                robust,
            };
        }
    };
}

#[cfg(any(feature = "std", feature = "prove_dep_is_no_std"))]
static_assert_dep_is_std!();


#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn it_works()
    {
        let (a, b) = (My, My);
        assert!(a == b);
    }
}
