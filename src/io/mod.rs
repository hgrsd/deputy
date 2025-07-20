pub mod display;
pub mod input;

use std::sync::{Arc, Mutex};

pub use display::Display;
pub use input::InputHandler;

pub trait IO: Send + Sync {
    fn show_message(&self, title: &str, text: &str);
    fn show_snippet(&self, title: &str, text: &str, max_lines: usize);
    fn get_user_input(&mut self, prompt: &str) -> anyhow::Result<Option<String>>;
}

pub struct TerminalIO {
    display: Display,
    input: Arc<Mutex<InputHandler>>,
}

impl TerminalIO {
    pub fn new() -> anyhow::Result<Self> {
        let display = Display::new();
        let input = Arc::new(Mutex::new(InputHandler::new()?));
        Ok(TerminalIO { display, input })
    }
}

impl IO for TerminalIO {
    fn show_message(&self, title: &str, text: &str) {
        self.display.print_message_box(title, text);
    }

    fn show_snippet(&self, path: &str, content: &str, total_lines: usize) {
        let snippet_lines = content.lines().take(10).collect::<Vec<&str>>();
        let mut formatted_output = snippet_lines.join("\n");

        if total_lines > 10 {
            formatted_output.push_str(&format!("\n... ({} more lines)", total_lines - 10));
        }

        self.display
            .print_message_box(&format!("Reading: {}", path), &formatted_output);
    }

    fn get_user_input(&mut self, prompt: &str) -> anyhow::Result<Option<String>> {
        let mut input = self.input.lock().unwrap();
        input.read_line(prompt)
    }
}
