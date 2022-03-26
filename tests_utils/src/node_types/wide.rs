use std::{
    iter::repeat,
    ops::Index,
    rc::Rc,
};


/// Generates very many edges but uses little memory itself.
#[derive(Eq, Clone, Debug)]
pub enum Datum
{
    Leaf,
    Branch
    {
        width:      usize,
        proto_edge: Rc<Self>,
    },
}

impl Datum
{
    pub fn degenerate_chain(
        width: usize,
        depth: u32,
    ) -> Self
    {
        let mut head = Self::Leaf;

        for _ in 0 .. depth {
            head = Self::Branch { width, proto_edge: Rc::new(head) }
        }

        head
    }

    pub fn width(&self) -> usize
    {
        match self {
            Datum::Leaf => 0,
            Datum::Branch { width, .. } => *width,
        }
    }
}

impl Index<usize> for Datum
{
    type Output = Rc<Self>;

    #[inline]
    fn index(
        &self,
        index: usize,
    ) -> &Self::Output
    {
        match self {
            Datum::Branch { width, proto_edge } if index < *width => proto_edge,
            _ => panic!("out of bounds"),
        }
    }
}

/// This `PartialEq` does not implement a `graph_safe_compare` algorithm and is only used for
/// having an intentionally-naive algorithm that acts as if there are `width` amount of edges.
/// When `graph_safe_compare` algorithms are tested against this type, their functions must
/// be called directly.
impl PartialEq for Datum
{
    fn eq(
        &self,
        other: &Self,
    ) -> bool
    {
        match (self, other) {
            (Datum::Leaf, Datum::Leaf) => true,
            (
                Datum::Branch { width: aw, proto_edge: ae },
                Datum::Branch { width: bw, proto_edge: be },
            ) =>
                aw == bw
                    && repeat((Rc::clone(ae), Rc::clone(be)))
                        .take(*aw)
                        .all(|(ae, be)| Datum::eq(&ae, &be)),
            (Datum::Leaf, Datum::Branch { width: 0, .. }) => true,
            (Datum::Branch { width: 0, .. }, Datum::Leaf) => true,
            _ => false,
        }
    }
}
