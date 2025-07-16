pub mod display;
pub mod input;

pub use display::Display;
pub use input::InputHandler;

pub trait IO {
    fn show_message(&self, title: &str, text: &str);
    fn get_user_input(&mut self, prompt: &str) -> anyhow::Result<Option<String>>;
}

pub struct TerminalIO {
    display: Display,
    input: InputHandler,
}

impl TerminalIO {
    pub fn new() -> anyhow::Result<Self> {
        let display = Display::new();
        let input = InputHandler::new()?;
        Ok(TerminalIO { display, input })
    }
}

impl IO for TerminalIO {
    fn show_message(&self, title: &str, text: &str) {
        self.display.print_message_box(title, text);
    }

    fn get_user_input(&mut self, prompt: &str) -> anyhow::Result<Option<String>> {
        self.input.read_line(prompt)
    }
}
