pub trait MeterKind: sealed::Sealed {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockMeterKind;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TxMeterKind;

impl MeterKind for BlockMeterKind {}
impl MeterKind for TxMeterKind {}

mod sealed {
    use super::*;

    pub trait Sealed {}

    impl Sealed for BlockMeterKind {}
    impl Sealed for TxMeterKind {}
}
