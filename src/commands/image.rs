use crate::commands::CommandError;
use crate::io::processes::ProcessesToExecute;
use crate::io::prompt::Prompts;
use crate::Context;
use clap::Args;

#[derive(Args, Default)]
pub struct Image {
    /// Force rebuilding the image
    #[clap(short, long)]
    force: bool,
}

pub fn command<PROMPTS: Prompts, PROCESSES: ProcessesToExecute>(
    prompt: &mut PROMPTS,
    processes: &mut PROCESSES,
    context: &Context,
    args: &Image,
) -> Result<(), CommandError> {
    if args.force || processes.image_needs_to_be_built(&context.image_tag)? {
        prompt.info("Building Fake CI image");
        processes.build_image(&context.image_tag)?;
    } else {
        prompt.info("Image is up-to-date");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::processes::tests::ProcessesSpy;
    use crate::io::prompt::tests::SpyPrompt;

    #[test]
    fn provides_info_message_when_image_is_up_to_date() {
        let mut prompt = SpyPrompt::default();
        let mut processes = ProcessesSpy::default();
        let context = Context::default();
        let default_args = Image::default();

        command(&mut prompt, &mut processes, &context, &default_args).unwrap();

        assert_eq!(prompt.info_call_count, 1);
    }

    #[test]
    fn builds_new_image_when_it_needs_to_be_built() {
        let mut prompt = SpyPrompt::default();
        let mut processes = ProcessesSpy::with_image_to_be_built();
        let context = Context::default();
        let default_args = Image::default();

        command(&mut prompt, &mut processes, &context, &default_args).unwrap();

        assert_eq!(processes.build_image_call_count, 1);
    }

    #[test]
    fn provides_info_message_when_image_is_going_to_be_built() {
        let mut prompt = SpyPrompt::default();
        let mut processes = ProcessesSpy::with_image_to_be_built();
        let context = Context::default();
        let default_args = Image::default();

        command(&mut prompt, &mut processes, &context, &default_args).unwrap();

        assert_eq!(prompt.info_call_count, 1);
    }

    #[test]
    fn force_argument_rebuilds_image_even_if_it_exists_already() {
        let mut prompt = SpyPrompt::default();
        let mut processes = ProcessesSpy::default();
        let context = Context::default();
        let args = Image { force: true };

        command(&mut prompt, &mut processes, &context, &args).unwrap();

        assert_eq!(processes.build_image_call_count, 1);
    }
}
