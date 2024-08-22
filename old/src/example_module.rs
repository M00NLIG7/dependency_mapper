use crate::{implement_module, base::{Response}};
use serde::Deserialize;
use crate::base::Module;
use crate::base::run_module;
use std::error::Error;

#[derive(Deserialize)]
struct HelloArgs {
    name: Option<String>,
}

fn run_hello(args: HelloArgs) -> Result<Response, Box<dyn std::error::Error>> {
    let name = args.name.unwrap_or_else(|| "World".to_string());
    Ok(Response {
        msg: format!("Hello, {}!", name),
        changed: false,
        failed: false,
        extra: serde_json::Map::new(),
    })
}

implement_module!(HelloModule, HelloArgs, run_hello);
