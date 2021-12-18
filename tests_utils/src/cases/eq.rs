#[macro_export]
macro_rules! eq_case {
    (
        $alloc_trans:tt,
        $make_alloc:expr,
        $alloc_size:expr,
        $shape_size:expr,
        $shape_method:ident,
        $datum_trans:expr
    ) => {
        let alloc1 = $alloc_trans($make_alloc($alloc_size));
        let make1 = $crate::shapes::PairChainMaker::new_with($shape_size, alloc1);
        let ddag1 = make1.$shape_method();

        let alloc2 = $alloc_trans($make_alloc($alloc_size));
        let make2 = $crate::shapes::PairChainMaker::new_with($shape_size, alloc2);
        let ddag2 = make2.$shape_method();

        // TODO: With assert_eq!, failures try to format the values, but there are some huge
        // values for which this tries to consume all memory just to format. Need the Debug impls
        // to limit how big of a formatted string they generate or something.
        assert!(
            $crate::cases::NoDrop(std::mem::ManuallyDrop::new($datum_trans(ddag1)))
            ==
            $crate::cases::NoDrop(std::mem::ManuallyDrop::new($datum_trans(ddag2)))
        );
    };
}


#[macro_export]
macro_rules! eq_shapes_tests
{
    (
        $alloc_trans:tt,
        $make_alloc:expr,
        $datum_trans:expr,
        #[$maybe_ignore_needs_our_algo:meta],
        #[$maybe_ignore_stack_overflow:meta]
    ) => {
        #[cfg(test)]
        mod degenerate
        {
            use super::*;

            #[test]
            #[$maybe_ignore_needs_our_algo]
            fn dag_fast()
            {
                $crate::eq_case!(
                    $alloc_trans,
                    $make_alloc,
                    $crate::DEGENERATE_TEST_DEPTH + 1,
                    $crate::DEGENERATE_TEST_DEPTH,
                    degenerate_dag,
                    $datum_trans
                );
            }

            #[test]
            #[$maybe_ignore_needs_our_algo]
            fn cyclic_works_and_fast()
            {
                $crate::eq_case!(
                    $alloc_trans,
                    $make_alloc,
                    $crate::DEGENERATE_TEST_DEPTH + 1,
                    $crate::DEGENERATE_TEST_DEPTH,
                    degenerate_cyclic,
                    $datum_trans
                );
            }

            mod long
            {
                use super::*;

                #[test]
                #[$maybe_ignore_needs_our_algo]
                #[$maybe_ignore_stack_overflow]
                fn dag_stack_overflow()
                {
                    const DEPTH: u32 = 1000 * $crate::DEGENERATE_TEST_DEPTH;

                    $crate::eq_case!(
                        $alloc_trans,
                        $make_alloc,
                        DEPTH + 1,
                        DEPTH,
                        degenerate_dag,
                        $datum_trans
                    );
                }

                #[test]
                #[$maybe_ignore_needs_our_algo]
                #[$maybe_ignore_stack_overflow]
                fn cyclic_stack_overflow()
                {
                    const DEPTH: u32 = 1000 * $crate::DEGENERATE_TEST_DEPTH;

                    $crate::eq_case!(
                        $alloc_trans,
                        $make_alloc,
                        DEPTH + 1,
                        DEPTH,
                        degenerate_cyclic,
                        $datum_trans
                    );
                }
            }
        }

        #[cfg(test)]
        mod long_list
        {
            use super::*;

            #[test]
            #[$maybe_ignore_stack_overflow]
            fn stack_overflow()
            {
                $crate::eq_case!(
                    $alloc_trans,
                    $make_alloc,
                    2 * $crate::LONG_LIST_TEST_LENGTH + 1,
                    $crate::LONG_LIST_TEST_LENGTH,
                    list,
                    $datum_trans
                );
            }

            #[test]
            #[$maybe_ignore_stack_overflow]
            fn inverted_stack_overflow()
            {
                $crate::eq_case!(
                    $alloc_trans,
                    $make_alloc,
                    2 * $crate::LONG_LIST_TEST_LENGTH + 1,
                    $crate::LONG_LIST_TEST_LENGTH,
                    inverted_list,
                    $datum_trans
                );
            }
        }
    };
}


#[macro_export]
macro_rules! eq_variation_mod_body {
    ($algo_func:path, $my_type:ty, $datum_type:ty, $alloc_trans:tt, $make_alloc:expr) => {
        use {
            super::*,
            std::marker::PhantomData,
        };

        #[derive(Debug)]
        pub struct MyEq<'l>($my_type, PhantomData<&'l ()>);

        impl<'l> MyEq<'l>
        {
            pub fn new(d: $datum_type) -> Self
            {
                Self(My(d), PhantomData)
            }
        }

        impl<'l> PartialEq for MyEq<'l>
        {
            fn eq(
                &self,
                other: &Self,
            ) -> bool
            {
                $algo_func(&self.0, &other.0)
            }
        }

        #[test]
        fn rudimentary()
        {
            let alloc1 = $alloc_trans($make_alloc(1));
            let alloc2 = $alloc_trans($make_alloc(1));
            let leaf1 = Leaf::new_in(&alloc1);
            let leaf2 = Leaf::new_in(&alloc2);
            assert_eq!(MyEq::new(leaf1), MyEq::new(leaf2));
        }
    };
}


