use rustyline::{Editor, error::ReadlineError};
use std::path::PathBuf;
use crate::error::{Result, SessionError};

pub struct InputHandler {
    editor: Editor<(), rustyline::history::FileHistory>,
}

impl InputHandler {
    pub fn new() -> Result<Self> {
        let mut editor = Editor::new().map_err(|e| SessionError::Processing { reason: format!("Failed to create editor: {}", e) })?;

        let history_file = Self::get_history_file();
        if history_file.exists() {
            let _ = editor.load_history(&history_file);
        }

        Ok(Self { editor })
    }

    pub fn read_line(&mut self, prompt: &str) -> Result<Option<String>> {
        match self.editor.readline(prompt) {
            Ok(line) => {
                if !line.trim().is_empty() {
                    self.editor.add_history_entry(line.as_str()).map_err(|e| SessionError::Processing { reason: format!("Failed to add history entry: {}", e) })?;
                }
                Ok(Some(line.trim().to_owned()))
            }
            Err(ReadlineError::Interrupted) => Ok(None),
            Err(ReadlineError::Eof) => {
                println!();
                Err(SessionError::UserInput { reason: "EOF".to_string() }.into())
            }
            Err(err) => Err(SessionError::UserInput { reason: format!("Error reading line: {}", err) }.into()),
        }
    }

    pub fn save_history(&mut self) -> Result<()> {
        let history_file = Self::get_history_file();
        if let Some(parent) = history_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        self.editor.save_history(&history_file).map_err(|e| SessionError::Processing { reason: format!("Failed to save history: {}", e) })?;
        Ok(())
    }

    fn get_history_file() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("deputy").join("history")
        } else {
            PathBuf::from(".deputy_history")
        }
    }
}

impl Drop for InputHandler {
    fn drop(&mut self) {
        let _ = self.save_history();
    }
}
