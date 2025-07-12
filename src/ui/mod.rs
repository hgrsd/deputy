use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub mod input;

pub struct Spinner {
    progress_bar: ProgressBar,
}

impl Spinner {
    pub fn new(message: &str) -> Self {
        let progress_bar = ProgressBar::new_spinner();
        progress_bar.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                .template("{spinner:.blue} {msg}")
                .unwrap(),
        );
        progress_bar.set_message(message.to_string());
        progress_bar.enable_steady_tick(Duration::from_millis(120));
        
        Self { progress_bar }
    }
    
    pub fn finish(&self) {
        self.progress_bar.finish_and_clear();
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.progress_bar.finish_and_clear();
    }
}