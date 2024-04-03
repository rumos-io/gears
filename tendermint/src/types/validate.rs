// Copy of https://docs.rs/ibc-core-host-types/0.51.0/src/ibc_core_host_types/validate.rs.html

use super::chain_id::ChainIdErrors;

const VALID_SPECIAL_CHARS: &str = "._+-#[]<>";

/// Checks if a prefix forms a valid identifier with the given min/max identifier's length.
/// The prefix must be between `min_id_length - 2`, considering `u64::MIN` (1 char) and "-"
/// and `max_id_length - 21` characters, considering `u64::MAX` (20 chars) and "-".
pub fn validate_prefix_length(
    prefix: &str,
    min_id_length: u64,
    max_id_length: u64,
) -> Result<(), ChainIdErrors> {
    // Prefix must be at least `min_id_length - 2` characters long since the
    // shortest identifier we can construct is `{prefix}-0` which extends prefix
    // by 2 characters.
    let min = min_id_length.saturating_sub(2);
    // Prefix must be at most `max_id_length - 21` characters long since the
    // longest identifier we can construct is `{prefix}-{u64::MAX}` which
    // extends prefix by 21 characters.
    let max = max_id_length.saturating_sub(21);

    validate_identifier_length(prefix, min, max)
}

/// Checks if the identifier forms a valid identifier with the given min/max length as specified in the
/// [`ICS-24`](https://github.com/cosmos/ibc/tree/main/spec/core/ics-024-host-requirements#paths-identifiers-separators)]
/// spec.
pub fn validate_identifier_length(id: &str, min: u64, max: u64) -> Result<(), ChainIdErrors> {
    // Make sure min is at least one so we reject empty identifiers.
    let min = min.max(1);
    let length = id.len() as u64;
    if (min..=max).contains(&length) {
        Ok(())
    } else {
        Err(ChainIdErrors::InvalidLength {
            id: id.into(),
            min,
            max,
        })
    }
}

/// Checks if the identifier only contains valid characters as specified in the
/// [`ICS-24`](https://github.com/cosmos/ibc/tree/main/spec/core/ics-024-host-requirements#paths-identifiers-separators)]
/// spec.
pub fn validate_identifier_chars(id: &str) -> Result<(), ChainIdErrors> {
    // Check that the identifier comprises only valid characters:
    // - Alphanumeric
    // - `.`, `_`, `+`, `-`, `#`
    // - `[`, `]`, `<`, `>`
    if !id
        .chars()
        .all(|c| c.is_alphanumeric() || VALID_SPECIAL_CHARS.contains(c))
    {
        return Err(ChainIdErrors::InvalidCharacter(id.to_owned()));
    }

    // All good!
    Ok(())
}
