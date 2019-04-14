//! Number

use std::borrow::Cow;
use std::ops::Deref;
use std::str::FromStr;

use nom::types::CompleteStr;
use nom::IResult;
use nom::{alt, char, digit, opt, pair, tuple};

use crate::AsOwned;

#[derive(Debug, Clone, Eq, Hash)]
pub struct Number<'a> {
    /// The original input number literal
    input: Cow<'a, str>,
    /// Whether the number is positive
    positive: bool,
    /// The whole number part of the number
    whole: Option<Cow<'a, str>>,
    /// The fraction (decimal) part of the number
    fraction: Option<Cow<'a, str>>,
    /// Whether an exponent is present
    exponent: Option<Exponent<'a>>,
}

impl<'a> Number<'a> {
    fn new(
        input: Cow<'a, str>,
        positive: bool,
        whole: Option<Cow<'a, str>>,
        fraction: Option<Cow<'a, str>>,
        exponent: Option<Exponent<'a>>,
    ) -> Self {
        Self {
            input,
            positive,
            whole,
            fraction,
            exponent,
        }
    }

    /// Is signed integer
    pub fn is_signed(&self) -> bool {
        self.fraction.is_none() && self.exponent.is_none()
    }

    /// Is unsigned integer
    pub fn is_unsigned(&self) -> bool {
        self.is_signed() && self.positive
    }

    /// Is a float
    pub fn is_float(&self) -> bool {
        !self.is_signed()
    }
}

macro_rules! from_uint {
    ($($from:ty )*) => {$(
        impl<'a> From<$from> for Number<'a> {
            fn from(n: $from) -> Self {
                let input = Cow::Owned(n.to_string());
                let whole = Some(Cow::Owned(n.to_string()));
                Self::new(input, true, whole, None, None)
            }
        }
    )*};
}

macro_rules! from_int {
    ($($from:ty )*) => {$(
        impl<'a> From<$from> for Number<'a> {
            fn from(n: $from) -> Self {
                let input = Cow::Owned(n.to_string());
                let whole = Some(Cow::Owned(n.abs().to_string()));
                Self::new(input, n >= 0, whole, None, None)
            }
        }
    )*};
}

from_uint!(u8 u16 u32 u64 u128);
from_int!(i8 i16 i32 i64 i128);

macro_rules! from_float {
    ($($from:ty )*) => {$(
        impl<'a> From<$from> for Number<'a> {
            fn from(n: $from) -> Self {
                let string = if n >= 0.0 {
                    n.to_string()
                } else {
                    (n*-1.0).to_string()
                };
                let mut parts = string.split(".");
                let whole = parts.next().map(|s| Cow::Owned(s.to_string()));
                let fraction = parts.next().map(|s| Cow::Owned(s.to_string()));
                Self::new(Cow::Owned(n.to_string()), n >= 0.0, whole, fraction, None)
            }
        }
    )*};
}

from_float!(f32 f64);

macro_rules! to_number {
    ($($name:ident => $to:ty, )*) => {$(
        to_number!($name => $to => stringify!(Attempt conversion to $to));
    )*};
    ($name:ident => $to:ty => $doc:expr) => {
        #[doc=$doc]
        pub fn $name(&self) -> Result<$to, <$to as FromStr>::Err> {
            self.input.as_ref().parse()
        }
    };
}

impl<'a> Number<'a> {
    to_number!(
        as_u8 => u8,
        as_u16 => u16,
        as_u32 => u32,
        as_u64 => u64,
        as_u128 => u128,
        as_i8 => i8,
        as_i16 => i16,
        as_i32 => i32,
        as_i64 => i64,
        as_i128 => i128,
        as_f32 => f32,
        as_f64 => f64,
    );
}

impl<'a> Deref for Number<'a> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.input.as_ref()
    }
}

impl<'a> AsOwned for Number<'a> {
    type Output = Number<'static>;

    fn as_owned(&self) -> Self::Output {
        Number {
            input: Cow::Owned(self.input.to_string()),
            positive: self.positive,
            whole: self.whole.as_ref().map(|s| Cow::Owned(s.to_string())),
            fraction: self.fraction.as_ref().map(|s| Cow::Owned(s.to_string())),
            exponent: self.exponent.as_owned(),
        }
    }
}

