# `StoreKeys` macro

This macros implements `StoreKey` trait and validates that implementation contains no duplicates. Uses `#[skey()]` attribute.

## Target

You could implement it only on enum without fields or tuple.

## Possible attributes

### Enum

***params***: required, ident of enum variant. Used in params implementation, for details check trait description.

### Variant

***to_string****: string, not empty unique key.

```rust
#[derive(EnumIter, Debug, PartialEq, Eq, Hash, Clone, StoreKeys)]
#[skey(params = Params)]
pub enum GaiaStoreKey {
    #[skey(store_str = "bank")]
    Bank,
    #[skey(store_str = "acc")]
    Auth,
    #[skey(store_str = "params")]
    Params,
}
```

*Note*: macro doesn't implement required traits for `StoreKey` trait.
