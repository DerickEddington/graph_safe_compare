//! TODO

use {
    super::{
        Equiv,
        EquivClasses,
        EquivControl,
        Node,
        PRE_LIMIT,
    },
    std::ops::ControlFlow,
};


struct NextStack<N>(Vec<(N, N)>);

impl<N> NextStack<N>
{
    fn new() -> Self
    {
        Self(Vec::new())
    }

    #[allow(clippy::unnecessary_wraps)]
    fn recur_on_vec(
        &mut self,
        a: N,
        b: N,
    ) -> Result<bool, ()>
    {
        self.0.push((a, b));
        Ok(true)
    }

    fn next(&mut self) -> Option<(N, N)>
    {
        self.0.pop()
    }
}


pub fn precheck_interleave_equiv<N: Node>(
    a: &N,
    b: &N,
) -> bool
{
    match precheck(a, b, PRE_LIMIT.into()) {
        ControlFlow::Break(result) => result,
        ControlFlow::Continue(()) => interleave(a, b, -1),
    }
}


fn precheck<N: Node>(
    a: &N,
    b: &N,
    limit: i32,
) -> ControlFlow<bool, ()>
{
    struct Precheck<N>(NextStack<N>);

    impl<N: Node> EquivControl for Equiv<Precheck<N>>
    {
        type Node = N;

        fn do_descend(
            &mut self,
            _a: &N,
            _b: &N,
        ) -> ControlFlow<(), bool>
        {
            self.do_descend_above_limit()
        }

        fn recur(
            &mut self,
            a: N,
            b: N,
        ) -> Result<bool, ()>
        {
            self.state.0.recur_on_vec(a, b)
        }

        fn next(&mut self) -> Option<(N, N)>
        {
            self.state.0.next()
        }
    }

    let mut e = Equiv::new(limit, Precheck(NextStack::<N>::new()));

    e.equiv(a, b).map_or(ControlFlow::Continue(()), ControlFlow::Break)
}


fn interleave<N: Node>(
    a: &N,
    b: &N,
    limit: i32,
) -> bool
{
    struct Interleave<N: Node>
    {
        equiv_classes: EquivClasses<N::Id>,
        next_stack:    NextStack<N>,
    }

    impl<N: Node> AsMut<EquivClasses<N::Id>> for Interleave<N>
    {
        fn as_mut(&mut self) -> &mut EquivClasses<N::Id>
        {
            &mut self.equiv_classes
        }
    }

    impl<N: Node> EquivControl for Equiv<Interleave<N>>
    {
        type Node = N;

        fn do_descend(
            &mut self,
            a: &N,
            b: &N,
        ) -> ControlFlow<(), bool>
        {
            self.do_descend_slow_or_fast(a, b)
        }

        fn recur(
            &mut self,
            a: N,
            b: N,
        ) -> Result<bool, ()>
        {
            self.state.next_stack.recur_on_vec(a, b)
        }

        fn next(&mut self) -> Option<(N, N)>
        {
            self.state.next_stack.next()
        }
    }

    let mut e = Equiv::new(limit, Interleave::<N> {
        equiv_classes: EquivClasses::new(),
        next_stack:    NextStack::new(),
    });
    matches!(e.equiv(a, b), Ok(true))
}
