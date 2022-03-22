use {
    crate::shapes::{
        Allocator,
        Leaf,
        Pair,
    },
    std::{
        cell::{
            Cell,
            Ref,
            RefCell,
            RefMut,
        },
        rc::Rc,
    },
};


#[derive(Clone, Debug)]
pub struct Datum
{
    pub index:  Index,
    pub region: Region,
}

impl Datum
{
    pub fn deref(&self) -> Ref<'_, Inner>
    {
        self.region[self.index as usize].borrow()
    }

    pub fn deref_mut(&self) -> RefMut<'_, Inner>
    {
        self.region[self.index as usize].borrow_mut()
    }
}

// Note that this PartialEq impl does not implement a `graph_safe_compare` algorithm and is
// only used for demonstrating the limitations of a naive algorithm.  When
// `graph_safe_compare` algorithms are tested against this type, their functions must be
// called directly.
impl PartialEq for Datum
{
    fn eq(
        &self,
        other: &Self,
    ) -> bool
    {
        *self.deref() == *other.deref()
    }
}
impl Eq for Datum {}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Inner
{
    Leaf,
    Pair(Datum, Datum),
}

impl Default for Inner
{
    fn default() -> Self
    {
        Inner::Leaf
    }
}

type Region = Rc<[RefCell<Inner>]>;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Index
{
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}

impl Default for Index
{
    fn default() -> Self
    {
        Self::Zero
    }
}

impl Index
{
    pub fn increment(&self) -> Self
    {
        match self {
            Index::Zero => Index::One,
            Index::One => Index::Two,
            Index::Two => Index::Three,
            Index::Three => Index::Four,
            Index::Four => Index::Five,
            Index::Five => Index::Six,
            Index::Six => Index::Seven,
            Index::Seven => panic!(),
        }
    }
}

impl Leaf for Datum
{
    type Alloc = DatumAllocator;

    fn new_in(alloc: &Self::Alloc) -> Self
    {
        alloc.alloc()
    }
}

impl Pair for Datum
{
    fn set(
        &self,
        a: Self,
        b: Self,
    )
    {
        *self.deref_mut() = Inner::Pair(a, b);
    }

    fn take(&self) -> Option<(Self, Self)>
    {
        let val = std::mem::replace(&mut *self.deref_mut(), Inner::Leaf);
        match val {
            Inner::Leaf => None,
            Inner::Pair(a, b) => Some((a, b)),
        }
    }
}


pub struct DatumAllocator
{
    region: Region,
    next:   Cell<Option<Index>>,
}

impl DatumAllocator
{
    pub fn new(size: u32) -> Self
    {
        assert!(size <= 8);
        let size = size.try_into().unwrap();
        let mut vec = Vec::with_capacity(size);
        vec.resize(size, RefCell::new(Inner::default()));
        Self { region: vec.into(), next: Cell::new(Some(Index::Zero)) }
    }
}

impl Allocator<Datum> for DatumAllocator
{
    fn alloc(&self) -> Datum
    {
        let index = self.next.get().unwrap();
        self.next.set((index < Index::Seven).then(|| index.increment()));
        Datum { index, region: Rc::clone(&self.region) }
    }
}
