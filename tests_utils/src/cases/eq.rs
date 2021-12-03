#[macro_export]
macro_rules! eq_case {
    (
        $trans:tt,
        $make_alloc:expr,
        $alloc_size:expr,
        $shape_size:expr,
        $shape_method:ident,
        $into_node:expr
    ) => {
        let alloc1 = $trans($make_alloc($alloc_size));
        let make1 = $crate::shapes::PairChainMaker::new_with($shape_size, alloc1);
        let ddag1 = make1.$shape_method();

        let alloc2 = $trans($make_alloc($alloc_size));
        let make2 = $crate::shapes::PairChainMaker::new_with($shape_size, alloc2);
        let ddag2 = make2.$shape_method();

        assert_eq!($into_node(ddag1), $into_node(ddag2));
    };
}

#[macro_export]
macro_rules! eq_tests {
    ($trans:tt, $make_alloc:expr, $into_node:expr) => {
        $crate::eq_tests!($trans, $make_alloc, $into_node, #[cfg(all())]);
    };
    ($trans:tt, $make_alloc:expr, $into_node:expr, #[$maybe_ignore:meta]) => {
        #[cfg(test)]
        mod degenerate
        {
            use super::*;

            #[test]
            #[$maybe_ignore]
            fn dag_fast()
            {
                $crate::eq_case!(
                    $trans,
                    $make_alloc,
                    $crate::DEGENERATE_TEST_DEPTH + 1,
                    $crate::DEGENERATE_TEST_DEPTH,
                    degenerate_dag,
                    $into_node
                );
            }

            #[test]
            #[$maybe_ignore]
            fn cyclic_works_and_fast()
            {
                $crate::eq_case!(
                    $trans,
                    $make_alloc,
                    $crate::DEGENERATE_TEST_DEPTH + 1,
                    $crate::DEGENERATE_TEST_DEPTH,
                    degenerate_cyclic,
                    $into_node
                );
            }

            mod long
            {
                use super::*;

                #[test]
                #[ignore]
                fn dag_stack_overflow()
                {
                    const DEPTH: u32 = 1000 * $crate::DEGENERATE_TEST_DEPTH;

                    $crate::eq_case!(
                        $trans,
                        $make_alloc,
                        DEPTH + 1,
                        DEPTH,
                        degenerate_dag,
                        $into_node
                    );
                }

                #[test]
                #[ignore]
                fn cyclic_stack_overflow()
                {
                    const DEPTH: u32 = 1000 * $crate::DEGENERATE_TEST_DEPTH;

                    $crate::eq_case!(
                        $trans,
                        $make_alloc,
                        DEPTH + 1,
                        DEPTH,
                        degenerate_cyclic,
                        $into_node
                    );
                }
            }

            mod derived_eq
            {
                use std::convert::identity;

                use super::*;

                #[test]
                #[ignore]
                fn dag_slow()
                {
                    $crate::eq_case!(
                        $trans,
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
                        $trans,
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
            #[ignore]
            fn stack_overflow()
            {
                $crate::eq_case!(
                    $trans,
                    $make_alloc,
                    2 * $crate::LONG_LIST_TEST_LENGTH + 1,
                    $crate::LONG_LIST_TEST_LENGTH,
                    list,
                    $into_node
                );
            }

            #[test]
            #[ignore]
            fn inverted_stack_overflow()
            {
                $crate::eq_case!(
                    $trans,
                    $make_alloc,
                    2 * $crate::LONG_LIST_TEST_LENGTH + 1,
                    $crate::LONG_LIST_TEST_LENGTH,
                    inverted_list,
                    $into_node
                );
            }
        }
    };
}
