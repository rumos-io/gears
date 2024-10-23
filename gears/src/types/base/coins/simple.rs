use bytes::Bytes;

use crate::types::base::coin::{DecimalCoin, UnsignedCoin};

use super::*;

pub type SimpleDecimalCoins = SimpleCoins<DecimalCoin>;
pub type SimpleUnsignedCoins = SimpleCoins<UnsignedCoin>;

pub fn format_coins<'a, U: ToString + 'a>(coins: impl IntoIterator<Item = &'a U>) -> Bytes {
    coins
        .into_iter()
        .map(|this| this.to_string())
        .collect::<Vec<_>>()
        .join(",")
        .into_bytes()
        .into()
}

// TODO: Allow borrowed data
#[derive(Debug, Clone)]
pub struct SimpleCoins<U>(Vec<U>);

impl<U> SimpleCoins<U> {
    pub fn new(coins: impl IntoIterator<Item = U>) -> Self {
        Self(coins.into_iter().collect())
    }
}

impl<U: ToString> SimpleCoins<U> {
    pub fn to_string_bytes(&self) -> Bytes {
        self.to_string().into_bytes().into()
    }
}

impl<U> From<Vec<U>> for SimpleCoins<U> {
    fn from(value: Vec<U>) -> Self {
        Self(value)
    }
}

impl<U> From<SimpleCoins<U>> for Vec<U> {
    fn from(value: SimpleCoins<U>) -> Self {
        value.0
    }
}

impl<T: ZeroNumeric, U: Coin<Amount = T>> TryFrom<SimpleCoins<U>> for Coins<T, U> {
    type Error = CoinsError;

    fn try_from(value: SimpleCoins<U>) -> Result<Self, Self::Error> {
        Self::new(value.0)
    }
}

impl<U: ToString> std::fmt::Display for SimpleCoins<U> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = self
            .0
            .iter()
            .map(|this| this.to_string())
            .collect::<Vec<_>>()
            .join(",");

        write!(f, "{result}")
    }
}

#[cfg(test)]
mod tests {
    use extensions::testing::UnwrapTesting;

    use super::*;

    // https://github.com/cosmos/cosmos-sdk/blob/d3f09c222243bb3da3464969f0366330dcb977a8/types/coin.go#L190-L205

    #[test]
    fn test_display_multiple_i32() {
        let list = SimpleCoins::new([1, 2, 3]);

        assert_eq!("1,2,3", list.to_string())
    }

    #[test]
    fn test_display_single_i32() {
        let list = SimpleCoins::new([1]);

        assert_eq!("1", list.to_string())
    }

    #[test]
    fn test_display_empty_i32() {
        let list = SimpleCoins::<u32>::new([]);

        assert_eq!("", list.to_string())
    }

    #[test]
    fn test_display_multiple_signed() {
        let list = SimpleCoins::new([
            DecimalCoin::from_str("1uatom").unwrap_test(),
            DecimalCoin::from_str("2uatom").unwrap_test(),
            DecimalCoin::from_str("3uatom").unwrap_test(),
        ]);

        assert_eq!("1uatom,2uatom,3uatom", list.to_string())
    }

    #[test]
    fn test_display_single_signed() {
        let list = SimpleCoins::new([DecimalCoin::from_str("1uatom").unwrap_test()]);

        assert_eq!("1uatom", list.to_string())
    }

    #[test]
    fn test_display_empty_signed() {
        let list = SimpleCoins::<DecimalCoin>::new([]);

        assert_eq!("", list.to_string())
    }

    #[test]
    fn test_display_multiple_unsigned() {
        let list = SimpleCoins::new([
            UnsignedCoin::from_str("1uatom").unwrap_test(),
            UnsignedCoin::from_str("2uatom").unwrap_test(),
            UnsignedCoin::from_str("3uatom").unwrap_test(),
        ]);

        assert_eq!("1uatom,2uatom,3uatom", list.to_string())
    }

    #[test]
    fn test_display_single_unsigned() {
        let list = SimpleCoins::new([UnsignedCoin::from_str("1uatom").unwrap_test()]);

        assert_eq!("1uatom", list.to_string())
    }

    #[test]
    fn test_display_empty_unsigned() {
        let list = SimpleCoins::<UnsignedCoin>::new([]);

        assert_eq!("", list.to_string())
    }
}
