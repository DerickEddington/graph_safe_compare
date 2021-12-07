use {
    crate::shapes::{
        Allocator,
        Leaf,
        Pair,
    },
    std::cell::{
        Cell,
        RefCell,
    },
};


// Note that this derived PartialEq does not implement a `cycle_deep_safe_compare` algorithm and
// is only used for demonstrating the limitations of the derived algorithm.  When
// `cycle_deep_safe_compare` algorithms are tested against this type, their functions must be
// called directly.
#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub struct Datum<'l>(pub RefCell<Inner<'l>>);

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Inner<'l>
{
    Leaf,
    Pair(&'l Datum<'l>, &'l Datum<'l>),
}

impl Default for Inner<'_>
{
    fn default() -> Self
    {
        Self::Leaf
    }
}

impl<'l> Leaf for &'l Datum<'l>
{
    type Alloc = &'l DatumAllocator<Datum<'l>>;

    fn new_in(alloc: &Self::Alloc) -> Self
    {
        let datum_ref = alloc.alloc();
        *datum_ref.0.borrow_mut() = Inner::Leaf;
        datum_ref
    }
}

impl<'l> Pair for &'l Datum<'l>
{
    fn set(
        &self,
        a: Self,
        b: Self,
    )
    {
        *self.0.borrow_mut() = Inner::Pair(a, b);
    }
}


pub struct DatumAllocator<D>
{
    slice: Box<[D]>,
    next:  Cell<u32>,
}

impl<D: Default> DatumAllocator<D>
{
    pub fn new(size: u32) -> Self
    {
        let size = size.try_into().unwrap();
        let mut vec = Vec::with_capacity(size);
        vec.resize_with(size, D::default);
        Self { slice: vec.into_boxed_slice(), next: Cell::new(0) }
    }
}

impl<'a, D> Allocator<&'a D> for &'a DatumAllocator<D>
{
    fn alloc(&self) -> &'a D
    {
        let i = self.next.get();
        self.next.set(i + 1);
        let i: usize = i.try_into().unwrap();
        &self.slice[i]
    }
}
