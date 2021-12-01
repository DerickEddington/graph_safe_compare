pub trait Allocator<T>
{
    fn alloc(&self) -> T;
}


pub trait Leaf: Sized
{
    type Alloc: Allocator<Self>;

    fn new_in(alloc: &Self::Alloc) -> Self;

    fn new() -> Self
    where
        Self::Alloc: Default,
    {
        Self::new_in(&Self::Alloc::default())
    }
}

pub trait Pair: Leaf
{
    fn new_in(
        a: Self,
        b: Self,
        alloc: &Self::Alloc,
    ) -> Self
    {
        let x = alloc.alloc();
        x.set(a, b);
        x
    }

    fn set(
        &self,
        a: Self,
        b: Self,
    );

    fn new(
        a: Self,
        b: Self,
    ) -> Self
    where
        Self::Alloc: Default,
    {
        <Self as Pair>::new_in(a, b, &Self::Alloc::default())
    }
}


pub struct PairChainMaker<A, T>
{
    depth: usize,
    alloc: A,
    head:  T,
    tail:  T,
}

impl<T: Pair<Alloc = A> + Clone, A: Allocator<T>> PairChainMaker<A, T>
{
    pub fn new_with(
        depth: usize,
        alloc: A,
    ) -> Self
    {
        let tail = <T as Leaf>::new_in(&alloc);
        let head = tail.clone();
        Self { depth, alloc, head, tail }
    }

    pub fn new(depth: usize) -> Self
    where
        A: Default,
    {
        Self::new_with(depth, A::default())
    }

    pub fn list(self) -> T
    {
        self.chain(Self::leaf, Self::clone_head).0
    }

    pub fn inverted_list(self) -> T
    {
        self.chain(Self::clone_head, Self::leaf).0
    }

    pub fn degenerate_dag(self) -> T
    {
        self.degenerate().0
    }

    pub fn degenerate_cyclic(self) -> T
    {
        let (head, tail) = self.degenerate();
        Pair::set(&tail, head.clone(), head.clone());
        head
    }

    fn degenerate(self) -> (T, T)
    {
        self.chain(Self::clone_head, Self::clone_head)
    }

    fn chain<F1: FnMut(&mut Self) -> T, F2: FnMut(&mut Self) -> T>(
        mut self,
        mut a: F1,
        mut b: F2,
    ) -> (T, T)
    {
        while self.depth >= 1
        {
            self.depth = self.depth.saturating_sub(1);
            self.head = <T as Pair>::new_in(a(&mut self), b(&mut self), &self.alloc);
        }

        (self.head, self.tail)
    }

    fn leaf(&mut self) -> T
    {
        <T as Leaf>::new_in(&self.alloc)
    }

    fn clone_head(&mut self) -> T
    {
        self.head.clone()
    }
}
