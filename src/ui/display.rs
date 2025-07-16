use crate::core::Message;

pub struct DisplayManager {
    terminal_width: usize,
}

impl DisplayManager {
    pub fn new() -> Self {
        Self {
            terminal_width: Self::get_terminal_width(),
        }
    }

    fn get_terminal_width() -> usize {
        match crossterm::terminal::size() {
            Ok((width, _)) => width as usize,
            Err(_) => 80,
        }
    }

    pub fn handle_message(&self, message: &Message) {
        match message {
            Message::User(text) => {
                self.print_message_box("You", text);
            }
            Message::Model(text) => {
                self.print_message_box("Deputy", text);
            }
            _ => {}
        }
    }

    fn print_message_box(&self, title: &str, text: &str) {
        let content_width = self.terminal_width.saturating_sub(4);

        println!("\n┌─ {}", title);

        let wrapped_lines = self.wrap_text(text, content_width);
        for line in wrapped_lines {
            println!("│ {}", line);
        }

        println!("└─");
    }

    fn wrap_text(&self, text: &str, width: usize) -> Vec<String> {
        let mut wrapped_lines = Vec::new();

        for line in text.lines() {
            if line.len() <= width {
                wrapped_lines.push(line.to_string());
            } else {
                let mut current_line = String::new();
                let words: Vec<&str> = line.split_whitespace().collect();

                for word in words {
                    if word.len() > width {
                        if !current_line.is_empty() {
                            wrapped_lines.push(current_line.clone());
                            current_line.clear();
                        }
                        for chunk in word.chars().collect::<Vec<_>>().chunks(width) {
                            wrapped_lines.push(chunk.iter().collect());
                        }
                    } else if current_line.len() + word.len() < width {
                        if !current_line.is_empty() {
                            current_line.push(' ');
                        }
                        current_line.push_str(word);
                    } else {
                        if !current_line.is_empty() {
                            wrapped_lines.push(current_line.clone());
                        }
                        current_line = word.to_string();
                    }
                }

                if !current_line.is_empty() {
                    wrapped_lines.push(current_line);
                }
            }
        }

        wrapped_lines
    }
}
