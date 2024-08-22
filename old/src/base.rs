use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fs;
use std::process;

pub trait Module {
    type Args: DeserializeOwned + Default;
    fn run(args: Self::Args) -> Result<Response, Box<dyn Error>>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub msg: String,
    pub changed: bool,
    pub failed: bool,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

impl Response {
    pub fn new(msg: String, changed: bool, failed: bool) -> Self {
        Response {
            msg,
            changed,
            failed,
            extra: serde_json::Map::new(),
        }
    }

    pub fn add_extra<T: Serialize>(&mut self, key: &str, value: &T) -> Result<(), Box<dyn Error>> {
        let value = serde_json::to_value(value)?;
        self.extra.insert(key.to_string(), value);
        Ok(())
    }
}

pub fn exit_json(response_body: Response) -> ! {
    return_response(&response_body);
    process::exit(0);
}

pub fn fail_json(msg: &str) -> ! {
    let response = Response::new(msg.to_string(), false, true);
    return_response(&response);
    process::exit(1);
}

fn return_response(response_body: &Response) {
    let response = match serde_json::to_string(&response_body) {
        Ok(json) => json,
        Err(_) => serde_json::to_string(&Response::new(
            "Invalid response object".to_string(),
            false,
            true,
        ))
        .unwrap(),
    };
    println!("{}", response);
}
pub fn run_module<T: Module>() {
    let args = match std::env::var("ARGS_FILE") {
        Ok(args_file) => match fs::read_to_string(&args_file) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(args) => args,
                Err(_) => {
                    eprintln!("Warning: Could not parse args file as JSON. Using default args.");
                    T::Args::default()
                }
            },
            Err(_) => {
                eprintln!("Warning: Could not read args file. Using default args.");
                T::Args::default()
            }
        },
        Err(_) => {
            eprintln!("No args file provided. Using default args.");
            T::Args::default()
        }
    };

    match T::run(args) {
        Ok(response) => exit_json(response),
        Err(e) => fail_json(&format!("Module execution failed: {}", e)),
    }
}

#[macro_export]
macro_rules! implement_module {
    ($module:ident, $args:ty, $run_fn:expr) => {
        pub struct $module;

        impl $crate::base::Module for $module {
            type Args = $args;

            fn run(args: Self::Args) -> Result<$crate::base::Response, Box<dyn std::error::Error>> {
                $run_fn(args)
            }
        }

        pub fn run() {
            $crate::base::run_module::<$module>();
        }
    };
}
