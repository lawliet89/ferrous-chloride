//! Number

use std::borrow::Cow;
use std::ops::Deref;
use std::str::FromStr;

use nom::types::CompleteStr;
use nom::{alt, call, char, digit, map, named, opt, pair, recognize_float, tuple};

use crate::Error;

/// A number, represented as a string for aribitrary precision
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Number<'a>(Cow<'a, str>);

macro_rules! impl_from_number {
    ($($from:ty )*) => {$(
        impl<'a> From<$from> for Number<'a> {
            fn from(n: $from) -> Self {
                Number(Cow::Owned(n.to_string()))
            }
        }
    )*};
}

impl_from_number!(u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64);

impl<'a> From<&'a str> for Number<'a> {
    fn from(s: &'a str) -> Self {
        Number(Cow::Borrowed(s.trim_matches('+')))
    }
}

impl<'a> From<CompleteStr<'a>> for Number<'a> {
    fn from(s: CompleteStr<'a>) -> Self {
        Number(Cow::Borrowed(s.0.trim_matches('+')))
    }
}

impl<'a> Deref for Number<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<'a> crate::AsOwned for Number<'a> {
    type Output = Number<'static>;

    fn as_owned(&self) -> Self::Output {
        Number(Cow::Owned(self.0.as_owned()))
    }
}

macro_rules! impl_to_number {
    ($($name:ident => $to:ty, )*) => {$(
        impl_to_number!($name => $to => stringify!(Attempt conversion to $to));
    )*};
    ($name:ident => $to:ty => $doc:expr) => {
        #[doc=$doc]
        pub fn $name(&self) -> Result<$to, <$to as FromStr>::Err> {
            self.parse()
        }
    };
}

impl<'a> Number<'a> {
    impl_to_number!(
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Component<'a> {
    /// The original input number literal
    pub input: &'a str,
    /// Whether the number is positive
    pub positive: bool,
    /// The whole number part of the number
    pub whole: &'a str,
    /// The fraction (decimal) part of the number
    pub fraction: Option<&'a str>,
    /// Whether an exponent is present
    pub exponent: Option<Exponent<'a>>,
}

impl<'a> Component<'a> {
    /// Parse a number string into its components
    ///
    /// Referenced from the implementation of [`nom::recognize_float`]
    pub fn parse(s: &'a str) -> Result<Self, Error> {
        let input = CompleteStr(s);
        let (input, positive) =
            opt!(input, alt!(char!('+') | char!('-'))).map_err(|e| Error::from_err_str(&e))?;
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
                    Some((_, None)) => None,
                    Some((_, Some(decimals))) => Some(decimals)
                };
                (digit, decimals)
            } }
            | tuple!(char!('.'), digit) => { |(_, decimals)| (CompleteStr("0"), Some(decimals))  }
        )
        .map_err(|e| Error::from_err_str(&e))?;

        let whole = whole.0;
        let fraction = fraction.map(|frac| frac.0);

        let exponent = Exponent::parse(input.0, true)?;

        Ok(Component {
            input: s,
            positive,
            whole,
            fraction,
            exponent,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Exponent<'a> {
    /// Whether the exponent is positive
    pub positive: bool,
    /// The number part of the exponent
    pub exponent: &'a str,
}

impl<'a> Exponent<'a> {
    pub fn parse(input: &'a str, check_remaining: bool) -> Result<Option<Self>, Error> {
        let input = CompleteStr(input);
        let (remaining, exponent) = opt!(
            input,
            tuple!(
                alt!(char!('e') | char!('E')),
                opt!(alt!(char!('+') | char!('-'))),
                digit
            )
        )
        .map_err(|e| Error::from_err_str(&e))?;

        let exponent = exponent.map(|(_, sign, exponent)| {
            let positive = match sign {
                None => true,
                Some('+') => true,
                Some('-') => false,
                _ => unreachable!("bug in number sign parsing"),
            };
            let exponent = exponent.0;
            Exponent { positive, exponent }
        });

        if check_remaining && !remaining.is_empty() {
            Err(Error::UnexpectedRemainingInput(remaining.to_string()))?;
        }

        Ok(exponent)
    }
}

named!(
    pub number(CompleteStr) -> Number,
    map!(call!(recognize_float), From::from)
);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::utils::ResultUtilsString;

    #[test]
    fn integers_are_parsed_correctly() {
        assert_eq!(
            number(CompleteStr("12345")).unwrap_output(),
            From::from(12345)
        );
        assert_eq!(
            number(CompleteStr("+12345")).unwrap_output(),
            From::from(12345)
        );
        assert_eq!(
            number(CompleteStr("-12345")).unwrap_output(),
            From::from(-12345)
        );
    }

    #[test]
    fn floats_are_parsed_correctly() {
        assert_eq!(
            number(CompleteStr("12.34")).unwrap_output(),
            From::from(12.34)
        );
        assert_eq!(
            number(CompleteStr("+12.34")).unwrap_output(),
            From::from(12.34)
        );
        assert_eq!(
            number(CompleteStr("-12.34")).unwrap_output(),
            From::from(-12.34)
        );
    }
}
