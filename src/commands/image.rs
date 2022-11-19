use crate::commands::CommandError;
use crate::io::docker::build_image;
use crate::Context;
use clap::Args;

#[derive(Args)]
pub struct Image {}

pub fn command(context: &Context) -> Result<(), CommandError> {
    build_image(&context.image_tag).map_err(CommandError::execution)
}
