use anyhow::*;
use clap::{App, Arg};
use inotify_guest::Multiplexer;
use log::info;

const QGS_SOCKET_PATH: &str = "/var/run/tdx-qgs/qgs.socket";

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let matches = App::new("qgs-multiplexer")
        .author("xynnn007")
        .arg(
            Arg::with_name("qgs")
                .long("qgs")
                .value_name("qgs")
                .help("path to the qgs file.")
                .takes_value(true)
                .default_value(QGS_SOCKET_PATH)
                .required(false),
        )
        .get_matches();

    let qgs_path = matches.value_of("qgs").expect("get qgs path failed");
    info!("start watch and use qgs path {qgs_path}");

    let mut notifier = Multiplexer::new()?;
    notifier.start(qgs_path).await
}
