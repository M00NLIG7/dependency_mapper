use std::path::PathBuf;
use agent::Config;
use agent::CollectionEngine;
use tokio::sync::mpsc;
use clap::{arg, command, value_parser, Command};

#[cfg(unix)]
use daemonize::Daemonize;
#[cfg(unix)]
use std::fs::File;
#[cfg(unix)]
use rand::{thread_rng, Rng};
#[cfg(unix)]
use rand::distributions::Alphanumeric;

#[cfg(windows)]
use std::process::Command as StdCommand;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use winapi::um::winbase::CREATE_NO_WINDOW;

#[tokio::main]
async fn main() -> agent::Result<()> {
    let matches = command!()
        .arg(
            arg!(-c --config <FILE> "Sets the config file to use")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(-d --detach "Run in detached mode")
                .required(false)
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let config_path = matches.get_one::<PathBuf>("config").unwrap();
    let detach_mode = matches.get_flag("detach");

    if detach_mode {
        return detach_process(config_path);
    }

    run_engine(config_path, detach_mode).await
}

fn detach_process(config_path: &PathBuf) -> agent::Result<()> {
    #[cfg(unix)]
    return unix_detach();

    #[cfg(windows)]
    return windows_detach(config_path);

    #[cfg(not(any(unix, windows)))]
    {
        println!("Detached mode is not supported on this platform.");
        Err(agent::Error::DetachError("Unsupported platform".to_string()))
    }
}

#[cfg(unix)]
fn unix_detach() -> agent::Result<()> {
    let random_id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();

    let stdout_path = format!("/tmp/daemon_{}.out", random_id);
    let stderr_path = format!("/tmp/daemon_{}.err", random_id);
    let pid_file_path = format!("/tmp/daemon_{}.pid", random_id);

    let stdout = File::create(&stdout_path)?;
    let stderr = File::create(&stderr_path)?;

    let daemonize = Daemonize::new()
        .pid_file(&pid_file_path)
        .chown_pid_file(true)
        .working_directory("/tmp")
        .stdout(stdout)
        .stderr(stderr);

    match daemonize.start() {
        Ok(_) => {
            println!("Successfully daemonized. PID file: {}", pid_file_path);
            println!("Stdout: {}", stdout_path);
            println!("Stderr: {}", stderr_path);
            Ok(())
        },
        Err(e) => {
            eprintln!("Error daemonizing: {}", e);
            Err(agent::Error::DetachError(e.to_string()))
        }
    }
}

#[cfg(windows)]
fn windows_detach(config_path: &PathBuf) -> agent::Result<()> {
    let executable = std::env::current_exe()?;
    let mut command = StdCommand::new(executable);
    command.arg("--config").arg(config_path);
    command.creation_flags(CREATE_NO_WINDOW);

    match command.spawn() {
        Ok(child) => {
            println!("Successfully started detached process. PID: {}", child.id());
            Ok(())
        },
        Err(e) => {
            eprintln!("Error starting detached process: {}", e);
            Err(agent::Error::DetachError(e.to_string()))
        }
    }
}

async fn run_engine(config_path: &PathBuf, detach_mode: bool) -> agent::Result<()> {
    let config_str = std::fs::read_to_string(config_path)?;
    println!("Config file loaded: {}", config_path.display());
    let config: Config = serde_yaml::from_str(&config_str)?;

    println!("Starting agent with config: {:?}", config);

    let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
    let mut engine = CollectionEngine::new(config);

    if !detach_mode {
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
            println!("Ctrl+C received, initiating shutdown...");
            shutdown_tx.send(()).await.expect("Failed to send shutdown signal");
        });
    }

    // Run the engine
    engine.run(shutdown_rx).await?;
    
    if !detach_mode {
        println!("Shutdown complete.");
    }
    
    Ok(())
}
