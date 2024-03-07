use std::marker::PhantomData;

use crate::{ApplicationInfo, NilAuxCommand};

#[derive(Debug, Clone, ::clap::Args)]
#[command( about = format!( "{} doesn't handle any aux command", T::APP_NAME))]
pub struct CliNilAuxCommand<T: ApplicationInfo> {
    #[arg(skip)]
    _marker: PhantomData<T>,
}

impl<T: ApplicationInfo> From<CliNilAuxCommand<T>> for NilAuxCommand {
    fn from(_value: CliNilAuxCommand<T>) -> Self {
        Self
    }
}
