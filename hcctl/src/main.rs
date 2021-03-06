#[macro_use]
extern crate prettytable;

use std::env::var;
use std::time::SystemTime;

use anyhow::anyhow;
use chrono::prelude::{DateTime, Datelike, Timelike};
use chrono::Duration;
use clap::{App, AppSettings, Arg};
use prettytable::{format, Table};

use healthchecks::manage;

#[derive(Debug)]
struct Settings {
    token: String,
    ua: Option<String>,
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

    let matches = App::new("hcctl")
        .about("Command-line tool for interacting with a https://healthchecks.io account")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(App::new("list").about("Lists the checks in your account with their last ping"))
        .subcommand(
            App::new("pings")
                .about("Get the last 10 pings for the given check ID")
                .args(&[Arg::new("check_id")
                    .about("ID of the check whose pings are being fetched")
                    .required(true)
                    .index(1)]),
        )
        .get_matches();

    match matches.subcommand().unwrap() {
        ("list", _) => list(settings)?,
        ("pings", matches) => pings(settings, matches.value_of("check_id").unwrap())?,
        (cmd, _) => return Err(anyhow!("unknown subcommand: {}", cmd)),
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
        let id = check.id().unwrap_or_else(|| "-".to_owned());
        table.add_row(row![id, check.name, date]);
    }

    table.printstd();

    Ok(())
}
