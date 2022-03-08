#![cfg(any(rust_lib_feature = "test", rust_comp_feature = "unstable_features"))]
#![cfg_attr(not(rust_lib_feature = "test"), feature(test))]

extern crate test;


mod common;


mod inputs
{
    #![cfg_attr(not(feature = "std"), allow(dead_code))]

    use {
        super::common::{
            borrow_pair::{
                Datum,
                DatumAllocator,
                My,
            },
            defaults::*,
        },
        tests_utils::shapes::PairChainMaker,
    };

    const fn log2(n: u32) -> u32
    {
        (u32::BITS - 1) - n.leading_zeros()
    }

    /// Long enough to exceed the limit of "precheck" so that it aborts, when involved, and so
    /// that "interleave" must be done, when involved; but not very long.  `2*L` recursions for
    /// all variants.
    const LIST_LENGTH: u32 = 20 * PRECHECK_LIMIT;
    /// Long enough to cause stack overflow if the call-stack were used, and enough to cause
    /// reallocations of a vec-stack (when the kind of "tail-call elimination" with a vec-stack is
    /// not involved) but not of a vec-queue.
    const LONG_LIST_LENGTH: u32 = 2 * 2_u32.pow(17);
    /// Short enough that "precheck" completes, when involved.
    const SHORT_LIST_LENGTH: u32 = PRECHECK_LIMIT / 4;
    /// Deep enough to be slow for the basic variants but fast for the variants that use
    /// "interleave" (and exceed "precheck"); but not very deep.  `2^(D+1)-2` recursions for those
    /// that are basic; `2*D` for those that use "interleave".  The value is derived to cause the
    /// same amount of recursions as with lists of `LONG_LIST_LENGTH`, for basic variants.
    const DEGENERATE_DEPTH: u32 = log2(2 * LONG_LIST_LENGTH + 2) - 1;
    /// Deep enough to be far too slow for basic variants but doable for "interleave" variants,
    /// and enough to require reallocations of a vec-stack but not of a vec-queue.  The value is
    /// derived to cause the same amount of recursions as with lists of `LONG_LIST_LENGTH`, for
    /// "interleave" variants.
    const LONG_DEGENERATE_DEPTH: u32 = LONG_LIST_LENGTH;
    /// Shallow enough that "precheck" completes, when involved.
    const SHORT_DEGENERATE_DEPTH: u32 = log2(PRECHECK_LIMIT + 2) - 1;

    pub struct Input<'l>
    {
        pub head: My<'l>,
        pub tail: My<'l>,
    }

    type PairChainMakerMethod<'l> = fn(
        PairChainMaker<&'l DatumAllocator<Datum<'l>>, &'l Datum<'l>>,
    ) -> (&'l Datum<'l>, &'l Datum<'l>);

    fn new_inputs<'l>(
        method: PairChainMakerMethod<'l>,
        depth: u32,
        alloc: &'l DatumAllocator<Datum<'l>>,
    ) -> [Input<'l>; 2]
    {
        let new_input = || {
            let maker = PairChainMaker::new_with(depth, alloc);
            let (head, tail) = method(maker);
            Input { head: My(head), tail: My(tail) }
        };

        [new_input(), new_input()]
    }

    fn list_size(depth: u32) -> u32
    {
        2 * depth + 1
    }

    fn degenerate_size(depth: u32) -> u32
    {
        depth + 1
    }

    macro_rules! shape {
        ($name:ident, $depth:expr, $size:ident) => {
            shape! { $name, $name, $depth, $size }
        };
        ($name:ident, $make:ident, $depth:expr, $size:ident) => {
            pub mod $name
            {
                use super::*;

                pub fn alloc_size() -> u32
                {
                    let single_alloc_size = $size($depth);
                    2 * single_alloc_size
                }

                pub fn new_inputs<'l>(alloc: &'l DatumAllocator<Datum<'l>>) -> [Input<'l>; 2]
                {
                    super::new_inputs(PairChainMaker::$make, $depth, alloc)
                }
            }
        };
    }

    shape! { list, LIST_LENGTH, list_size }
    shape! { inverted_list, LIST_LENGTH, list_size }
    shape! { degenerate_dag, DEGENERATE_DEPTH, degenerate_size }
    shape! { degenerate_cyclic, DEGENERATE_DEPTH, degenerate_size }
    shape! { long_list, list, LONG_LIST_LENGTH, list_size }
    shape! { long_inverted_list, inverted_list, LONG_LIST_LENGTH, list_size }
    shape! { long_degenerate_dag, degenerate_dag, LONG_DEGENERATE_DEPTH, degenerate_size }
    shape! { long_degenerate_cyclic, degenerate_cyclic, LONG_DEGENERATE_DEPTH, degenerate_size }
    shape! { short_list, list, SHORT_LIST_LENGTH, list_size }
    shape! { short_inverted_list, inverted_list, SHORT_LIST_LENGTH, list_size }
    shape! { short_degenerate_dag, degenerate_dag, SHORT_DEGENERATE_DEPTH, degenerate_size }
    shape! { short_degenerate_cyclic, degenerate_cyclic, SHORT_DEGENERATE_DEPTH, degenerate_size }
}