impl<'a> std::cmp::PartialEq for Number<'a> {
    fn eq(&self, other: &Self) -> bool {
        if self.is_float() || other.is_float() {
            // Good enough comparison
            // From https://users.rust-lang.org/t/assert-eq-for-float-numbers/7034/4
            let us = self.as_f64();
            let them = other.as_f64();
            if us.is_err() || them.is_err() {
                println!("Error converting to float!");
                return false;
            }

            let us = us.unwrap().abs();
            let them = them.unwrap().abs();
            let diff = (us - them).abs();

            if us == them {
                // Handle infinities.
                true
            } else if us == 0.0 || them == 0.0 || diff < std::f64::MIN_POSITIVE {
                // One of a or b is zero (or both are extremely close to it,) use absolute error.
                diff < (std::f64::EPSILON * std::f64::MIN_POSITIVE)
            } else {
                // Use relative error.
                (diff / f64::min(us + them, std::f64::MAX)) < std::f64::EPSILON
            }
        } else if self.is_unsigned() && other.is_unsigned() {
            // Unsigned
            let us = self.as_u128();
            let them = other.as_u128();
            if us.is_err() || them.is_err() {
                println!("Error converting to unsigned!");
                return false;
            }
            us.unwrap().eq(&them.unwrap())
        } else {
            // Signed
            let us = self.as_i128();
            let them = other.as_i128();
            if us.is_err() || them.is_err() {
                println!("Error converting to signed!");
                return false;
            }
            us.unwrap().eq(&them.unwrap())
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct Exponent<'a> {
    /// Whether the exponent is positive
    pub positive: bool,
    /// The number part of the exponent
    pub exponent: Cow<'a, str>,
}

impl<'a> AsOwned for Exponent<'a> {
    type Output = Exponent<'static>;

    fn as_owned(&self) -> Self::Output {
        Exponent {
            positive: self.positive,
            exponent: Cow::Owned(self.exponent.to_string()),
        }
    }
}

pub fn number<'a>(s: CompleteStr<'a>) -> IResult<CompleteStr<'a>, Number<'a>, u32> {
    use nom::InputTake;

    let (input, positive) = opt!(s, alt!(char!('+') | char!('-')))?;
    let positive = match positive {
        None => true,
        Some('+') => true,
        Some('-') => false,
        _ => unreachable!("bug in number sign parsing"),
    };

    let (input, (whole, fraction)) = alt!(
        input,
        tuple!(digit, opt!(pair!(char!('.'), opt!(digit)))) => { |(digit, decimals )| {
            let decimals = match decimals {
                None => None,
                Some((_, None)) => Some(CompleteStr("")),
                Some((_, Some(decimals))) => Some(decimals)
            };
            (Some(digit), decimals)
        } }
        | tuple!(char!('.'), digit) => { |(_, decimals)| (None, Some(decimals))  }
    )?;

    let (remaining, exponent) = exponent(input)?;

    let input = s.take(s.len() - remaining.len());
    let number = Number::new(
        Cow::Borrowed(input.0),
        positive,
        whole.map(|w| Cow::Borrowed(w.0)),
        fraction.map(|f| Cow::Borrowed(f.0)),
        exponent,
    );

    Ok((remaining, number))
}

fn exponent<'a>(input: CompleteStr<'a>) -> IResult<CompleteStr<'a>, Option<Exponent<'a>>, u32> {
    let (remaining, exponent) = opt!(
        input,
        tuple!(
            alt!(char!('e') | char!('E')),
            opt!(alt!(char!('+') | char!('-'))),
            digit
        )
    )?;

    Ok((
        remaining,
        exponent.map(|(_, sign, exponent)| {
            let positive = match sign {
                None => true,
                Some('+') => true,
                Some('-') => false,
                _ => unreachable!("bug in number sign parsing"),
            };
            Exponent {
                positive,
                exponent: Cow::Borrowed(exponent.0),
            }
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_parsed_correctly() {
        let cases = [
            "+3.14",
            "3.14",
            "-3.14",
            "0",
            "0.0",
            "1.",
            ".789",
            "-.5",
            "1e7",
            "-1E-7",
            ".3e-2",
            "1.e4",
            "1.2e4",
            "-1.234E-12",
            "-1.234e-12",
        ];

        for case in cases.iter() {
            println!("Testing {}", case);

            let (remaining, parsed) = number(CompleteStr(*case)).unwrap();
            assert!(remaining.is_empty());

            let expected_int: Result<i64, _> = case.parse();
            let actual_int = parsed.as_i64();
            assert_eq!(expected_int, actual_int);
            if let Ok(int) = expected_int {
                let test_from = Number::from(int);
                assert_eq!(parsed, test_from);
            }

            let expected_uint: Result<u64, _> = case.parse();
            let actual_uint = parsed.as_u64();
            assert_eq!(expected_uint, actual_uint);
            if let Ok(int) = expected_uint {
                let test_from = Number::from(int);
                assert_eq!(parsed, test_from);
            }

            let expected_f64: Result<f64, _> = case.parse();
            let actual_f64 = parsed.as_f64();
            assert_eq!(expected_f64, actual_f64);
            if let Ok(f) = expected_f64 {
                let test_from = Number::from(f);
                assert_eq!(parsed, test_from);
            }
        }
    }
}
