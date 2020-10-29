#[macro_use]
extern crate prettytable;

use argh::FromArgs;
use std::env::var;
use std::time::SystemTime;

use chrono::prelude::{DateTime, Datelike, Timelike};
use chrono::Duration;
use prettytable::{format, Table};

use healthchecks::manage;

#[derive(Debug)]
struct Settings {
    token: String,
    ua: Option<String>,
}

/// Command-line tool for interacting with a https://healthchecks.io account
#[derive(FromArgs)]
struct Args {
    #[argh(subcommand)]
    command: Command,
}

#[derive(FromArgs)]
#[argh(subcommand)]
enum Command {
    List(ListChecks),
    Pings(GetPings),
}

/// List all checks associated with an account
#[derive(FromArgs)]
#[argh(subcommand, name = "list")]
struct ListChecks {}

/// Get logged pings for a given check
#[derive(FromArgs)]
#[argh(subcommand, name = "pings")]
struct GetPings {
    /// UUID for the check whose pings need to be logged
    #[argh(positional)]
    check_id: String,
}

fn main() -> anyhow::Result<()> {
    let ua = match var("HEALTHCHECKS_USERAGENT") {
        Ok(f) => Some(f),
        Err(_) => None,
    };
    let settings = Settings {
        token: var("HEALTHCHECKS_TOKEN").expect("HEALTHCHECKS_TOKEN must be set to run monitor"),
        ua,
    };

    let cli: Args = argh::from_env();

    match cli.command {
        Command::List(_) => list(settings)?,
        Command::Pings(args) => pings(settings, &args.check_id)?,
    }

    Ok(())
}

fn pings(settings: Settings, check_id: &str) -> anyhow::Result<()> {
    let api = manage::get_config(settings.token, settings.ua)?;
    let mut pings = api.list_logged_pings(check_id)?;
    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row!["Number", "Time", "Type", "Duration"]);
    pings.truncate(10);
    for ping in pings {
        let utc_time = DateTime::parse_from_rfc3339(&ping.date)?.naive_utc();
        let date = utc_time.date();
        let time = utc_time.time();
        let time_str = format!(
            "{}/{} {}:{}",
            date.day(),
            date.month(),
            time.hour(),
            time.minute(),
        );
        let duration_str = if let Some(duration) = ping.duration {
            format!("{0:.3} sec", duration)
        } else {
            "".to_owned()
        };
        table.add_row(row![
            format!("#{}", ping.n),
            time_str,
            ping.type_field,
            duration_str
        ]);
    }
    table.printstd();
    Ok(())
}

fn list(settings: Settings) -> anyhow::Result<()> {
    let api = manage::get_config(settings.token, settings.ua)?;
    let checks = api.get_checks()?;

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row!["ID", "Name", "Last Ping"]);

    let now = SystemTime::now();
    for check in checks {
        let date = if let Some(ref date_str) = check.last_ping {
            let date = DateTime::parse_from_rfc3339(&date_str)?;
            let duration = Duration::from_std(now.duration_since(SystemTime::from(date))?)?;
            format!(
                "{} hour(s) and {} minute(s) ago",
                duration.num_hours(),
                duration.num_minutes()
            )
        } else {
            "-".to_owned()
        };
        let id = check.id().unwrap_or("-".to_owned());
        table.add_row(row![id, check.name, date]);
    }

    table.printstd();

    Ok(())
}
