use anyhow::anyhow;
use argh::FromArgs;
use execute::Execute;
use healthchecks::ping::get_config;
use std::env::var;
use std::process::Command;

#[derive(Debug)]
struct Settings {
    token: String,
    ua: Option<String>,
}

/// Report results of arbitrary commands to https://healthchecks.io
#[derive(FromArgs)]
struct Cli {
    /// starts a timer before running the command
    #[argh(switch, short = 't')]
    timer: bool,

    /// command to execute and monitor
    #[argh(option, short = 'X', long = "exec")]
    command: String,
}

fn main() -> anyhow::Result<()> {
    let ua = match var("HEALTHCHECKS_USERAGENT") {
        Ok(f) => Some(f),
        Err(_) => None,
    };
    let settings = Settings {
        token: var("HEALTHCHECKS_CHECK_ID").expect("HEALTHCHECKS_TOKEN must be set to run monitor"),
        ua,
    };

    let cli: Cli = argh::from_env();
    let cmds = cli.command.split(" ").collect::<Vec<&str>>();
    if cmds.is_empty() {
        return Err(anyhow!("Command must be provided!"))
    }
    let mut config = get_config(&settings.token)?;
    if let Some(user_agent) = settings.ua {
        config = config.set_user_agent(&user_agent)
    }
    if cli.timer {
        config.start_timer();
    }
    let mut command = Command::new(&cmds.get(0).expect("Should have at least one command"));
    for cmd in cmds.iter().skip(1) {
        command.arg(cmd);
    }
    if let Some(exit_code) = command.execute_output()?.status.code() {
        if exit_code == 0 {
            config.report_success();
        } else {
            config.report_failure();
        }
    } else {
        eprintln!("Interrupted!");
    };
    Ok(())
}
