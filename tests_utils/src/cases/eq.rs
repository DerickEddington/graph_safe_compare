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
        let (head1, tail1) = make1.$shape_method();

        let alloc2 = $alloc_trans($make_alloc($alloc_size));
        let make2 = $crate::shapes::PairChainMaker::new_with($shape_size, alloc2);
        let (head2, tail2) = make2.$shape_method();

        // TODO: With assert_eq!, failures try to format the values, but there are some huge
        // values for which this tries to consume all memory just to format. Need the Debug impls
        // to limit how big of a formatted string they generate or something.

        // The main purpose of this macro: test the `PartialEq` implementation:
        assert!($datum_trans(Clone::clone(&head1)) == $datum_trans(Clone::clone(&head2)));

        $crate::shapes::cycle_deep_safe_drop([(head1, tail1), (head2, tail2)]);
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
                    $crate::sizes::degenerate_depth() + 1,
                    $crate::sizes::degenerate_depth(),
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
                    $crate::sizes::degenerate_depth() + 1,
                    $crate::sizes::degenerate_depth(),
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
                    let depth = 1000 * $crate::sizes::degenerate_depth();

                    $crate::eq_case!(
                        $alloc_trans,
                        $make_alloc,
                        depth + 1,
                        depth,
                        degenerate_dag,
                        $datum_trans
                    );
                }

                #[test]
                #[$maybe_ignore_needs_our_algo]
                #[$maybe_ignore_stack_overflow]
                fn cyclic_stack_overflow()
                {
                    let depth = 1000 * $crate::sizes::degenerate_depth();

                    $crate::eq_case!(
                        $alloc_trans,
                        $make_alloc,
                        depth + 1,
                        depth,
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
                    2 * $crate::sizes::long_list_length() + 1,
                    $crate::sizes::long_list_length(),
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
                    2 * $crate::sizes::long_list_length() + 1,
                    $crate::sizes::long_list_length(),
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
            graph_safe_compare::Cmp as _,
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
                let cmp = $algo_func(self.0.clone(), other.0.clone());
                cmp.is_equiv()
            }
        }

        #[test]
        fn rudimentary()
        {
            use $crate::shapes::Leaf;

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
                    $crate::sizes::degenerate_depth() + 1,
                    $crate::sizes::degenerate_depth(),
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
                    $crate::sizes::degenerate_depth() + 1,
                    $crate::sizes::degenerate_depth(),
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
                    graph_safe_compare::basic::equiv,
                    $my_type, $datum_type, $alloc_trans, $make_alloc);

                $crate::eq_shapes_tests!($alloc_trans, $make_alloc, MyEq::new,
                                         #[ignore], #[ignore]);
            }

            fn limited_equiv<N: graph_safe_compare::Node>(
                a: N,
                b: N,
            ) -> bool
            {
                use graph_safe_compare::Cmp as _;

                const LIMIT: u32 = 50;
                matches!(graph_safe_compare::basic::limited_equiv(LIMIT, a, b),
                         Ok(cmp) if cmp.is_equiv())
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
                    graph_safe_compare::cycle_safe::equiv,
                    $my_type, $datum_type, $alloc_trans, $make_alloc);

                $crate::eq_shapes_tests!($alloc_trans, $make_alloc, MyEq::new,
                                         #[cfg(all())], #[ignore]);
            }

            mod precheck_interleave
            {
                $crate::eq_variation_mod_body!(
                    graph_safe_compare::cycle_safe::precheck_equiv,
                    $my_type, $datum_type, $alloc_trans, $make_alloc);

                $crate::eq_shapes_tests!($alloc_trans, $make_alloc, MyEq::new,
                                         #[cfg(all())], #[ignore]);
            }
        }

        #[cfg(test)]
        mod wide_safe
        {
            use super::*;

            mod vecstack
            {
                $crate::eq_variation_mod_body!(
                    graph_safe_compare::wide_safe::equiv,
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
                    graph_safe_compare::robust::equiv,
                    $my_type, $datum_type, $alloc_trans, $make_alloc);

                $crate::eq_shapes_tests!($alloc_trans, $make_alloc, MyEq::new,
                                         #[cfg(all())], #[cfg(all())]);
            }

            mod precheck_interleave_vecstack
            {
                $crate::eq_variation_mod_body!(
                    graph_safe_compare::robust::precheck_equiv,
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
            fn precheck_interleave_equiv<N: graph_safe_compare::Node + Clone>(
                a: N,
                b: N,
            ) -> bool
            {
                use {
                    graph_safe_compare::{
                        basic::recursion::callstack::CallStack,
                        cycle_safe::modes::interleave::{
                            self,
                            random::default,
                        },
                        wide_safe::recursion::vecstack::{
                            self,
                            VecStack,
                        },
                        generic::{
                            precheck_interleave,
                            equiv_classes::premade::hash_map,
                        },
                        Cmp as _,
                        utils::IntoOk as _,
                    },
                    core::{
                        convert::Infallible,
                        marker::PhantomData,
                    },
                };

                struct Args<N>(PhantomData<N>);

                impl<N: Node> precheck_interleave::Params<N> for Args<N>
                {
                    type Error = Infallible;
                    type PrecheckRecurMode = CallStack;
                    type InterleaveRecurMode = VecStack<Self>;
                    type InterleaveParams = Self;
                }

                impl<N: Node> vecstack::Params for Args<N>
                {
                    // Use custom value for this constant, not its default.
                    const INITIAL_CAPACITY: usize = 1 << 10;
                    type Node = N;
                }

                impl<N: Node> hash_map::Params for Args<N>
                {
                    // Use custom value for this constant, not its default.
                    const INITIAL_CAPACITY: usize = 0;
                    type Node = N;
                }

                impl<N: Node> interleave::Params for Args<N>
                {
                    // Use custom values for these constants, not their defaults.
                    const PRECHECK_LIMIT: u16 = 321;
                    const FAST_LIMIT_MAX: u16 = 3 * Self::PRECHECK_LIMIT;
                    const SLOW_LIMIT: u16 = Self::PRECHECK_LIMIT / 5;

                    type Node = N;
                    type Table = hash_map::Table<Self>;
                    type RNG = default::RandomNumberGenerator;
                }

                let cmp = precheck_interleave::equiv::<_, Args<_>>(a, b).into_ok();
                cmp.is_equiv()
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
