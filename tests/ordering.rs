#[cfg(feature = "alloc")]
use cycle_deep_safe_compare::deep_safe;
#[cfg(feature = "std")]
use cycle_deep_safe_compare::{
    cycle_safe,
    robust,
};
use {
    cycle_deep_safe_compare::{
        basic,
        Node,
    },
    std::cmp::Ordering,
    Datum::*,
    Ordering::*,
};


#[derive(Debug, Eq, PartialEq)]
enum Datum
{
    A,
    B,
    C(char),
    D([Option<Box<Self>>; 2]),
    E(Vec<Self>),
}

impl Node for &Datum
{
    type Cmp = Ordering;
    type Id = *const Datum;
    type Index = usize;

    fn id(&self) -> Self::Id
    {
        *self
    }

    fn amount_edges(&self) -> Self::Index
    {
        match self {
            A => 0,
            B => 0,
            C(_) => 0,
            D([None, None]) => 0,
            D([None, Some(_)]) => 1,
            D([Some(_), None]) => 1,
            D([Some(_), Some(_)]) => 2,
            E(v) => v.len(),
        }
    }

    fn get_edge(
        &self,
        index: &Self::Index,
    ) -> Self
    {
        match (self, index) {
            (D([Some(d), None]), 0) => d,
            (D([None, Some(d)]), 0) => d,
            (D([Some(d), Some(_)]), 0) => d,
            (D([Some(_), Some(d)]), 1) => d,
            (E(v), &i) => &v[i],
            _ => unimplemented!(),
        }
    }

    fn equiv_modulo_edges(
        &self,
        other: &Self,
    ) -> Self::Cmp
    {
        match (self, other) {
            (A, A) => Equal,
            (A, B) => Less,
            (A, C(_)) => Less,
            (A, D(_)) => Less,
            (A, E(_)) => Less,
            (B, A) => Greater,
            (B, B) => Equal,
            (B, C(_)) => Less,
            (B, D(_)) => Less,
            (B, E(_)) => Less,
            (C(_), A) => Greater,
            (C(_), B) => Greater,
            (C(c1), C(c2)) => c1.cmp(c2),
            (C(_), D(_)) => Less,
            (C(_), E(_)) => Less,
            (D(_), A) => Greater,
            (D(_), B) => Greater,
            (D(_), C(_)) => Greater,
            (D([None, _]), D([Some(_), _])) => Less,
            (D([None, None]), D([None, Some(_)])) => Less,
            (D([Some(_), None]), D([Some(_), Some(_)])) => Less,
            (D([Some(_), _]), D([None, _])) => Greater,
            (D([None, Some(_)]), D([None, None])) => Greater,
            (D([Some(_), Some(_)]), D([Some(_), None])) => Greater,
            (D([None, None]), D([None, None])) => Equal,
            (D([Some(_), None]), D([Some(_), None])) => Equal,
            (D([None, Some(_)]), D([None, Some(_)])) => Equal,
            (D([Some(_), Some(_)]), D([Some(_), Some(_)])) => Equal,
            (E(_), A) => Greater,
            (E(_), B) => Greater,
            (E(_), C(_)) => Greater,
            (E(_), E(_)) => Equal,
            // For D and E, only the amount of edges matters.
            (D(_), E(_)) => Equal,
            (E(_), D(_)) => Equal,
        }
    }
}


macro_rules! case {
    ($a:expr, $b:expr => $r:expr) => {{
        let a: &&Datum = &&$a;
        let b: &&Datum = &&$b;
        let r: Ordering = $r;

        #[cfg(feature = "std")]
        {
            assert_eq!(robust::equiv(a, b), r);
            assert_eq!(robust::precheck_equiv(a, b), r);
            assert_eq!(cycle_safe::equiv(a, b), r);
            assert_eq!(cycle_safe::precheck_equiv(a, b), r);
        }

        #[cfg(feature = "alloc")]
        assert_eq!(deep_safe::equiv(a, b), r);

        assert_eq!(basic::equiv(a, b), r);
        assert_eq!(basic::limited_equiv(usize::MAX, a, b).unwrap(), r);

        r
    }};
}


