use super::{CellBranch, CellBranchBounds};
use math_tools::bounded_value::BoundedValueError;
use std::ops::Add;
use std::fmt::{self, Display, Formatter};
use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Coords(pub Vec<CellBranch>);

impl Display for Coords {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0.iter().map(|branch| branch.get() + 1).collect::<Vec<usize>>())
    }
}

impl Add for &Coords {
    type Output = Coords;

    fn add(self, rhs: &Coords) -> Self::Output {
        Coords([self.0.clone(), rhs.0.clone()].concat())
    }
}

impl Coords {
    pub fn new_checked(vec: Vec<usize>) -> Result<Self, BoundedValueError<CellBranchBounds>> {
        vec.into_iter()
            .map(CellBranch::new_checked)
            .collect::<Result<Vec<CellBranch>, _>>()
            .map(Self)
    }

    pub fn new(vec: Vec<usize>) -> Self {
        Self::new_checked(vec).expect("Invalid Vec<u8> passed to Coords::new")
    }
}