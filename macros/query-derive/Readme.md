# Query macros for generating implementation of Query trait

Macro generates implementations for the Query trait based on attributes specified.

## Possible attributes

**kind**:
Macro able to guess kind by structure nam simply checking if name contains(ignoring case) "request"/"response" word otherwise you need to specify it by yourself.
Possible values are "request" and "response".
Type: *String* \
**raw**: The raw Protobuf type for serialization. \
**url**: The URL for the query, required for request types.

## Targets

### Structs

**Request**: \
Url attribute is required. Generates `const QUERY_URL` with provided value. \
Generates the query_url method returning value of `QUERY_URL`. \
Implements Query trait with methods for getting the query URL and converting the struct into bytes using Protobuf.

**Response**: \
Implements QueryResponse trait with a method to convert the struct into bytes using Protobuf.

### Enums

**Request**:
generates match arms for each variant to get the query URL and convert the variant into bytes.

**Response**
Generates match arms for each variant to convert the variant into bytes.

*Note*: enum implementation expect inner struct to implement `Query` trait similar to way this macro does.
