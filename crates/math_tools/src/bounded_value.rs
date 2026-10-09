use std::{fmt, marker, ops, str};
use fmt::{Debug, Display, Formatter};
use marker::PhantomData;
use ops::{Add, Bound, Div, Mul, Sub};
use str::FromStr;
use terminal_tools::ParseError;
use serde::{Serialize, Deserialize};
use Bound::*;
use BoundedValueError::*;

pub trait Bounds {
    type Val: PartialOrd + Clone;
    const LOWER: Bound<Self::Val>;
    const UPPER: Bound<Self::Val>;

    fn contains(val: &Self::Val) -> bool {
        let lower_ok = match &Self::LOWER {
            Included(lower) => val >= lower,
            Excluded(lower) => val > lower,
            Unbounded => true,
        };

        let upper_ok = match &Self::UPPER {
            Included(upper) => val <= upper,
            Excluded(upper) => val < upper,
            Unbounded => true,
        };

        lower_ok && upper_ok
    }

    fn display() -> BoundsFormatter<Self>
    where
        Self: Sized,
    {
        BoundsFormatter(PhantomData)
    }
}

pub struct BoundsFormatter<B: Bounds>(PhantomData<B>);

impl<B: Bounds> Display for BoundsFormatter<B>
where
    B::Val: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &B::LOWER {
            Included(lower) => write!(f, "{lower}")?,
            Excluded(lower) => write!(f, "{lower}<")?,
            Unbounded => (),
        }

        write!(f, "..")?;

        match &B::UPPER {
            Included(upper) => write!(f, "={upper}"),
            Excluded(upper) => write!(f, "{upper}"),
            Unbounded => Ok(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundedValue<B: Bounds>(B::Val);

impl<B: Bounds> Default for BoundedValue<B>
where
    B::Val: Default,
{
    fn default() -> Self {
        match B::LOWER {
            Included(val) => Self(val),
            Excluded(_) => panic!("Excluded bounds do not have a default value"),
            Unbounded => Self(B::Val::default()),
        }
    }
}

impl<B: Bounds> FromStr for BoundedValue<B>
where
    B::Val: FromStr,
{
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new_checked(s.parse()
            .map_err(|_| ParseError(s.to_string()))?)
            .map_err(|_| ParseError(s.to_string()))
    }
}

impl<B: Bounds> BoundedValue<B> {
    pub fn new_checked(val: B::Val) -> Result<Self, BoundedValueError<B>> {
        if !B::contains(&val) {
            return Err(OutOfBounds(val));
        }

        Ok(Self(val))
    }
    
    pub fn new(val: B::Val) -> Self
    where
        B: Debug,
        B::Val: Debug,
    {
        Self::new_checked(val).expect("Invalid val passed to BoundedValue::new")
    }
    
    pub unsafe fn new_unchecked(val: B::Val) -> Self {
        Self(val)
    }

    pub fn get(&self) -> &B::Val {
        &self.0
    }

    pub fn into_inner(self) -> B::Val {
        self.0
    }
}

impl<B: Bounds> Display for BoundedValue<B>
where
    B::Val: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<B, C, T> TryFrom<&BoundedValue<B>> for BoundedValue<C>
where
    T: PartialOrd + Copy + Sub<Output = T> + Add<Output = T> + Mul<Output = T> + Div<Output = T>,
    B: Bounds<Val = T>,
    C: Bounds<Val = T>,
{
    type Error = BoundedValueError<C>;

    fn try_from(val: &BoundedValue<B>) -> Result<Self, BoundedValueError<C>> {
        match (&B::LOWER, &B::UPPER, &C::LOWER, &C::UPPER) {
            (
                Included(b_lower),
                Included(b_upper),
                Included(c_lower),
                Included(c_upper),
            ) => {
                let b_range = *b_upper - *b_lower;
                let c_range = *c_upper - *c_lower;
                let val_offset = *val.get() - *b_lower;

                let scaled = (val_offset * c_range) / b_range + *c_lower;
                Self::new_checked(scaled)
            }
            _ => Err(IncludedBoundsRequired),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundedValueError<B: Bounds> {
    OutOfBounds(B::Val),
    IncludedBoundsRequired,
}

impl<B: Bounds> Display for BoundedValueError<B>
where
    B::Val: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OutOfBounds(val) => write!(f, "{val} is out of bounds {}", B::display()),
            IncludedBoundsRequired => write!(f, "Included bounds are required for translation"),
        }
    }
}

impl<B: Bounds + Debug> std::error::Error for BoundedValueError<B> where B::Val: Debug + Display {}

#[macro_export]
macro_rules! define_bounds {
    ($name:ident, $type:ty, $lower:expr, $upper:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
        pub struct $name;

        impl $crate::bounded_value::Bounds for $name {
            type Val = $type;
            const LOWER: std::ops::Bound<Self::Val> = $lower;
            const UPPER: std::ops::Bound<Self::Val> = $upper;
        }
    };
}

define_bounds!(DigitBounds, u8, Included(0), Included(9));
pub type Digit = BoundedValue<DigitBounds>;

#[cfg(test)]
mod tests {
    use super::*;

    define_bounds!(TranslatedBounds, u8, Included(10), Included(29));
    type Translated = BoundedValue<TranslatedBounds>;

    define_bounds!(UnboundedBounds, u8, Unbounded, Unbounded);
    type Unbounded = BoundedValue<UnboundedBounds>;

    #[test]
    fn test() {
        assert!(Digit::new_checked(0).is_ok());
        assert!(Digit::new_checked(9).is_ok());
        assert!(Digit::new_checked(10).is_err());

        let digit = Digit::new_checked(5).unwrap();
        assert_eq!(*digit.get(), 5);

        let translated = Translated::try_from(&digit).unwrap();
        assert_eq!(translated.into_inner(), 20);
        assert!(Unbounded::try_from(&digit).is_err());
    }
}