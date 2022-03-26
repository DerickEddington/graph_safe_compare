//! Integration test of using the parent package as `no_std` from an executable.

#![no_std]
// Must have this, otherwise the following error would happen:
//     error: requires `start` lang_item
#![no_main]

// Only link the lib to have its `panic_handler`.
extern crate test_no_std as _;

#[cfg(any(feature = "std", feature = "prove_dep_is_no_std"))]
test_no_std::static_assert_dep_is_std!();


use {
    core::cmp::Ordering,
    graph_safe_compare::{
        basic::equiv,
        Node,
    },
    libc::{
        c_char,
        c_int,
    },
};


#[derive(Copy, Clone)]
pub struct It(i32);

impl Node for It
{
    type Cmp = Ordering;
    type Id = i32;
    type Index = u8;

    fn id(&self) -> Self::Id
    {
        self.0
    }

    fn get_edge(
        &self,
        _index: &Self::Index,
    ) -> Option<Self>
    {
        None
    }

    fn equiv_modulo_edges(
        &self,
        other: &Self,
    ) -> Ordering
    {
        self.0.cmp(&other.0)
    }
}

/// The usual reason for using `graph_safe_compare` is to impl this trait, but usually
/// `basic::equiv` would not be used and instead the more-capable functionality would be, but
/// using it suffices for this test where the premade more-capable functions are not present (due
/// to no "std") (would have to go to more effort to provide custom types for the generic
/// more-capable functions that are present).
impl PartialEq for It
{
    fn eq(
        &self,
        other: &Self,
    ) -> bool
    {
        equiv(*self, *other).is_eq()
    }
}
impl Eq for It {}


// Must provide the classic entry-point `main` symbol, otherwise the following error would happen
// (on NixOS Linux, at least):
//
//     error: linking with `cc` failed: exit status: 1
//       = note: .../ld: .../Scrt1.o: in function `_start':
//               .../start.S:104: undefined reference to `main'
//               collect2: error: ld returned 1 exit status
#[no_mangle] // ensure that this symbol is called `main` in the output
pub extern "C" fn main(
    _argc: c_int,
    _argv: *const *const c_char,
) -> c_int
{
    my_main();
    0
}


fn my_main()
{
    let (a, b) = (It(42), It(42));
    assert!(a == b);
}
