pub trait MeterKind: sealed::Sealed {}

#[derive(Debug, Clone)]
pub struct BlockMeterKind;

impl MeterKind for BlockMeterKind {}

mod sealed {
    use super::*;

    pub trait Sealed {}

    impl Sealed for BlockMeterKind {}
}
