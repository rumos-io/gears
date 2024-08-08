use crate::x::module::Module;
use kv_store::StoreKey;

/// EvidenceSlashingKeeper defines the slashing module interface contract needed by the
/// evidence module.
pub trait EvidenceSlashingKeeper<SK: StoreKey, M: Module>: Clone + Send + Sync + 'static {
    // GetPubkey(sdk.Context, cryptotypes.Address) (cryptotypes.PubKey, error)
    // IsTombstoned(sdk.Context, sdk.ConsAddress) bool
    // HasValidatorSigningInfo(sdk.Context, sdk.ConsAddress) bool
    // Tombstone(sdk.Context, sdk.ConsAddress)
    // Slash(sdk.Context, sdk.ConsAddress, sdk.Dec, int64, int64)
    // SlashFractionDoubleSign(sdk.Context) sdk.Dec
    // Jail(sdk.Context, sdk.ConsAddress)
    // JailUntil(sdk.Context, sdk.ConsAddress, time.Time)
}
