use std::future::Future;
use std::pin::Pin;

use crate::io::IO;

pub trait Tool {
    fn name(&self) -> String;
    fn description(&self) -> String;
    fn input_schema(&self) -> serde_json::Value;
    fn call<'a>(
        &'a self,
        args: serde_json::Value,
        io: &'a mut Box<dyn IO>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send + 'a>>;
    fn ask_permission(&self, args: serde_json::Value, io: &mut Box<dyn IO>);
    fn permission_id(&self, args: serde_json::Value) -> String;
}