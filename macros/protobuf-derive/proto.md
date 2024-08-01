# `Protobuf` macro

This macros to generate cast from raw structure and implementing protobuf trait. Uses `#[proto()]` attribute.

## Target

You could implement it only on structures with named fields. **Note** that number of fields in structure should be >= than number of fields in raw structure.

## Possible attributes

### Structure

**raw**: name of raw structure which you want to use.\
Possible values: name of structure or path to it. \
_Example_: `RawProto` or `inner::RawProto`

### Field

**name** : _optional_, name of field that you want to use in raw structure.
If not specified tries to use structure with name `Raw{NameOfStructure}` \
**optional** : _flag_ indicates that raw structure field type of `Option<T>`. Exclusive to `repeated` flag. \
**repeated** : _flag_ indicates that raw structure filed type of `Vec<T>`. Exclusive to `optional` flag.

## Example

Use on structure with singe field and raw structure from 3rd party crate.

```rust
    #[derive(Protobuf)]
    #[proto(raw = "inner::QueryAccountRequest")]
    pub struct QueryAccountRequest {
        pub address: AccAddress,
    }
```

With renamed field and auto resolved name().

```rust
#[derive(Protobuf)]
pub struct QueryParamsRequest {
    #[proto(name = "params_type")]
    pub kind: ParamsQuery,
}
```
