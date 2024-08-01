# `Raw` macro

Use this macros to generate raw structure which implement `prost::Message`. Uses `#[raw()]` attribute.

## Target

You could implement it only on structures with named fields.

## Possible attributes

### Structure

**derive**: additional derives to generated structures.

Example:

```rust
#[derive(Raw)]
#[raw(derive(Clone, Debug))]
struct Example
{
    // omit details
}
```

## Field

**raw**: type of raw field. Could be any rust type. \
**kind**: proto kind of raw field. See possible values for scalar types in [prost](https://docs.rs/prost/latest/prost/#scalar-values) and for structures use `message`. \
**optional**: flag, indicates that field should be wrapped in `Option`. \
**repeated**: flag, indicates that field should be wrapped in `Vec` \
**tag**: optional number

## Example

Simple usage

```rust
#[derive(Raw)]
pub struct QuerySigningInfoRequest {
    #[raw(raw = String, kind(string))]
    pub cons_address: ConsAddress,
}
```

Usage of optional.

```rust
#[derive(Raw)]
pub struct QuerySigningInfosRequest {
    #[raw(kind(message), optional, raw = PageRequest)]
    pub pagination: PaginationRequest,
}
```

It forbidden to use `repeated` and `optional` flags, but you could omit this with `raw = Vec::<T>`.

```rust
#[derive(Raw)]
pub struct QueryValidatorResponse {
    #[raw(kind(bytes), raw = Vec::<u8>, optional )]
    pub validator: Option<Validator>,
}
```
