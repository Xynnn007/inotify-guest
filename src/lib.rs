//! use to inotify the creation of new tdx guest's vsocks

use std::{collections::BTreeMap, ffi::OsString};

use anyhow::*;
use futures::TryStreamExt;
use inotify::{Event, EventMask, Inotify, WatchMask};
use log::{info, warn};
use tokio::{io, net::UnixStream, task::JoinHandle};

const DRAGONBALL_WORKDIR: &str = "/var/lib/vc/dragonball";

const VSOCK_SUFFIX: &str = "/root/kata.hvsock_";

const VSOCK_PREFIX: &str = "/var/lib/vc/dragonball/";

pub struct Multiplexer {
    inotify: Inotify,
    qgs_map: BTreeMap<String, JoinHandle<()>>,
}

impl Multiplexer {
    pub fn new() -> Result<Self> {
        let inotify = Inotify::init().context("inotify init failed")?;
        Ok(Self {
            inotify,
            qgs_map: BTreeMap::new(),
        })
    }

    pub async fn start(&mut self, qgs_socket_path: &str) -> Result<()> {
        self.inotify
            .add_watch(DRAGONBALL_WORKDIR, WatchMask::CREATE | WatchMask::DELETE)
            .context("watch failed")?;

        let mut buffer = [0; 4096];
        let mut events = self
            .inotify
            .event_stream(&mut buffer)
            .context("Error while reading events")?;

        while let std::result::Result::Ok(event) = events.try_next().await {
            let event = match event {
                Some(e) => e,
                None => {
                    warn!("get an empty event.");
                    continue;
                }
            };

            if event.mask.contains(EventMask::CREATE) {
                let name = get_guest_id(event)?;
                if name.ends_with(VSOCK_SUFFIX) && name.starts_with(VSOCK_PREFIX) {
                    let id = name
                        .trim_start_matches(VSOCK_PREFIX)
                        .trim_end_matches(VSOCK_SUFFIX);

                    info!("Create new guest id {id}");
                    let qgs_socket = UnixStream::connect(qgs_socket_path)
                        .await
                        .context("dragonball unix socket bind")?;
                    let (mut qgs_r, mut qgs_w) = qgs_socket.into_split();

                    let guest_socket = UnixStream::connect(&name)
                        .await
                        .context("connect guest unix socket")?;
                    let (mut guest_r, mut guest_w) = guest_socket.into_split();

                    let slot = tokio::task::spawn(async move {
                        let _ = tokio::try_join!(
                            async {
                                io::copy(&mut qgs_r, &mut guest_w)
                                    .await
                                    .context("qgs receive guest request failed")
                            },
                            async {
                                io::copy(&mut guest_r, &mut qgs_w)
                                    .await
                                    .context("qgs send guest response failed")
                            }
                        );
                    });

                    self.qgs_map.insert(id.into(), slot);
                }
            } else if event.mask.contains(EventMask::DELETE) {
                let name = get_guest_id(event)?;

                if name.ends_with(VSOCK_SUFFIX) && name.starts_with(VSOCK_PREFIX) {
                    let id = name
                        .trim_start_matches(VSOCK_PREFIX)
                        .trim_end_matches(VSOCK_SUFFIX);

                    info!("Remove guest id {id}");

                    match self.qgs_map.get_mut(id) {
                        Some(slot) => {
                            slot.abort();
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

fn get_guest_id(event: Event<OsString>) -> Result<String> {
    let res = event
        .name
        .ok_or_else(|| anyhow!("inotify catches empty filename"))?
        .to_string_lossy()
        .to_string();
    Ok(res)
}
