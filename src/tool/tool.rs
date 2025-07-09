pub trait Tool {
    const NAME: &'static str;
    type Error: std::error::Error + Send + Sync + 'static;

    fn description(&self) -> String;
    fn input_schema(&self) -> serde_json::Value;
    fn call(&self, args: serde_json::Value) -> impl std::future::Future<Output = anyhow::Result<String>> + Send + Sync;
}
