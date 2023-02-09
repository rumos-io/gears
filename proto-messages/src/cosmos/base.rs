pub mod v1beta1 {
    use ibc_proto::{cosmos::base::v1beta1::Coin as RawCoin, protobuf::Protobuf};

    use crate::error::Error;

    /// Coin defines a token with a denomination and an amount.
    #[derive(Clone, PartialEq)]
    pub struct Coin {
        pub denom: proto_types::Denom,
        pub amount: ::cosmwasm_std::Uint256,
    }

    impl TryFrom<RawCoin> for Coin {
        type Error = Error;

        fn try_from(value: RawCoin) -> Result<Self, Self::Error> {
            let denom = value
                .denom
                .try_into()
                .map_err(|_| Error::Coin(String::from("coin error")))?;
            let amount = value.amount;

            Ok(Coin { denom, amount })
        }
    }

    impl From<Coin> for RawCoin {
        fn from(value: Coin) -> RawCoin {
            RawCoin {
                denom: value.denom.to_string(),
                amount: value.amount,
            }
        }
    }

    impl Protobuf<RawCoin> for Coin {}

    // Represents a list of coins with the following properties:
    // - Contains at least one coin
    // - All coin amounts are positive
    // - No duplicate denominations
    // - Sorted lexicographically
    #[derive(Clone, PartialEq)]
    pub struct SendCoins(Vec<Coin>);

    impl SendCoins {
        pub fn new(coins: Vec<Coin>) -> Result<SendCoins, Error> {
            Self::validate_coins(&coins)?;

            Ok(SendCoins(coins))
        }

        // Checks that the SendCoins are sorted, have positive amount, with a valid and unique
        // denomination (i.e no duplicates). Otherwise, it returns an error.
        // A valid list of coins satisfies:
        // - Contains at least one coin
        // - All amounts are positive
        // - No duplicate denominations
        // - Sorted lexicographically
        // TODO: implement ordering on coins or denominations so that conversion to string can be avoided
        fn validate_coins(coins: &Vec<Coin>) -> Result<(), Error> {
            if coins.is_empty() {
                return Err(Error::Coins(String::from("list of coins is empty")));
            }

            if coins[0].amount.is_zero() {
                return Err(Error::Coins(String::from("coin amount must be positive")));
            };

            let mut previous_denom = coins[0].denom.to_string();

            for coin in &coins[1..] {
                if coin.amount.is_zero() {
                    return Err(Error::Coins(String::from("coin amount must be positive")));
                };

                // Less than to ensure lexicographical ordering
                // Equality to ensure that there are no duplications
                if coin.denom.to_string() <= previous_denom {
                    return Err(Error::Coins(String::from(
                        "coins are not sorted and/or contain duplicates",
                    )));
                }

                previous_denom = coin.denom.to_string();
            }

            return Ok(());
        }
    }

    impl From<SendCoins> for Vec<Coin> {
        fn from(coins: SendCoins) -> Vec<Coin> {
            coins.0
        }
    }

    impl IntoIterator for SendCoins {
        type Item = Coin;
        type IntoIter = std::vec::IntoIter<Self::Item>;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }
}
