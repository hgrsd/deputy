pub mod anthropic;
pub mod openai;
pub mod session_factory;

use clap::ValueEnum;
use crate::error::{ConfigError, Result};

#[derive(Debug, Clone, ValueEnum)]
pub enum Provider {
    Anthropic,
    OpenAI,
    Ollama,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::Anthropic => write!(f, "anthropic"),
            Provider::OpenAI => write!(f, "open-ai"),
            Provider::Ollama => write!(f, "ollama"),
        }
    }
}

impl Provider {
    pub fn required_env_vars(&self) -> Vec<&'static str> {
        match self {
            Provider::Anthropic => vec!["ANTHROPIC_API_KEY"],
            Provider::OpenAI => vec!["OPENAI_API_KEY"],
            Provider::Ollama => vec![],
        }
    }

    pub fn validate_configuration(&self) -> Result<()> {
        let required_vars = self.required_env_vars();
        let missing_vars: Vec<_> = required_vars
            .into_iter()
            .filter(|&var| std::env::var(var).is_err())
            .collect();

        if !missing_vars.is_empty() {
            return Err(ConfigError::Missing {
                reason: format!("environment variable: {}", missing_vars.join(", "))
            }.into());
        }
        
        Ok(())
    }
}
