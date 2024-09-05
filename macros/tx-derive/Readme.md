# AppMessage macros for generating implementation of TxMessage trait

Macro generates implementation of `TxMessage` trait and `TryFrom<Any, Error = CoreError> for Self` and `From<Self> for Any`. Uses `#[msg()]` attribute.

## Possible attributes

### Structure

Container: \
***url***: *required*, type url for usage in cast from `Any`. \
***amino_url***: *optional*, url for legacy amino signing. If no option specified uses `url` variant.

Fields: \
***signer***: *optional*, mark fields as part of signers. *Note*: in current implementation this field should be type of `AccAddress`.

### Enum

Container: \
No attribute available for container layer.

Variants: \
***url***: *required*, type url for usage in cast from `Any`. Could be `String` or `Path` e.g. `MsgSend::TYPE_URL`.
