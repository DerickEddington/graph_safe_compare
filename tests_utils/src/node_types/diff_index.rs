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

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub enum Inner
{
    #[default]
    Leaf,
    Pair(Datum, Datum),
}

type Region = Rc<[RefCell<Inner>]>;

#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Index
{
    #[default]
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
}

impl Index
{
    pub const MAX: Self = Index::Seven;
    pub const MIN: Self = Index::Zero;

    #[inline]
    pub fn increment(&self) -> Option<Self>
    {
        match self {
            Index::Zero => Some(Index::One),
            Index::One => Some(Index::Two),
            Index::Two => Some(Index::Three),
            Index::Three => Some(Index::Four),
            Index::Four => Some(Index::Five),
            Index::Five => Some(Index::Six),
            Index::Six => Some(Index::Seven),
            Index::Seven => None,
        }
    }

    #[inline]
    pub fn decrement(&self) -> Option<Self>
    {
        match self {
            Index::Zero => None,
            Index::One => Some(Index::Zero),
            Index::Two => Some(Index::One),
            Index::Three => Some(Index::Two),
            Index::Four => Some(Index::Three),
            Index::Five => Some(Index::Four),
            Index::Six => Some(Index::Five),
            Index::Seven => Some(Index::Six),
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
        self.next.set(index.increment());
        Datum { index, region: Rc::clone(&self.region) }
    }
}
