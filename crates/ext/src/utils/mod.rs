// ext::utils — CLI output and utility helpers

mod output_channel;
pub use output_channel::{Confirm, ConfirmDefault, ConfirmOrEmpty, InputOutputChannel, OutputChannel};

pub mod metrics;
pub mod time;
