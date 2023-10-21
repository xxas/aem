use std::str::FromStr;

pub trait ParseWithRadix
{
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, std::num::ParseIntError>
    where Self: Sized;
}

macro_rules! impl_parse_with_radix
{
    ($($type:ty),* ) => 
    {
        $(
            impl ParseWithRadix for $type 
            {
                fn from_str_radix(src: &str, radix: u32) -> Result<Self, std::num::ParseIntError> 
                {
                    <$type>::from_str_radix(src, radix)
                }
            }
        )*
    };
}

impl_parse_with_radix!(
    u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize
);

pub enum ParseValueError<T: FromStr>
{
    ParseInt(std::num::ParseIntError),
    FromStr(<T as FromStr>::Err),
    OtherError(String)
}

pub fn parse_value<T: ParseWithRadix + FromStr>(s: &str) -> Result<T, ParseValueError<T>> 
{
    if (s.starts_with("0x") || s.starts_with("-0x")) && s[if s.starts_with('-') { 3.. } else { 2.. }].chars().all(|c| c.is_digit(16)) 
    {  // Hexadecimal value.
        T::from_str_radix(&s[if s.starts_with('-') { 3.. } else { 2.. }], 16).map_err(ParseValueError::ParseInt)
    } 
    else if s.starts_with('-') && s[1..].chars().all(|c| c.is_digit(10)) || s.chars().all(|c| c.is_digit(10)) 
    { // Decimal value (signed or unsigned).
        s.parse::<T>().map_err(ParseValueError::FromStr)
    } 
    else 
    { // Unparsable value.
        Err(ParseValueError::OtherError(format!(r#"Unable to parse value from: "{}""#, s)))
    }
}
