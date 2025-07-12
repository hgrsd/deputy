use std::future::Future;
use std::pin::Pin;

pub trait Tool {
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn input_schema(&self) -> serde_json::Value;
    fn call(&self, args: serde_json::Value) -> Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send + '_>>;
}
