use std::str::FromStr;

pub use cosmwasm_std::Decimal256;
use cosmwasm_std::StdError;

/// Trait for converting to and from a string which is compatible with the cosmos SDK protobufs.
/// The cosmos SDK uses a string representation of the inner `Uint256` to represent a `Decimal256`.
#[allow(dead_code)] // TODO:NOW where it was used?
trait CosmosDecimalProtoString: Sized {
    /// Converts to a string which is compatible with the cosmos SDK protobufs.
    fn to_cosmos_proto_string(&self) -> String;

    /// Generates a Decimal256 from a cosmos SDK protobuf string representation.
    fn from_cosmos_proto_string(input: &str) -> Result<Self, StdError>;
}

impl CosmosDecimalProtoString for Decimal256 {
    /// Converts to a string which is compatible with the cosmos SDK protobufs.
    fn to_cosmos_proto_string(&self) -> String {
        self.atomics().to_string()
    }

    /// Generates a Decimal256 from a cosmos SDK protobuf string representation.
    fn from_cosmos_proto_string(input: &str) -> Result<Self, StdError> {
        let internal = cosmwasm_std::Uint256::from_str(input)?;
        Ok(Self::new(internal))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn decimal256_from_cosmos_proto_string_works() {
        assert_eq!(
            Decimal256::from_cosmos_proto_string("123000000000000000000").unwrap(),
            Decimal256::from_str("123").unwrap(),
        );

        assert_eq!(
            Decimal256::from_cosmos_proto_string("123456000000000000000").unwrap(),
            Decimal256::from_str("123.456").unwrap(),
        );

        assert_eq!(
            Decimal256::from_cosmos_proto_string("123456000000000000001").unwrap(),
            Decimal256::from_str("123.456000000000000001").unwrap(),
        );
    }

    #[test]
    fn decimal256_to_cosmos_proto_string_works() {
        assert_eq!(
            Decimal256::from_str("123")
                .unwrap()
                .to_cosmos_proto_string(),
            "123000000000000000000"
        );

        assert_eq!(
            Decimal256::from_str("123.456")
                .unwrap()
                .to_cosmos_proto_string(),
            "123456000000000000000"
        );

        assert_eq!(
            Decimal256::from_str("123.456000000000000001")
                .unwrap()
                .to_cosmos_proto_string(),
            "123456000000000000001"
        );
    }
}
