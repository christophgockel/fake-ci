#[cfg(not(test))]
use crossterm::style::Stylize;
#[cfg(not(test))]
use dialoguer::theme::SimpleTheme;
#[cfg(not(test))]
use dialoguer::Confirm;

pub trait Prompts {
    fn question(&mut self) -> PromptResponse;
    fn info(&mut self, message: &str);
}

#[cfg(not(test))]
#[derive(Default)]
pub struct Prompt;
#[cfg(test)]
pub use tests::FakePrompt as Prompt;

#[cfg(not(test))]
impl Prompts for Prompt {
    fn question(&mut self) -> PromptResponse {
        let confirm = Confirm::with_theme(&SimpleTheme {})
            .with_prompt(format!(
                "{}",
                "Do you really want to prune all artifacts?".blue()
            ))
            .default(true)
            .interact();

        match confirm {
            Ok(true) => PromptResponse::Yes,
            _ => PromptResponse::No,
        }
    }

    fn info(&mut self, message: &str) {
        println!("{}", message.blue());
    }
}

#[derive(Clone)]
pub enum PromptResponse {
    Yes,
    No,
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub struct FakePrompt {
        pub has_been_asked_to_confirm: bool,
        pub response: PromptResponse,
    }

    impl Default for FakePrompt {
        fn default() -> Self {
            Self {
                has_been_asked_to_confirm: false,
                response: PromptResponse::No,
            }
        }
    }

    impl FakePrompt {
        pub fn always_confirming() -> Self {
            Self {
                has_been_asked_to_confirm: false,
                response: PromptResponse::Yes,
            }
        }

        pub fn always_denying() -> Self {
            Self {
                has_been_asked_to_confirm: false,
                response: PromptResponse::No,
            }
        }
    }

    impl Prompts for FakePrompt {
        fn question(&mut self) -> PromptResponse {
            self.has_been_asked_to_confirm = true;
            self.response.clone()
        }

        fn info(&mut self, _message: &str) {}
    }
}
