//! use to inotify the creation of new tdx guest's vsocks

use std::collections::BTreeMap;

use anyhow::*;
use inotify::{EventMask, Inotify, WatchMask};
use log::{info, warn};
use tokio::process::Child;

const DRAGONBALL_WORKDIR: &str = "/var/lib/vc/dragonball";

const VSOCK_SUFFIX: &str = "/root/kata.hvsock_";

const VSOCK_PREFIX: &str = "/var/lib/vc/dragonball/";

pub struct Inotifier {
    exec_cmd: String,
    inotify: Inotify,
    qgs_map: BTreeMap<String, Child>,
}

impl Inotifier {
    pub fn new(cmd: &str) -> Result<Self> {
        let inotify = Inotify::init().context("inotify init failed")?;
        Ok(Self {
            exec_cmd: cmd.into(),
            inotify,
            qgs_map: BTreeMap::new(),
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        self.inotify
            .add_watch(DRAGONBALL_WORKDIR, WatchMask::CREATE | WatchMask::DELETE)
            .context("watch failed")?;

        let mut buffer = [0; 4096];
        let events = self
            .inotify
            .read_events_blocking(&mut buffer)
            .context("Error while reading events")?;

        for event in events {
            if event.mask.contains(EventMask::CREATE) {
                let name = event
                    .name
                    .ok_or_else(|| anyhow!("inotify catches empty filename"))?
                    .to_string_lossy()
                    .to_string();
                if name.ends_with(VSOCK_SUFFIX) && name.starts_with(VSOCK_PREFIX) {
                    let id = name.trim_start_matches(VSOCK_PREFIX).trim_end_matches(VSOCK_SUFFIX);
                    let cmd = tokio::process::Command::new(&self.exec_cmd)
                        .spawn()
                        .context("spawn qgs failed")?;
                    self.qgs_map.insert(id.into(), cmd);
                }
            } else if event.mask.contains(EventMask::DELETE) {
                let name = event
                    .name
                    .ok_or_else(|| anyhow!("inotify catches empty filename"))?
                    .to_string_lossy()
                    .to_string();
                if name.ends_with(VSOCK_SUFFIX) && name.starts_with(VSOCK_PREFIX) {
                    let id = name.trim_start_matches(VSOCK_PREFIX).trim_end_matches(VSOCK_SUFFIX);

                    match self.qgs_map.get_mut(id) {
                        Some(cmd) => {
                            cmd.kill().await.context("guest qgs quit failed")?;
                            info!("guest id {id} qgs exited.");
                        }
                        None => warn!("guest id {id} exits but no qgs process found"),
                    }
                }
            }
        }

        Ok(())
    }
}
