use std::marker::PhantomData;

use crate::{ApplicationInfo, NilAuxCommand};

#[derive(Debug, Clone, ::clap::Subcommand)]
#[command( about = format!( "{} doesn't handle any aux command", T::APP_NAME))]
pub enum CliNilAuxCommand<T: ApplicationInfo> {
    #[command(skip)]
    None(PhantomData<T>),
}

impl<T: ApplicationInfo> From<CliNilAuxCommand<T>> for NilAuxCommand {
    fn from(_value: CliNilAuxCommand<T>) -> Self {
        Self
    }
}
