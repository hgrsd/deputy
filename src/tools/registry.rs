use crate::core::Tool;
use super::{ExecCommandTool, ListFilesTool, ReadFilesTool, WriteFileTool};

pub struct ToolRegistry {
    tools: Vec<Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn with_default_tools() -> Self {
        Self {
            tools: vec![
                Box::new(ListFilesTool {}),
                Box::new(ReadFilesTool {}),
                Box::new(WriteFileTool {}),
                Box::new(ExecCommandTool {}),
            ],
        }
    }

    pub fn into_tools(self) -> Vec<Box<dyn Tool>> {
        self.tools
    }
}