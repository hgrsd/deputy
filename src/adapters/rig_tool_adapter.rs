use anyhow::Result;
use rig::completion::ToolDefinition;

use crate::tool::Tool;

pub struct RigToolAdapter<T: Tool> {
    tool: T,
}

impl<T: Tool> RigToolAdapter<T> {
    pub fn new(tool: T) -> Self {
        Self { tool }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Tool adapter error: {0}")]
pub struct RigToolAdapterError(#[from] anyhow::Error);

impl<T: Tool + Send + Sync> rig::tool::Tool for RigToolAdapter<T> {
    const NAME: &'static str = T::NAME;

    type Error = RigToolAdapterError;
    type Args = serde_json::Value;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: T::NAME.to_string(),
            description: self.tool.description(),
            parameters: self.tool.input_schema(),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let result = self.tool.call(args).await?;
        Ok(result)
    }
}
