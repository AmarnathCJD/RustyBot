use std::process::{Command, Stdio};
use std::io::Read;
use std::time::Instant;
use std::time::Duration;
use grammers_client::{Client}

fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let milliseconds = duration.subsec_millis();

    format!("{}.{:02}s", seconds, milliseconds)
}

fn execute_command(command: &str) -> (String, String, String, String, String) {
    let parts: Vec<&str> = command.split_whitespace().collect();
    let (system_command, args) = parts.split_at(1);

    let start_time = Instant::now();

    let mut output = Command::new(system_command[0])
        .args(args)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to execute command");

    let pid = output.id();
    let status = output.wait().expect("Failed to wait for command");
    let exit_code = status.code().unwrap_or(-1);

    let mut stderr = String::new();
    output.stderr.unwrap().read_to_string(&mut stderr).expect("Failed to read stderr");

    let mut stdout = String::new();
    output.stdout.unwrap().read_to_string(&mut stdout).expect("Failed to read stdout");

    let end_time = Instant::now();
    let execution_time = end_time.duration_since(start_time);

    (
        pid.to_string(),
        exit_code.to_string(),
        stderr,
        stdout,
        format_duration(execution_time),
    )
}

async fn handle_exec(client: Client, message: grammers_client::types::Message) -> Result {
    Ok(())
}
