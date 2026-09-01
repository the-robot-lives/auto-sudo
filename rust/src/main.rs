mod config;
mod decision;
mod shell;
mod sudoers;

use std::path::PathBuf;
use std::process;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::config::Config;

#[derive(Parser, Debug)]
#[command(name = "auto-sudo", version, about = "Configurable sudo prefix helper")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Decide whether a wrapped command should be prefixed with sudo.
    Decide(DecideArgs),
    /// Emit shell wrappers for configured commands.
    Shell(ShellArgs),
    /// Generate and manage sudoers.d snippets for configured commands.
    Sudoers(SudoersArgs),
}

#[derive(Args, Debug)]
struct DecideArgs {
    /// Path to config.yaml.
    #[arg(long)]
    config: Option<PathBuf>,
    /// Explain the matched rule to stderr.
    #[arg(long)]
    explain: bool,
    /// The command is receiving piped stdin.
    #[arg(long)]
    stdin_piped: bool,
    /// The command is writing into a pipe.
    #[arg(long)]
    stdout_piped: bool,
    /// Command and arguments after `--`.
    #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
    command_line: Vec<String>,
}

#[derive(Args, Debug)]
struct ShellArgs {
    /// Shell syntax to emit.
    #[arg(long, value_enum, default_value_t = ShellKind::Zsh)]
    shell: ShellKind,
    /// Path to config.yaml.
    #[arg(long)]
    config: Option<PathBuf>,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum ShellKind {
    Zsh,
    Bash,
}

#[derive(Args, Debug)]
struct SudoersArgs {
    #[command(subcommand)]
    command: SudoersCommand,
}

#[derive(Subcommand, Debug)]
enum SudoersCommand {
    /// Print generated sudoers entries.
    Print(SudoersPrintArgs),
    /// Write generated entries to a sudoers.d file.
    Write(SudoersWriteArgs),
    /// Comment or uncomment a managed entry in an existing file.
    Toggle(SudoersToggleArgs),
    /// Regenerate the managed file with refreshed executable paths/checksums.
    Refresh(SudoersWriteArgs),
    /// Validate an existing sudoers file with visudo.
    Check(SudoersCheckArgs),
}

#[derive(Args, Debug)]
struct SudoersPrintArgs {
    #[arg(long)]
    config: Option<PathBuf>,
    /// Additional command path/name to include.
    #[arg(long = "command")]
    commands: Vec<String>,
}

#[derive(Args, Debug)]
struct SudoersWriteArgs {
    #[arg(long)]
    config: Option<PathBuf>,
    /// sudoers.d target file.
    #[arg(long, default_value = "/etc/sudoers.d/auto-sudo")]
    file: PathBuf,
    /// Additional command path/name to include.
    #[arg(long = "command")]
    commands: Vec<String>,
    /// Append generated entries to the file instead of replacing it.
    #[arg(long)]
    append: bool,
}

#[derive(Args, Debug)]
struct SudoersToggleArgs {
    /// Managed entry id, for example `vim-root`.
    entry_id: String,
    /// sudoers.d target file.
    #[arg(long, default_value = "/etc/sudoers.d/auto-sudo")]
    file: PathBuf,
    /// Enable the entry.
    #[arg(long, conflicts_with = "off")]
    on: bool,
    /// Disable the entry by commenting it out.
    #[arg(long, conflicts_with = "on")]
    off: bool,
}

#[derive(Args, Debug)]
struct SudoersCheckArgs {
    /// sudoers file to validate.
    #[arg(long, default_value = "/etc/sudoers.d/auto-sudo")]
    file: PathBuf,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("auto-sudo: {err}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    match cli.command {
        Command::Decide(args) => {
            let cfg = Config::load(args.config.as_deref())?;
            let (command, rest) = args
                .command_line
                .split_first()
                .ok_or_else(|| "missing command".to_string())?;
            let request = decision::DecisionRequest {
                command,
                args: rest,
                stdin_piped: args.stdin_piped,
                stdout_piped: args.stdout_piped,
            };
            let decision = decision::decide(&cfg, &request)?;
            if args.explain {
                eprintln!("{}", decision.reason);
            }
            print!("{}", decision.prefix);
        }
        Command::Shell(args) => {
            let cfg = Config::load(args.config.as_deref())?;
            let kind = match args.shell {
                ShellKind::Zsh => shell::ShellKind::Zsh,
                ShellKind::Bash => shell::ShellKind::Bash,
            };
            print!("{}", shell::render(&cfg, kind));
        }
        Command::Sudoers(args) => match args.command {
            SudoersCommand::Print(args) => {
                let cfg = Config::load(args.config.as_deref())?;
                print!("{}", sudoers::render(&cfg, &args.commands)?);
            }
            SudoersCommand::Write(args) | SudoersCommand::Refresh(args) => {
                let cfg = Config::load(args.config.as_deref())?;
                let body = sudoers::render(&cfg, &args.commands)?;
                sudoers::write_checked(&args.file, &body, args.append)?;
            }
            SudoersCommand::Toggle(args) => {
                if !args.on && !args.off {
                    return Err("toggle requires --on or --off".to_string());
                }
                sudoers::toggle(&args.file, &args.entry_id, args.on)?;
            }
            SudoersCommand::Check(args) => {
                sudoers::check_file(&args.file)?;
            }
        },
    }
    Ok(())
}
