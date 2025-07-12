pub trait Tool {
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn input_schema(&self) -> serde_json::Value;
    fn call(&self, args: serde_json::Value) -> impl Future<Output = anyhow::Result<String>>;
}
