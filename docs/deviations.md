# Deviations from the Cosmos SDK

These are known deviations from the Cosmos SDK. In future some of these deviations maybe removed.

1. In staking related modules Gears uses the Tendermint public key for consensus public key fields whereas in the Cosmos SDK they use the application key. Our rationale is that these two keys are not required to support the same schemes. In fact currently our Tendermint key supports Ed25519 and Secp256k1, whereas our app key only supports Secp256k1. This also means that staking structures with a consensus public key are JSON encoded with a type of the form `tendermint/PubKeyEd25519"` whereas in the Cosmos SDK they take the form `/cosmos.crypto.ed25519.PubKey`. This results in an incompatibility in the genesis files between Gears and the Cosmos SDK.