mod into_bool
{
    use {
        core::cmp::Ordering,
        graph_safe_compare::Cmp,
    };

    pub trait IntoBool
    {
        fn into_bool(self) -> bool;
    }

    impl IntoBool for bool
    {
        fn into_bool(self) -> bool
        {
            self
        }
    }

    impl IntoBool for Ordering
    {
        fn into_bool(self) -> bool
        {
            self.is_eq()
        }
    }

    impl<T: Cmp, E> IntoBool for Result<T, E>
    {
        fn into_bool(self) -> bool
        {
            matches!(self, Ok(cmp) if cmp.is_equiv())
        }
    }
}


macro_rules! variation_benches {
    ($name:ident, [$($func:ident $(($($var:ident = $arg:expr),+))?),+]) => {
        $(
            #[bench]
            fn $func(bencher: &mut Bencher)
            {
                let alloc = DatumAllocator::<Datum<'_>>::new(alloc_size());
                let [input1, input2] = new_inputs(&alloc);
                $($(let $var = $arg;)*)?
                let f = || $name::$func($($($var,)*)? input1.head, input2.head);

                // Check that they are equivalent as expected.
                assert!(IntoBool::into_bool(f()));

                // Benchmark.
                bencher.iter(f);

                // Not needed with borrow_pair
                // // Drop cyclic and/or deep without stack overflow.
                // let [Input { head: My(head1), tail: My(tail1)},
                //      Input { head: My(head2), tail: My(tail2)}] = [input1, input2];
                // cycle_deep_safe_drop([(head1, tail1), (head2, tail2)]);
            }
        )+
    };
}

macro_rules! variation {
    ($name:ident, $shapes:tt, $benches:tt) => {
        variation!($name, graph_safe_compare::$name, $shapes, $benches);
    };
    ($name:ident, $use_path:path, [$($shape:ident),+], $benches:tt) => {
        mod $name
        {
            use {
                super::*,
                $use_path,
            };

            $(
                mod $shape
                {
                    use {
                        super::*,
                        crate::{
                            common::borrow_pair::{
                                Datum,
                                DatumAllocator,
                            },
                            inputs::{
                                $shape::{
                                    alloc_size,
                                    new_inputs
                                },
                            },
                            into_bool::IntoBool,
                        },
                        test::Bencher,
                    };

                    variation_benches!($name, $benches);
                }
            )+
        }
    };
}


variation! {
    basic,
    [
        list,
        inverted_list,
        degenerate_dag,
        short_list,
        short_inverted_list,
        short_degenerate_dag
    ],
    [equiv, limited_equiv(limit = usize::MAX)]
}

#[cfg(feature = "alloc")]
variation! {
    wide_safe,
    [
        list,
        inverted_list,
        degenerate_dag,
        long_list,
        long_inverted_list,
        short_list,
        short_inverted_list,
        short_degenerate_dag
    ],
    [equiv, limited_equiv(limit = u32::MAX)]
}

#[cfg(feature = "alloc")]
variation! {
    deep_safe,
    [
        list,
        inverted_list,
        degenerate_dag,
        long_list,
        long_inverted_list,
        short_list,
        short_inverted_list,
        short_degenerate_dag
    ],
    [equiv, limited_equiv(limit = u64::MAX)]
}

#[cfg(feature = "std")]
variation! {
    cycle_safe,
    [
        list,
        inverted_list,
        degenerate_dag,
        degenerate_cyclic,
        short_list,
        short_inverted_list,
        short_degenerate_dag,
        short_degenerate_cyclic
    ],
    [equiv, precheck_equiv]
}

#[cfg(feature = "std")]
variation! {
    robust,
    [
        list,
        inverted_list,
        degenerate_dag,
        degenerate_cyclic,
        long_list,
        long_inverted_list,
        long_degenerate_dag,
        long_degenerate_cyclic,
        short_list,
        short_inverted_list,
        short_degenerate_dag,
        short_degenerate_cyclic
    ],
    [equiv, precheck_equiv]
}


mod extra
{
    pub mod derived_eq
    {
        use crate::common::borrow_pair::{
            Datum,
            My,
        };

        pub fn eq<'l>(
            a: My<'l>,
            b: My<'l>,
        ) -> bool
        {
            let a: &Datum = a.0;
            let b: &Datum = b.0;
            <Datum as PartialEq>::eq(a, b)
        }
    }
}

variation! {
    derived_eq,
    crate::extra::derived_eq,
    [
        list,
        inverted_list,
        degenerate_dag,
        short_list,
        short_inverted_list,
        short_degenerate_dag
    ],
    [eq]
}