fn a() -> Box<Datum>
{
    Box::new(A)
}
fn b() -> Box<Datum>
{
    Box::new(B)
}
fn c(c: char) -> Box<Datum>
{
    Box::new(C(c))
}
fn d(
    l: Option<Box<Datum>>,
    r: Option<Box<Datum>>,
) -> Box<Datum>
{
    Box::new(D([l, r]))
}
fn e(v: Vec<Datum>) -> Box<Datum>
{
    Box::new(E(v))
}


#[test]
fn variants()
{
    case!(A, A => Equal);
    case!(A, B => Less);
    case!(A, C(' ') => Less);
    case!(A, D([None, None]) => Less);
    case!(A, E(vec![]) => Less);
    case!(B, A => Greater);
    case!(B, B => Equal);
    case!(B, C(' ') => Less);
    case!(B, D([None, None]) => Less);
    case!(B, E(vec![]) => Less);
    case!(C(' '), A => Greater);
    case!(C(' '), B => Greater);
    case!(C(' '), C(' ') => Equal);
    case!(C(' '), D([None, None]) => Less);
    case!(C(' '), E(vec![]) => Less);
    case!(D([None, None]), A => Greater);
    case!(D([None, None]), B => Greater);
    case!(D([None, None]), C(' ') => Greater);
    case!(D([None, None]), D([None, None]) => Equal);
    case!(D([None, None]), E(vec![]) => Equal);
    case!(E(vec![]), A => Greater);
    case!(E(vec![]), B => Greater);
    case!(E(vec![]), C(' ') => Greater);
    case!(E(vec![]), D([None, None]) => Equal);
    case!(E(vec![]), E(vec![]) => Equal);
}

#[test]
fn directly_contained()
{
    case!(C('x'), C('y') => Less);
    case!(C('z'), C('y') => Greater);

    case!(D([None, None]), D([None, Some(a())]) => Less);
    case!(D([None, None]), D([Some(a()), None]) => Less);
    case!(D([None, None]), D([Some(a()), Some(a())]) => Less);

    case!(D([None, Some(a())]), D([None, None]) => Greater);
    case!(D([None, Some(a())]), D([None, Some(a())]) => Equal);
    case!(D([None, Some(a())]), D([Some(a()), None]) => Less);
    case!(D([None, Some(a())]), D([Some(a()), Some(a())]) => Less);

    case!(D([Some(a()), None]), D([None, None]) => Greater);
    case!(D([Some(a()), None]), D([None, Some(a())]) => Greater);
    case!(D([Some(a()), None]), D([Some(a()), None]) => Equal);
    case!(D([Some(a()), None]), D([Some(a()), Some(a())]) => Less);

    case!(D([Some(a()), Some(a())]), D([None, None]) => Greater);
    case!(D([Some(a()), Some(a())]), D([None, Some(a())]) => Greater);
    case!(D([Some(a()), Some(a())]), D([Some(a()), None]) => Greater);
    case!(D([Some(a()), Some(a())]), D([Some(a()), Some(a())]) => Equal);
}

#[test]
fn amount_edges()
{
    case!(E(vec![]), E(vec![A]) => Less);
    case!(E(vec![A]), E(vec![A]) => Equal);
    case!(E(vec![A]), E(vec![]) => Greater);

    case!(E(vec![A]), E(vec![A, A]) => Less);
    case!(E(vec![A, A]), E(vec![A, A]) => Equal);
    case!(E(vec![A, A]), E(vec![A]) => Greater);

    case!(D([None, None]), E(vec![A]) => Less);
    case!(D([None, Some(a())]), E(vec![A]) => Equal);
    case!(D([Some(a()), None]), E(vec![A]) => Equal);
    case!(D([Some(a()), Some(a())]), E(vec![A, A]) => Equal);
    case!(D([None, Some(a())]), E(vec![]) => Greater);
    case!(D([Some(a()), None]), E(vec![]) => Greater);
    case!(D([Some(a()), Some(a())]), E(vec![A, A, A]) => Less);

    case!(E(vec![A]), D([None, None]) => Greater);
    case!(E(vec![A]), D([None, Some(a())]) => Equal);
    case!(E(vec![A]), D([Some(a()), None]) => Equal);
    case!(E(vec![A, A]), D([Some(a()), Some(a())]) => Equal);
    case!(E(vec![]), D([None, Some(a())]) => Less);
    case!(E(vec![]), D([Some(a()), None]) => Less);
    case!(E(vec![A, A, A]), D([Some(a()), Some(a())]) => Greater);
}