#[macro_export]
macro_rules! eq_variations_tests
{
    ($my_type:ty, $datum_type:ty, $alloc_trans:tt, $make_alloc:expr)
        =>
    {
        #[cfg(test)]
        mod derived_eq
        {
            use super::*;

            #[test]
            #[ignore]
            fn dag_slow()
            {
                $crate::eq_case!(
                    $alloc_trans,
                    $make_alloc,
                    $crate::DEGENERATE_TEST_DEPTH + 1,
                    $crate::DEGENERATE_TEST_DEPTH,
                    degenerate_dag,
                    std::convert::identity
                );
            }

            #[test]
            #[ignore]
            fn cyclic_stack_overflow()
            {
                $crate::eq_case!(
                    $alloc_trans,
                    $make_alloc,
                    $crate::DEGENERATE_TEST_DEPTH + 1,
                    $crate::DEGENERATE_TEST_DEPTH,
                    degenerate_cyclic,
                    std::convert::identity
                );
            }
        }

        #[cfg(test)]
        mod basic
        {
            use super::*;

            mod unlimited
            {
                $crate::eq_variation_mod_body!(
                    cycle_deep_safe_compare::basic::equiv,
                    $my_type, $datum_type, $alloc_trans, $make_alloc);

                $crate::eq_shapes_tests!($alloc_trans, $make_alloc, MyEq::new,
                                         #[ignore], #[ignore]);
            }

            fn limited_equiv<N: cycle_deep_safe_compare::Node>(
                a: &N,
                b: &N,
            ) -> bool
            {
                const LIMIT: i32 = 50;
                matches!(cycle_deep_safe_compare::basic::limited_equiv(LIMIT, a, b), Ok(true))
            }

            mod limited
            {
                $crate::eq_variation_mod_body!(
                    super::limited_equiv,
                    $my_type, $datum_type, $alloc_trans, $make_alloc);

                $crate::eq_shapes_tests!($alloc_trans, $make_alloc, MyEq::new,
                                         #[ignore], #[ignore]);
            }
        }

        #[cfg(test)]
        mod cycle_safe
        {
            use super::*;

            mod interleave
            {
                $crate::eq_variation_mod_body!(
                    cycle_deep_safe_compare::cycle_safe::equiv,
                    $my_type, $datum_type, $alloc_trans, $make_alloc);

                $crate::eq_shapes_tests!($alloc_trans, $make_alloc, MyEq::new,
                                         #[cfg(all())], #[ignore]);
            }

            mod precheck_interleave
            {
                $crate::eq_variation_mod_body!(
                    cycle_deep_safe_compare::cycle_safe::precheck_equiv,
                    $my_type, $datum_type, $alloc_trans, $make_alloc);

                $crate::eq_shapes_tests!($alloc_trans, $make_alloc, MyEq::new,
                                         #[cfg(all())], #[ignore]);
            }
        }

        #[cfg(test)]
        mod deep_safe
        {
            use super::*;

            mod vecstack
            {
                $crate::eq_variation_mod_body!(
                    cycle_deep_safe_compare::deep_safe::equiv,
                    $my_type, $datum_type, $alloc_trans, $make_alloc);

                $crate::eq_shapes_tests!($alloc_trans, $make_alloc, MyEq::new,
                                         #[ignore], #[cfg(all())]);
            }
        }

        #[cfg(test)]
        mod robust
        {
            use super::*;

            mod interleave_vecstack
            {
                $crate::eq_variation_mod_body!(
                    cycle_deep_safe_compare::robust::equiv,
                    $my_type, $datum_type, $alloc_trans, $make_alloc);

                $crate::eq_shapes_tests!($alloc_trans, $make_alloc, MyEq::new,
                                         #[cfg(all())], #[cfg(all())]);
            }

            mod precheck_interleave_vecstack
            {
                $crate::eq_variation_mod_body!(
                    cycle_deep_safe_compare::robust::precheck_equiv,
                    $my_type, $datum_type, $alloc_trans, $make_alloc);

                $crate::eq_shapes_tests!($alloc_trans, $make_alloc, MyEq::new,
                                         #[cfg(all())], #[cfg(all())]);
            }
        }

        #[cfg(test)]
        mod generic
        {
            use super::*;

            /// Use the call-stack for the precheck since that is limited and will not overflow
            /// the stack when the stack is already shallow, and use the vector-stack for the
            /// interleave so great depth is supported since an input could be very-deep.
            fn precheck_interleave_equiv<N: cycle_deep_safe_compare::Node>(
                a: &N,
                b: &N,
            ) -> bool
            {
                use cycle_deep_safe_compare::{
                    basic::recursion::callstack::CallStack,
                    deep_safe::recursion::vecstack::VecStack,
                    generic::{
                        self,
                        equiv_classes::premade::HashMap,
                    },
                };

                generic::precheck_interleave_equiv::<
                        _, HashMap<_>, CallStack, VecStack<_>>(
                    a, b)
            }

            mod precheck_interleave_callstack_vecstack
            {
                $crate::eq_variation_mod_body!(
                    super::precheck_interleave_equiv,
                    $my_type, $datum_type, $alloc_trans, $make_alloc);

                $crate::eq_shapes_tests!($alloc_trans, $make_alloc, MyEq::new,
                                         #[cfg(all())], #[cfg(all())]);
            }
        }

    };
}
