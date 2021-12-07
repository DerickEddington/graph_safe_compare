use {
    crate::shapes::{
        Allocator,
        Leaf,
        Pair,
    },
    std::{
        any::Any,
        cell::{
            Cell,
            Ref,
            RefCell,
            RefMut,
        },
        rc::Rc,
    },
};


pub enum Datum1
{
    Empty,
    Double(DatumRef, DatumRef),
}

pub enum Datum2<T>
{
    Value(T),
    Two(DatumRef, DatumRef),
    Four(DatumRef, DatumRef, DatumRef, DatumRef),
}


#[derive(Clone, Debug)]
pub struct DatumRef(pub Rc<RefCell<dyn Any>>);

pub enum DowncastDatum<'l>
{
    Datum1(Ref<'l, Datum1>),
    Datum2Int32(Ref<'l, Datum2<i32>>),
    Datum2Char(Ref<'l, Datum2<char>>),
}

pub enum DowncastMutDatum<'l>
{
    Datum1(RefMut<'l, Datum1>),
    Datum2Int32(RefMut<'l, Datum2<i32>>),
    Datum2Char(RefMut<'l, Datum2<char>>),
}

impl DatumRef
{
    pub fn downcast(&self) -> DowncastDatum<'_>
    {
        fn downcast_ref<U: 'static>(a: &dyn Any) -> &U
        {
            a.downcast_ref().unwrap()
        }

        let b = self.0.borrow();

        if b.is::<Datum1>() {
            DowncastDatum::Datum1(Ref::map(b, downcast_ref))
        }
        else if b.is::<Datum2<i32>>() {
            DowncastDatum::Datum2Int32(Ref::map(b, downcast_ref))
        }
        else if b.is::<Datum2<char>>() {
            DowncastDatum::Datum2Char(Ref::map(b, downcast_ref))
        }
        else {
            unreachable!();
        }
    }

    pub fn downcast_mut(&self) -> DowncastMutDatum<'_>
    {
        fn downcast_mut<U: 'static>(a: &mut dyn Any) -> &mut U
        {
            a.downcast_mut().unwrap()
        }

        let b = self.0.borrow_mut();

        if b.is::<Datum1>() {
            DowncastMutDatum::Datum1(RefMut::map(b, downcast_mut))
        }
        else if b.is::<Datum2<i32>>() {
            DowncastMutDatum::Datum2Int32(RefMut::map(b, downcast_mut))
        }
        else if b.is::<Datum2<char>>() {
            DowncastMutDatum::Datum2Char(RefMut::map(b, downcast_mut))
        }
        else {
            unreachable!();
        }
    }
}

// Note that this PartialEq impl does not implement a `cycle_deep_safe_compare` algorithm and is
// only used for demonstrating the limitations of a naive algorithm.  When
// `cycle_deep_safe_compare` algorithms are tested against this type, their functions must be
// called directly.
impl PartialEq for DatumRef
{
    fn eq(
        &self,
        other: &Self,
    ) -> bool
    {
        match (self.downcast(), other.downcast()) {
            (DowncastDatum::Datum1(d1a), DowncastDatum::Datum1(d1b)) => match (&*d1a, &*d1b) {
                (Datum1::Empty, Datum1::Empty) => true,
                (Datum1::Double(aa, ab), Datum1::Double(ba, bb)) => aa == ba && ab == bb,
                _ => false,
            },
            (DowncastDatum::Datum2Int32(d2a), DowncastDatum::Datum2Int32(d2b)) => {
                match (&*d2a, &*d2b) {
                    (Datum2::Value(a), Datum2::Value(b)) => a == b,
                    (Datum2::Two(aa, ab), Datum2::Two(ba, bb)) => aa == ba && ab == bb,
                    (Datum2::Four(aa, ab, ac, ad), Datum2::Four(ba, bb, bc, bd)) =>
                        aa == ba && ab == bb && ac == bc && ad == bd,
                    _ => false,
                }
            },
            (DowncastDatum::Datum2Char(d2a), DowncastDatum::Datum2Char(d2b)) => {
                match (&*d2a, &*d2b) {
                    (Datum2::Value(a), Datum2::Value(b)) => a == b,
                    (Datum2::Two(aa, ab), Datum2::Two(ba, bb)) => aa == ba && ab == bb,
                    (Datum2::Four(aa, ab, ac, ad), Datum2::Four(ba, bb, bc, bd)) =>
                        aa == ba && ab == bb && ac == bc && ad == bd,
                    _ => false,
                }
            },
            _ => false,
        }
    }
}
impl Eq for DatumRef {}

impl Leaf for DatumRef
{
    type Alloc = DatumAllocator;

    fn new_in(alloc: &Self::Alloc) -> Self
    {
        alloc.alloc()
    }
}

impl Pair for DatumRef
{
    fn set(
        &self,
        a: Self,
        b: Self,
    )
    {
        match self.downcast_mut() {
            DowncastMutDatum::Datum1(mut rd1) => *rd1 = Datum1::Double(a, b),
            DowncastMutDatum::Datum2Int32(mut rd2) => *rd2 = Datum2::Two(a, b),
            DowncastMutDatum::Datum2Char(mut rd2) => *rd2 = Datum2::Two(a, b),
        }
    }
}


pub struct DatumAllocator
{
    counter: Cell<usize>,
}

impl DatumAllocator
{
    pub fn new(_size: u32) -> Self
    {
        Self { counter: Cell::new(0) }
    }
}

impl Allocator<DatumRef> for DatumAllocator
{
    fn alloc(&self) -> DatumRef
    {
        self.counter.set(self.counter.get() + 1);

        match self.counter.get() % 3 {
            0 => DatumRef(Rc::new(RefCell::new(Datum1::Empty))),
            1 => DatumRef(Rc::new(RefCell::new(Datum2::Value(42_i32)))),
            2 => DatumRef(Rc::new(RefCell::new(Datum2::Value('λ')))),
            _ => unreachable!(),
        }
    }
}
