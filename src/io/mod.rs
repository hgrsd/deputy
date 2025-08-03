pub mod display;
pub mod input;

use std::sync::{Arc, Mutex};
use crate::error::Result;

pub use display::Display;
pub use input::InputHandler;

pub trait IO: Send + Sync {
    fn show_message(&self, title: &str, text: &str);
    fn show_snippet(&self, title: &str, text: &str);
    fn get_user_input(&mut self, prompt: &str) -> Result<Option<String>>;
}

pub struct TerminalIO {
    display: Display,
    input: Arc<Mutex<InputHandler>>,
}

impl TerminalIO {
    pub fn new() -> Result<Self> {
        let display = Display::new();
        let input = Arc::new(Mutex::new(InputHandler::new()?));
        Ok(TerminalIO { display, input })
    }
}

impl IO for TerminalIO {
    fn show_message(&self, title: &str, text: &str) {
        self.display.print_message_box(title, text);
    }

    fn show_snippet(&self, title: &str, text: &str) {
        let total_lines = text.lines().count();
        let snippet_lines = text.lines().take(10).collect::<Vec<&str>>();
        let mut formatted_output = snippet_lines.join("\n");

        if total_lines > 10 {
            formatted_output.push_str(&format!("\n... ({} more lines)", total_lines - 10));
        }

        self.display
            .print_message_box(title, &formatted_output);
    }

    fn get_user_input(&mut self, prompt: &str) -> Result<Option<String>> {
        let mut input = self.input.lock().unwrap();
        input.read_line(prompt)
    }
}
