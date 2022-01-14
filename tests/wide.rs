use {
    cycle_deep_safe_compare::Node,
    std::{
        convert::identity,
        rc::Rc,
    },
    tests_utils::{
        node_types::wide::Datum,
        sizes::great_width,
    },
};


#[derive(Clone, Debug)]
struct My(Rc<Datum>);

impl My
{
    fn new(depth: u32) -> Self
    {
        let width = f64::from(great_width()).powf(1.0 / f64::from(depth)).round() as usize;
        Self(Rc::new(Datum::degenerate_chain(width, depth)))
    }

    fn new_couple(depth: u32) -> (Self, Self)
    {
        (Self::new(depth), Self::new(depth))
    }
}

impl From<My> for Datum
{
    fn from(my: My) -> Self
    {
        Rc::try_unwrap(my.0).unwrap()
    }
}

impl Node for My
{
    type Cmp = bool;
    type Id = *const Datum;
    type Index = usize;

    fn id(&self) -> Self::Id
    {
        &*self.0
    }

    fn amount_edges(&self) -> Self::Index
    {
        self.0.width()
    }

    fn get_edge(
        &self,
        index: &Self::Index,
    ) -> Self
    {
        My(Rc::clone(&self.0[*index]))
    }

    fn equiv_modulo_edges(
        &self,
        _other: &Self,
    ) -> Self::Cmp
    {
        true
    }
}


macro_rules! case {
    ($depth:expr, $conv:expr, $assert_cmp:ident) => {{
        let (a, b) = My::new_couple($depth);
        let conv = $conv;
        $assert_cmp!(conv(a), conv(b));
    }};
}

macro_rules! depth_tests {
    ($conv:expr, $assert_cmp:ident) => {
        #[test]
        fn depth0()
        {
            case!(0, $conv, $assert_cmp);
        }

        #[test]
        fn depth1()
        {
            case!(1, $conv, $assert_cmp);
        }

        #[test]
        fn depth2()
        {
            case!(2, $conv, $assert_cmp);
        }

        #[test]
        fn depth3()
        {
            case!(3, $conv, $assert_cmp);
        }

        #[test]
        fn depth4()
        {
            case!(4, $conv, $assert_cmp);
        }
    };
}

macro_rules! variation_tests {
    ($name:ident) => {
        mod $name
        {
            use super::*;

            fn equiv(
                a: My,
                b: My,
            ) -> bool
            {
                cycle_deep_safe_compare::$name::equiv(&a, &b)
            }

            macro_rules! assert_equiv {
                ($a: expr,$b: expr) => {
                    assert!(equiv($a, $b))
                };
            }

            depth_tests!(identity, assert_equiv);
        }
    };
}

#[cfg(feature = "std")]
variation_tests!(robust);
#[cfg(feature = "std")]
variation_tests!(cycle_safe);
#[cfg(feature = "alloc")]
variation_tests!(deep_safe);
variation_tests!(basic);

mod derived_eq
{
    use super::*;

    depth_tests!(Into::<Datum>::into, assert_eq);
}
