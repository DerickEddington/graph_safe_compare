use std::{
    cell::RefCell,
    rc::Rc,
};

#[derive(Clone, Debug)]
pub struct Datum1
{
    pub child: Option<Rc<Datum2>>,
}

#[derive(Clone, Debug)]
pub enum Datum2
{
    Double(Rc<Datum3>, Rc<Datum3>),
    Triple(Rc<Datum3>, Rc<Datum3>, Rc<Datum3>),
}

#[derive(Clone, Debug)]
pub struct Datum3(pub RefCell<Datum4>);

#[derive(Clone, Debug)]
pub enum Datum4
{
    End,
    Link(Rc<Datum1>),
}
