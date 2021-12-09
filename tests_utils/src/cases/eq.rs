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

        assert_eq!(
            $crate::cases::NoDrop(std::mem::ManuallyDrop::new($datum_trans(ddag1))),
            $crate::cases::NoDrop(std::mem::ManuallyDrop::new($datum_trans(ddag2)))
        );
    };
}


#[macro_export]
macro_rules! eq_tests
{
    (
        $alloc_trans:tt,
        $make_alloc:expr,
        $datum_trans:expr,
        #[$maybe_ignore_needs_our_algo:meta],
        #[$maybe_ignore_stack_overflow:meta],
        #[$maybe_omit_derived:meta]
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

            #[$maybe_omit_derived]
            mod derived_eq
            {
                use {
                    super::*,
                    std::convert::identity,
                };

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
                        identity
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
                        identity
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
        struct MyEq<'l>($my_type, PhantomData<&'l ()>);

        impl<'l> MyEq<'l>
        {
            fn new(d: $datum_type) -> Self
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
        /// Use the variation of the algorithm that uses the normal function call stack and so
        /// cannot handle very-deep graphs because stack overflow can happen.
        #[cfg(test)]
        mod callstack
        {
            $crate::eq_variation_mod_body!(
                cycle_deep_safe_compare::alt::basic::precheck_interleave_equiv::<
                    _,
                    cycle_deep_safe_compare::alt::basic::CallStack,
                    cycle_deep_safe_compare::alt::basic::CallStack>,
                $my_type, $datum_type, $alloc_trans, $make_alloc);

            $crate::eq_tests!($alloc_trans, $make_alloc, MyEq::new,
                              #[cfg(all())], #[ignore], #[cfg(all())]);
        }

        /// Use the variation of the algorithm that does not use the call stack and that can
        /// handle very-deep graphs and stack overflow cannot happen.
        #[cfg(test)]
        mod robust
        {
            $crate::eq_variation_mod_body!(
                cycle_deep_safe_compare::alt::basic::precheck_interleave_equiv::<
                    _,
                    cycle_deep_safe_compare::alt::basic::CallStack,
                    cycle_deep_safe_compare::alt::basic::robust::VecStack<_>>,
                $my_type, $datum_type, $alloc_trans, $make_alloc);

            tests_utils::eq_tests!($alloc_trans, $make_alloc, MyEq::new,
                                   #[cfg(all())], #[cfg(all())], #[cfg(any())]);
        }
    };
}
