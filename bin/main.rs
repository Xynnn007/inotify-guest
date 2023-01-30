use anyhow::*;
use clap::{Arg, App};
use inotify_guest::Inotifier;
use log::info;

const QGS_PATH: &str = "";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let matches = App::new("guest inotifier")
        .author("xynnn007")
        .arg(
            Arg::with_name("qgs")
                .long("qgs")
                .value_name("qgs")
                .help("path to the qgs file.")
                .takes_value(true)
                .default_value(QGS_PATH)
                .required(false),
        )
        .get_matches();

    let cmd = matches.value_of("qgs").expect("get qgs path failed");
    info!("start watch and use qgs path {cmd}");

    let mut notifier = Inotifier::new(cmd)?;
    notifier.start().await
}