#[test]
fn descendents()
{
    case!(D([None, Some(a())]), D([None, Some(b())]) => Less);
    case!(D([None, Some(a())]), D([None, Some(c(' '))]) => Less);
    case!(D([None, Some(b())]), D([None, Some(a())]) => Greater);
    case!(D([None, Some(b())]), D([None, Some(c(' '))]) => Less);
    case!(D([None, Some(c('x'))]), D([None, Some(c('y'))]) => Less);
    case!(D([None, Some(c('z'))]), D([None, Some(c('y'))]) => Greater);

    case!(D([Some(b()), None]), D([Some(a()), None]) => Greater);
    case!(D([Some(c(' ')), None]), D([Some(a()), None]) => Greater);
    case!(D([Some(a()), None]), D([Some(b()), None]) => Less);
    case!(D([Some(c(' ')), None]), D([Some(b()), None]) => Greater);
    case!(D([Some(c('y')), None]), D([Some(c('x')), None]) => Greater);
    case!(D([Some(c('y')), None]), D([Some(c('z')), None]) => Less);

    case!(D([None, Some(d(Some(a()), Some(a())))]),
          D([None, Some(d(Some(a()), Some(a())))])
          => Equal);
    case!(D([None, Some(d(Some(a()), Some(a())))]),
          D([None, Some(d(Some(a()), Some(b())))])
          => Less);
    case!(D([None, Some(d(Some(b()), Some(a())))]),
          D([None, Some(d(Some(a()), Some(a())))])
          => Greater);
    case!(D([None, Some(d(Some(b()), Some(b())))]),
          D([None, Some(d(Some(b()), Some(a())))])
          => Greater);
    case!(D([None, Some(d(Some(a()), Some(b())))]),
          D([None, Some(d(Some(b()), Some(b())))])
          => Less);
    case!(D([None, Some(d(Some(b()), Some(b())))]),
          D([None, Some(d(Some(b()), Some(b())))])
          => Equal);

    case!(D([Some(d(Some(c('x')), None)), None]),
          D([Some(d(Some(c('x')), None)), None])
          => Equal);
    case!(D([Some(d(Some(c('x')), None)), None]),
          D([Some(d(Some(c('x')), Some(d(None, None)))), None])
          => Less);
    case!(D([Some(d(Some(c('y')), None)), None]),
          D([Some(d(Some(c('x')), Some(d(None, None)))), None])
          => Less);
    case!(D([Some(d(Some(c('y')), Some(a()))), None]),
          D([Some(d(Some(c('x')), Some(d(None, None)))), None])
          => Greater);
    case!(D([Some(d(Some(c('x')), Some(a()))), None]),
          D([Some(d(Some(c('x')), Some(d(None, None)))), None])
          => Less);


    case!(D([None, Some(a())]), E(vec![B]) => Less);
    case!(E(vec![A]), D([None, Some(c(' '))]) => Less);
    case!(D([None, Some(b())]), E(vec![A]) => Greater);
    case!(E(vec![B]), D([None, Some(c(' '))]) => Less);
    case!(D([None, Some(c('x'))]), E(vec![C('y')]) => Less);
    case!(E(vec![C('z')]), D([None, Some(c('y'))]) => Greater);

    case!(D([Some(b()), None]), E(vec![A]) => Greater);
    case!(E(vec![C(' ')]), D([Some(a()), None]) => Greater);
    case!(D([Some(a()), None]), E(vec![B]) => Less);
    case!(E(vec![C(' ')]), D([Some(b()), None]) => Greater);
    case!(D([Some(c('y')), None]), E(vec![C('x')]) => Greater);
    case!(E(vec![C('y')]), D([Some(c('z')), None]) => Less);

    case!(D([None, Some(d(Some(a()), Some(a())))]),
          E(vec![D([Some(a()), Some(a())])])
          => Equal);
    case!(E(vec![E(vec![A, A])]),
          D([None, Some(d(Some(a()), Some(b())))])
          => Less);
    case!(D([None, Some(e(vec![B, A]))]),
          E(vec![D([Some(a()), Some(a())])])
          => Greater);
    case!(E(vec![E(vec![B, B])]),
          D([None, Some(d(Some(b()), Some(a())))])
          => Greater);
    case!(D([None, Some(d(Some(a()), Some(b())))]),
          E(vec![D([Some(b()), Some(b())])])
          => Less);
    case!(E(vec![E(vec![B, B])]),
          D([None, Some(e(vec![B, B]))])
          => Equal);

    case!(D([Some(d(Some(c('x')), None)), None]),
          E(vec![D([Some(c('x')), None])])
          => Equal);
    case!(E(vec![E(vec![C('x')])]),
          D([Some(d(Some(c('x')), Some(d(None, None)))), None])
          => Less);
    case!(D([Some(d(Some(c('y')), None)), None]),
          E(vec![D([Some(c('x')), Some(d(None, None))])])
          => Less);
    case!(E(vec![D([Some(c('y')), Some(a())])]),
          D([Some(e(vec![C('x'), D([None, None])])), None])
          => Greater);
    case!(D([Some(d(Some(c('x')), Some(a()))), None]),
          E(vec![D([Some(c('x')), Some(d(None, None))])])
          => Less);
}

