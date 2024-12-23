#[cfg(not(test))]
use crossterm::style::Stylize;
#[cfg(not(test))]
use dialoguer::theme::SimpleTheme;
#[cfg(not(test))]
use dialoguer::Confirm;

pub trait Prompts {
    fn question(&mut self, question: &str) -> PromptResponse;
    fn info(&mut self, message: &str);
}

#[cfg(not(test))]
pub struct Prompt;
#[cfg(test)]
pub use tests::FakePrompt as Prompt;

#[cfg(not(test))]
impl Prompt {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(not(test))]
impl Prompts for Prompt {
    fn question(&mut self, question: &str) -> PromptResponse {
        let confirm = Confirm::with_theme(&SimpleTheme {})
            .with_prompt(format!("{}", question.blue()))
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

    pub struct SpyPrompt {
        pub info_call_count: u32,
    }

    impl FakePrompt {
        pub fn new() -> Self {
            Self {
                has_been_asked_to_confirm: false,
                response: PromptResponse::No,
            }
        }

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
        fn question(&mut self, _question: &str) -> PromptResponse {
            self.has_been_asked_to_confirm = true;
            self.response.clone()
        }

        fn info(&mut self, _message: &str) {}
    }

    impl SpyPrompt {
        pub fn new() -> Self {
            Self { info_call_count: 0 }
        }
    }

    impl Prompts for SpyPrompt {
        fn question(&mut self, _question: &str) -> PromptResponse {
            PromptResponse::Yes
        }

        fn info(&mut self, _message: &str) {
            self.info_call_count += 1;
        }
    }
}
