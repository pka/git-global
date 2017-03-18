//! The command line interface for git-global.

use std::io::{Write, stderr};

use clap::{Arg, App, SubCommand};

use config::Config;
use errors::GitGlobalError;
use subcommands;
use subcommands::Report;

/// Returns the definitive `clap::App` instance for git-global.
fn get_clap_app<'a, 'b>() -> App<'a, 'b> {
    App::new("git-global")
        .version(crate_version!())
        .author("Eric Petersen <eric@ericpetersen.io>")
        .about("git subcommand for working with all git repos on a machine")
        .arg(Arg::with_name("json")
            .long("json")
            .help("Output results in JSON."))
        .subcommand(SubCommand::with_name("info")
            .about("show meta-information about git-global"))
        .subcommand(SubCommand::with_name("list")
            .about("lists all git repos on your machine [the default]"))
        .subcommand(SubCommand::with_name("scan")
            .about("update cache of git repos on your machine"))
        .subcommand(SubCommand::with_name("status")
            .about("shows status of all git repos"))
}

/// Runs the appropriate git-global subcommand based on command line arguments.
///
/// As the effective binary entry point for `git-global`, prints results to
/// `STDOUT` and returns an exit code.
pub fn run_from_command_line() -> i32 {
    let config = Config::from_gitconfig();
    let clap_app = get_clap_app();
    let matches = clap_app.get_matches();
    let use_json = matches.is_present("json");
    let report_result = match matches.subcommand_name() {
        Some("info") => subcommands::info::run(&config),
        Some("list") => subcommands::list::run(&config),
        Some("scan") => subcommands::scan::run(&config),
        Some("status") => subcommands::status::run(&config),
        Some(cmd) => Err(GitGlobalError::BadSubcommand(cmd.to_string())),
        None => subcommands::status::run(&config),
    };
    match report_result {
        Ok(report) => show_report(report, use_json),
        Err(err) => show_error(err, use_json),
    }
}

/// Writes report to STDOUT, as either text or JSON, and returns `0`.
fn show_report(report: Report, use_json: bool) -> i32 {
    if use_json {
        report.print_json();
    } else {
        report.print();
    }
    0
}

/// Writes errors to STDERR, as either text or JSON, and returns `1`.
fn show_error(error: GitGlobalError, use_json: bool) -> i32 {
    if use_json {
        let json = object!{
            "error" => true,
            "message" => format!("{}", error)
        };
        writeln!(&mut stderr(), "{:#}", json).expect("failed write to STDERR");
    } else {
        writeln!(&mut stderr(), "{}", error).expect("failed write to STDERR");
    }
    1
}
