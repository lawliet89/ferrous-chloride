//! Number

use std::borrow::Cow;
use std::ops::Deref;
use std::str::FromStr;

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

impl<'a> Deref for Number<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
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
