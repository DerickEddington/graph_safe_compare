use std::{
    cell::RefCell,
    rc::Rc,
};

// Note that these derived PartialEq implementations do not do a `cycle_deep_safe_compare`
// algorithm and are only used for demonstrating the limitations of the derived algorithm.  When
// `cycle_deep_safe_compare` algorithms are tested against this type, their functions must be
// called directly.

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Datum1
{
    pub child: Option<Rc<Datum2>>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Datum2
{
    Double(Rc<Datum3>, Rc<Datum3>),
    Triple(Rc<Datum3>, Rc<Datum3>, Rc<Datum3>),
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Datum3(pub RefCell<Datum4>);

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Datum4
{
    End,
    Link(Rc<Datum1>),
}
