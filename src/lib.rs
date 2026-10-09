pub mod canvas;
pub mod board;
pub mod game;

use bounded_value::BoundedValue;
use math_tools::{bounded_value, define_bounds};

use std::ops::Bound::*;

define_bounds!(LevelBounds, u8, Included(0), Unbounded);
pub type Level = BoundedValue<LevelBounds>;

define_bounds!(UserLevelBounds, u8, Included(1), Unbounded);
pub type UserLevel = BoundedValue<UserLevelBounds>;