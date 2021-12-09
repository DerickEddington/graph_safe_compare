//! TODO

use super::{
    CallStack,
    Equiv,
    Node,
    Recur,
    Reset,
};


// TODO: Maybe this should be much larger, since VecStack will be used when large depth is
// expected.  Maybe the user should be able to pass a value thru the API for this instead.
const INIT_CAPACITY: usize = 32;


pub struct VecStack<N>(Vec<(N, N)>);

impl<N> Default for VecStack<N>
{
    fn default() -> Self
    {
        Self(Vec::with_capacity(INIT_CAPACITY))
    }
}

impl<N> Reset for VecStack<N>
{
    /// The aborted precheck phase might have left some elements, so we must reset before doing
    /// the interleave phase.
    fn reset(mut self) -> Self
    {
        self.0.clear();
        self
    }
}

/// Enables the call-stack to be used for the precheck phase and the vector-stack for the
/// interleave phase.
impl<N> From<CallStack> for VecStack<N>
{
    fn from(_: CallStack) -> Self
    {
        Self::default()
    }
}


impl<N: Node, P> Recur for Equiv<P, VecStack<N>>
{
    type Node = N;

    fn recur(
        &mut self,
        a: Self::Node,
        b: Self::Node,
    ) -> Result<bool, ()>
    {
        self.recur_stack.0.push((a, b));
        Ok(true)
    }

    fn next(&mut self) -> Option<(N, N)>
    {
        self.recur_stack.0.pop()
    }
}
