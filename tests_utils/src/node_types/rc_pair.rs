use {
    crate::shapes::{
        Allocator,
        Leaf,
        Pair,
    },
    std::{
        cell::RefCell,
        rc::Rc,
    },
};


// Note that this derived PartialEq does not implement a `graph_safe_compare` algorithm and
// is only used for demonstrating the limitations of the derived algorithm.  When
// `graph_safe_compare` algorithms are tested against this type, their functions must be
// called directly.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Datum(pub RefCell<Option<(Rc<Self>, Rc<Self>)>>);

impl Leaf for Rc<Datum>
{
    type Alloc = DatumAllocator;

    fn new_in(alloc: &Self::Alloc) -> Self
    {
        alloc.alloc()
    }
}

impl Pair for Rc<Datum>
{
    fn set(
        &self,
        a: Self,
        b: Self,
    )
    {
        *self.0.borrow_mut() = Some((a, b));
    }

    fn take(&self) -> Option<(Self, Self)>
    {
        self.0.replace(None)
    }
}


pub struct DatumAllocator;

impl Allocator<Rc<Datum>> for DatumAllocator
{
    fn alloc(&self) -> Rc<Datum>
    {
        Rc::new(Datum(RefCell::new(None)))
    }
}

impl DatumAllocator
{
    pub fn new(_size: u32) -> Self
    {
        Self::default()
    }
}

impl Default for DatumAllocator
{
    fn default() -> Self
    {
        Self
    }
}
