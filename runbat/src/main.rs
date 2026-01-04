use clap::Parser;
use std::fs;

#[derive(Parser, Debug)]
#[command(name = "host_exec")]
#[command(about = "Generate execution script from commands file")]
struct Args {
    /// Path to the commands file
    #[arg(long, default_value = "commands.txt")]
    commands: String,

    /// Path to the log folder
    #[arg(long, default_value = "logs")]
    log_folder: String,

    /// Path to the output script file
    #[arg(long, default_value = "tmp_run.sh")]
    output: String,
}

#[derive(Debug)]
struct Command {
    name: Option<String>,
    content: String,
}

fn parse_commands_file(
    file_path: &str,
) -> Result<(Option<String>, Vec<Command>), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let lines: Vec<&str> = content.lines().collect();

    let mut pre_build: Option<String> = None;
    let mut commands: Vec<Command> = Vec::new();
    let mut current_command: Option<Command> = None;
    let mut found_first_marker = false;

    for line in lines {
        if line.starts_with("##") {
            // Save previous command if exists
            if let Some(cmd) = current_command.take() {
                commands.push(cmd);
            }

            // Extract command name (text after ##)
            let name = line.strip_prefix("##").map(|s| s.trim().to_string());
            current_command = Some(Command {
                name,
                content: String::new(),
            });
            found_first_marker = true;
        } else {
            if !found_first_marker {
                // This is part of the pre-build script
                if pre_build.is_none() {
                    pre_build = Some(String::new());
                }
                if let Some(ref mut pb) = pre_build {
                    if !pb.is_empty() {
                        pb.push('\n');
                    }
                    pb.push_str(line);
                }
            } else {
                // This is part of a command
                if let Some(ref mut cmd) = current_command {
                    if !cmd.content.is_empty() {
                        cmd.content.push('\n');
                    }
                    cmd.content.push_str(line);
                }
            }
        }
    }

    // Don't forget the last command
    if let Some(cmd) = current_command {
        commands.push(cmd);
    }

    // Trim pre-build script
    if let Some(ref mut pb) = pre_build {
        *pb = pb.trim().to_string();
    }

    Ok((pre_build, commands))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Parse commands file
    let (pre_build, commands) = parse_commands_file(&args.commands)?;

    let log_folder = format!("{}", args.log_folder);
    let script_content =
        generate_script_content(&log_folder, &pre_build.unwrap_or_default(), &commands);
    fs::write(args.output.clone(), script_content)?;

    println!("Generated script: {}", args.output);
    println!("Log folder: {}", log_folder);

    println!(
        "\nRun Following Command to Start Guests: \n`bash {}`\n",
        args.output
    );

    Ok(())
}

fn generate_script_content(log_folder: &str, pre_build: &str, commands: &[Command]) -> String {
    let template = TEMPLATE.to_string();
    let mut script_content = template;

    let command_template = r#"
    ## {{command_name}}
    log_file="{{log_folder}}/$datetime/{{command_name}}.log"

    echo -e "${GREEN}Starting {{command_name}} (logs -> $log_file)...${NC}"
    
    RUST_BACKTRACE=full RUST_LOG=info {{command_content}} > "$log_file" 2>&1 &

    pids+=($!)
    
    "#;

    script_content = script_content.replace("{{pre_build}}", pre_build);

    let mut guest_commands = String::new();
    for (id, command) in commands.iter().enumerate() {
        let mut cmd = command.content.clone();
        cmd = cmd.trim().to_string();
        // assert cmd is one line
        assert!(cmd.lines().count() == 1, "Command must be one line");

        let mut command_content = command_template.to_string();
        command_content = command_content.replace(
            "{{command_name}}",
            command
                .name
                .as_deref()
                .unwrap_or(&format!("Guest-{}", id + 1)),
        );
        command_content = command_content.replace("{{command_content}}", &cmd);
        command_content = command_content.replace("{{log_folder}}", log_folder);
        guest_commands.push_str(&command_content);
    }

    script_content = script_content.replace("{{guest_commands}}", &guest_commands);
    script_content = script_content.replace("{{num_guests}}", &commands.len().to_string());
    script_content = script_content.replace("{{log_folder}}", log_folder);

    script_content
}

const TEMPLATE: &str = r#"
#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color


datetime=$(date +%Y%m%d-%H%M%S)

function stop_all_jobs() {
    echo -e "\n\n"
    echo -e ${YELLOW}Stopping All Jobs
    kill 0
    wait
    echo "All jobs stopped"
}

set -e
trap "exit" INT TERM
trap "stop_all_jobs" EXIT


# Ensure logs directory exists
mkdir -p {{log_folder}}/$datetime

{{pre_build}}

echo "Starting {{num_guests}} Guest instances..."

# Array to store process IDs
pids=()

{{guest_commands}}


jobs -pr

echo "All {{num_guests}} Guest instances are running!"
echo "Press Ctrl+C to stop all instances"

echo "Run Following Command to View Logs:"
echo "multitail -s 2 {{log_folder}}/$datetime/*"

# Trap Ctrl+C
# Wait for all background processes
for pid in $pids
do
  wait $pid
done
"#;