#[test]
fn sorting()
{
    fn compare(
        a: &Datum,
        b: &Datum,
    ) -> Ordering
    {
        case!(*a, *b => basic::equiv(&a, &b))
    }

    macro_rules! sort_case {
        ([$($pre:expr),*] => [$($post:expr),*]) => {
            let mut x = [$($pre),*];
            x.sort_by(compare);
            assert_eq!(x, [$($post),*]);
        };
    }

    sort_case!([] => []);
    sort_case!([A] => [A]);
    sort_case!([B] => [B]);
    sort_case!([C(' ')] => [C(' ')]);
    sort_case!([D([None, None])] => [D([None, None])]);
    sort_case!([E(vec![])] => [E(vec![])]);

    sort_case!([C('x'), C('y')] => [C('x'), C('y')]);
    sort_case!([C('y'), C('x')] => [C('x'), C('y')]);
    sort_case!([D([None, Some(b())]), D([Some(a()), None])]
               => [D([None, Some(b())]), D([Some(a()), None])]);
    sort_case!([D([Some(a()), None]), D([None, Some(b())])]
               => [D([None, Some(b())]), D([Some(a()), None])]);
    sort_case!([E(vec![A]), E(vec![B])] => [E(vec![A]), E(vec![B])]);
    sort_case!([E(vec![B]), E(vec![A])] => [E(vec![A]), E(vec![B])]);
    sort_case!([E(vec![B, B]), E(vec![A, A, A])] => [E(vec![B, B]), E(vec![A, A, A])]);
    sort_case!([E(vec![A, A, A]), E(vec![B, B])] => [E(vec![B, B]), E(vec![A, A, A])]);

    sort_case!([E(vec![C('z'), A]), B, A, D([Some(c('z')), Some(a())]), C('y'), C('x'), A, B]
               => [A, A, B, B, C('x'), C('y'), E(vec![C('z'), A]), D([Some(c('z')), Some(a())])]);
    sort_case!([
        D([None, Some(e(vec![E(vec![C('λ'), B, A]), D([Some(a()), None])]))]),
        E(vec![D([Some(d(Some(c('λ')), Some(b()))), Some(e(vec![A]))])]),
        D([None, Some(e(vec![E(vec![C('λ'), B]), D([Some(a()), None])]))]),
        E(vec![D([Some(d(Some(c('λ')), Some(b()))), Some(e(vec![A]))])])
    ] => [
        E(vec![D([Some(d(Some(c('λ')), Some(b()))), Some(e(vec![A]))])]),
        D([None, Some(e(vec![E(vec![C('λ'), B]), D([Some(a()), None])]))]),
        E(vec![D([Some(d(Some(c('λ')), Some(b()))), Some(e(vec![A]))])]),
        D([None, Some(e(vec![E(vec![C('λ'), B, A]), D([Some(a()), None])]))])
    ]);
}
