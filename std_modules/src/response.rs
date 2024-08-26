use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fs;
use std::process;

pub trait Module {
    type Error: Error;
    type Args: DeserializeOwned + Default;
    fn run(args: Self::Args) -> Result<Response, Self::Error>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Dependency {
    pub module: String,
    pub local_port: i32,
    pub local_ip: String,
    pub local_os: String,
    pub remote_port: i32,
    pub remote_ip: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub dependencies: Vec<Dependency>,
    pub changed: bool,
    pub failed: bool,
    #[serde(flatten)]
    pub extra: serde_json::Map<String, Value>,
}

impl Response {
    pub fn new(dependencies: Vec<Dependency>, changed: bool, failed: bool) -> Self {
        Response {
            dependencies,
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

    pub fn fail(error_msg: &str) -> Self {
        Response{
            dependencies: vec![],
            changed: false,
            failed: true,
            extra: serde_json::Map::from_iter(vec![("error".to_string(), Value::String(error_msg.to_string()))]),
        }
    }
}

pub fn exit_json(response_body: Response) -> ! {
    return_response(&response_body);
    process::exit(0);
}

pub fn fail_json(error_msg: &str) -> ! {
    return_response(&Response::fail(error_msg));
    process::exit(1);
}

fn return_response(response_body: &Response) {
    let response = match serde_json::to_string(&response_body) {
        Ok(json) => json,
        Err(_) => serde_json::to_string(&Response::new(
            vec![],
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
                Err(rr) => {
                    eprintln!("Warning: Could not parse args file as JSON. Using default args.");
                    eprintln!("Args file content: {}", content);
                    eprintln!("Error: {}", rr);
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
    ($module:ident, $args:ty, $err:ty, $run_fn:expr) => {
        pub struct $module;

        impl $crate::response::Module for $module {
            type Error = $err;
            type Args = $args;

            fn run(args: Self::Args) -> Result<$crate::response::Response, Self::Error> {
                $run_fn(args)
            }
        }

        pub fn run() {
            $crate::response::run_module::<$module>();
        }
    };
}


