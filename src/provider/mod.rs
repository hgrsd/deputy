pub mod anthropic;
pub mod session_factory;

use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum Provider {
    Anthropic,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::Anthropic => write!(f, "anthropic"),
        }
    }
}

impl Provider {
    pub fn required_env_vars(&self) -> Vec<&'static str> {
        match self {
            Provider::Anthropic => vec!["ANTHROPIC_API_KEY"],
        }
    }

    pub fn validate_configuration(&self) -> Result<(), String> {
        let required_vars = self.required_env_vars();
        let missing_vars: Vec<_> = required_vars
            .into_iter()
            .filter(|&var| std::env::var(var).is_err())
            .collect();

        if missing_vars.is_empty() {
            Ok(())
        } else {
            let var_list = missing_vars
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            Err(format!(
                "Missing required environment variables for provider {}: {}",
                self, var_list
            ))
        }
    }
}
