use std::fmt::{self, Display};

use lazy_static::lazy_static;
use regex::Regex;

use crate::error::Error;

// Denominations can be 3 ~ 128 characters long and support letters, followed by either
// a letter, a number or a separator ('/').
lazy_static! {
    static ref RE: Regex =
        Regex::new(r"^[a-zA-Z][a-zA-Z0-9/-]{2,127}$").expect("hard coded RE won't fail");
}

#[derive(Debug, PartialEq, Clone)]
pub struct Denom(String);

impl TryFrom<String> for Denom {
    type Error = Error;

    fn try_from(v: String) -> Result<Self, Self::Error> {
        if !RE.is_match(&v) {
            return Err(Error::InvalidDenom);
        };

        Ok(Denom(v))
    }
}

impl Display for Denom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn from_string_successes() {
        let res: Denom = "abcd".to_string().try_into().unwrap();
        assert_eq!(Denom("abcd".into()), res);

        let res: Denom = "ibc/7F1D3FCF4AE79E1554D670D1AD949A9BA4E4A3C76C63093E17E446A46061A7A2"
            .to_string()
            .try_into()
            .unwrap();
        assert_eq!(
            Denom("ibc/7F1D3FCF4AE79E1554D670D1AD949A9BA4E4A3C76C63093E17E446A46061A7A2".into()),
            res
        );

        let res: Denom = "at0m".to_string().try_into().unwrap();
        assert_eq!(Denom("at0m".into()), res);

        let res: Denom = "Atom".to_string().try_into().unwrap();
        assert_eq!(Denom("Atom".into()), res);
    }

    #[test]
    fn from_string_failures() {
        // too short
        let res: Result<Denom, Error> = "a".to_string().try_into();
        let err = res.unwrap_err();
        assert_eq!(err, Error::InvalidDenom);

        // starts with a number
        let res: Result<Denom, Error> = "8aaaaaaaaaaa".to_string().try_into();
        let err = res.unwrap_err();
        assert_eq!(err, Error::InvalidDenom);

        // too long
        let res: Result<Denom, Error> = "abcdefghijklmnopqrstuvwxyzxxxxxxxxxxabcdefghijklmnopqrstuvwxyzxxxxxxxxxxabcdefghijklmnopqrstuvwxyzxxxxxxxxxx123456789012345678901".to_string().try_into();
        let err = res.unwrap_err();
        assert_eq!(err, Error::InvalidDenom);

        // non alpha numeric character
        let res: Result<Denom, Error> = "ab🙂cd".to_string().try_into();
        let err = res.unwrap_err();
        assert_eq!(err, Error::InvalidDenom);

        // non alpha numeric characters
        let res: Result<Denom, Error> = "     ".to_string().try_into();
        let err = res.unwrap_err();
        assert_eq!(err, Error::InvalidDenom);

        // non alpha numeric characters
        let res: Result<Denom, Error> = "sdsdsd dsdsd".to_string().try_into();
        let err = res.unwrap_err();
        assert_eq!(err, Error::InvalidDenom);
    }

    #[test]
    fn to_string_success() {
        let denom: Denom = "atom".to_string().try_into().unwrap();
        assert_eq!("atom", denom.to_string());
    }
}
