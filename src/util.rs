use num_traits::Num;
use std::str::FromStr;

pub enum ParseFromError<T: FromStr + Num>
{
    FromRadix(<T as Num>::FromStrRadixErr),
    FromStr(<T as FromStr>::Err)
}

pub trait ParseFrom: FromStr + Num 
{
    fn parse(val_str: &str) -> Result<Self, ParseFromError<Self>>;
}

impl<T> ParseFrom for T
    where T: FromStr + Num
{
    fn parse(val_str: &str) -> Result<T, ParseFromError<T>>
    {
        let radix = if val_str.starts_with("0x") { 16 }
                    else if val_str.starts_with("0b") { 2 } 
                    else { 10 };

        if radix != 10
        {
            return T::from_str_radix(&val_str[2..], radix)
                .map_err(ParseFromError::FromRadix)
        }
        
        T::from_str(val_str)
            .map_err(ParseFromError::FromStr)
    }
}
