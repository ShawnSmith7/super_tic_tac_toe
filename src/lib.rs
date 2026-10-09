pub mod canvas;
pub mod board;
pub mod game;

use bounded_value::BoundedValue;
use math_tools::{bounded_value, define_bounds};

use std::ops::Bound::*;

define_bounds!(DegreeBounds, u8, Included(0), Unbounded);
pub type Degree = BoundedValue<DegreeBounds>;

define_bounds!(UserDegreeBounds, u8, Included(1), Unbounded);
pub type UserDegree = BoundedValue<UserDegreeBounds>;