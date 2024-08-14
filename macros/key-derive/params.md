# `ParamsKeys` macro

This macros implements `ParamsSubspaceKey` trait and validates that implementation contains no duplicates. Uses `#[pkey()]` attribute.

## Target

You could implement it only on enum without fields or tuple.

## Possible attributes

### Enum

Contains no additional attributes for container.

### Variant

***to_string****: string, not empty unique key.

```rust
#[derive(EnumIter, Debug, PartialEq, Eq, Hash, Clone, ParamsKeys)]
pub enum GaiaParamsStoreKey {
    #[pkey(to_string = "bank/")]
    Bank,
    #[pkey(to_string = "auth/")]
    Auth,
    #[pkey(to_string = "baseapp/")]
    BaseApp,
}
```

*Note*: macro doesn't implement required traits for `ParamsSubspaceKey` trait.
