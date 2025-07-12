use rustyline::{Editor, error::ReadlineError};
use std::path::PathBuf;

pub struct InputHandler {
    editor: Editor<(), rustyline::history::FileHistory>,
}

impl InputHandler {
    pub fn new() -> anyhow::Result<Self> {
        let mut editor = Editor::new()?;

        let history_file = Self::get_history_file();
        if history_file.exists() {
            let _ = editor.load_history(&history_file);
        }

        Ok(Self { editor })
    }

    pub fn read_line(&mut self, prompt: &str) -> anyhow::Result<Option<String>> {
        match self.editor.readline(prompt) {
            Ok(line) => {
                // Add to history if it's not empty
                if !line.trim().is_empty() {
                    self.editor.add_history_entry(line.as_str())?;
                }
                Ok(Some(line))
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C
                println!("^C");
                Ok(None)
            }
            Err(ReadlineError::Eof) => {
                // Ctrl+D
                println!();
                Err(anyhow::anyhow!("EOF"))
            }
            Err(err) => Err(anyhow::anyhow!("Error reading line: {}", err)),
        }
    }

    pub fn save_history(&mut self) -> anyhow::Result<()> {
        let history_file = Self::get_history_file();
        if let Some(parent) = history_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        self.editor.save_history(&history_file)?;
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
