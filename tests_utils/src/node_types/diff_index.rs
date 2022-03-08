use {
    crate::shapes::{
        Allocator,
        Leaf,
        Pair,
    },
    cfg_if::cfg_if,
    std::{
        cell::{
            Cell,
            Ref,
            RefCell,
            RefMut,
        },
        ops::{
            AddAssign,
            SubAssign,
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

impl From<u8> for Index
{
    fn from(n: u8) -> Self
    {
        match n {
            0 => Index::Zero,
            1 => Index::One,
            2 => Index::Two,
            3 => Index::Three,
            4 => Index::Four,
            5 => Index::Five,
            6 => Index::Six,
            7 => Index::Seven,
            _ => panic!("invalid"),
        }
    }
}

impl AddAssign for Index
{
    fn add_assign(
        &mut self,
        rhs: Self,
    )
    {
        *self = Index::from((*self as u8).saturating_add(rhs as u8));
    }
}

impl SubAssign for Index
{
    fn sub_assign(
        &mut self,
        rhs: Self,
    )
    {
        *self = Index::from((*self as u8).saturating_sub(rhs as u8));
    }
}

#[rustfmt::skip] // This unusual formatting preserves lines for cleaner diffs.
cfg_if! {
if #[cfg(rust_lib_feature = "step_trait")]
{
use core::iter::Step;

impl Step for Index
{
    fn steps_between(
        start: &Self,
        end: &Self,
    ) -> Option<usize>
    {
        <u8 as Step>::steps_between(&(*start as u8), &(*end as u8))
    }

    fn forward_checked(
        start: Self,
        count: usize,
    ) -> Option<Self>
    {
        <u8 as Step>::forward_checked(start as u8, count).map(Self::from)
    }

    fn backward_checked(
        start: Self,
        count: usize,
    ) -> Option<Self>
    {
        <u8 as Step>::backward_checked(start as u8, count).map(Self::from)
    }
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
    next:   Cell<u8>,
}

impl DatumAllocator
{
    pub fn new(size: u32) -> Self
    {
        assert!(size <= 8);
        let size = size.try_into().unwrap();
        let mut vec = Vec::with_capacity(size);
        vec.resize(size, RefCell::new(Inner::default()));
        Self { region: vec.into(), next: Cell::new(0) }
    }
}

impl Allocator<Datum> for DatumAllocator
{
    fn alloc(&self) -> Datum
    {
        let i = self.next.get();
        self.next.set(i + 1);
        Datum { index: i.into(), region: Rc::clone(&self.region) }
    }
}
