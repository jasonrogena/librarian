use crate::config::FsWatch;
use crate::error::Error;
use notify::{Op, RawEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::env::consts::OS;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use ttl_cache::TtlCache;

#[cfg(test)]
#[cfg(target_family = "unix")]
mod tests_supported_os;
#[cfg(test)]
#[cfg(target_family = "windows")]
mod tests_unsupported_os;

pub struct Notify<'a> {
    _watcher: RecommendedWatcher,
    on_event_sender: Sender<String>,
    watcher_receiver: Receiver<RawEvent>,
    unwatch_receiver: Receiver<bool>,
    notify_ttl: TtlCache<String, ()>,
    config: &'a Option<FsWatch>,
}

impl<'a> Notify<'a> {
    #[allow(dead_code)]
    pub fn new(
        config: &'a Option<FsWatch>,
        paths: &HashSet<String>,
        on_event_sender: Sender<String>,
    ) -> Result<(Notify<'a>, Sender<bool>), Error> {
        if OS == "windows" {
            return Err(Error::new(
                "Directory watching is currently not supported in this OS".to_string(),
            ));
        }

        let (watcher_sender, watcher_receiver) = channel();
        let mut watcher: RecommendedWatcher = match Watcher::new_raw(watcher_sender) {
            Ok(w) => w,
            Err(e) => {
                return Err(Error::new(format!(
                    "Unable to initialize code for notifying on filesystem changes: {:?}",
                    e
                )))
            }
        };

        for cur_path in paths {
            if let Err(e) = watcher.watch(cur_path, RecursiveMode::Recursive) {
                return Err(Error::new(format!(
                    "Could not watch '{}' for changes: {}",
                    cur_path, e
                )));
            }
        }

        let (unwatch_sender, unwatch_receiver) = channel();
        let notify_ttl: TtlCache<String, ()> = TtlCache::new(100000);
        Ok((
            Notify {
                _watcher: watcher,
                on_event_sender,
                watcher_receiver,
                unwatch_receiver,
                notify_ttl,
                config,
            },
            unwatch_sender,
        ))
    }

    fn should_notify(&self, path: &str) -> bool {
        let config = match self.config {
            Some(c) => c,
            None => return true,
        };

        let notification_ttl = match config.notification_ttl {
            Some(n) => n,
            None => return true,
        };

        if notification_ttl == 0 {
            return true;
        }

        !self.notify_ttl.contains_key(&path.to_string())
    }

    fn record_notify(&mut self, path: &str) {
        let config = match self.config {
            Some(c) => c,
            None => return,
        };

        let notification_ttl = match config.notification_ttl {
            Some(n) => n,
            None => return,
        };

        self.notify_ttl
            .insert(path.to_string(), (), Duration::from_secs(notification_ttl));
    }

    #[allow(dead_code)]
    pub fn watch(&mut self) {
        loop {
            match self.watcher_receiver.recv() {
                Ok(RawEvent {
                    path: Some(path),
                    op: Ok(op),
                    cookie: _,
                }) => {
                    if !path.is_dir() && !op.contains(Op::REMOVE) {
                        if let Some(path_str) = path.as_os_str().to_str() {
                            if !self.should_notify(path_str) {
                                println!("Ignoring {:?} event for '{}' since it occurred within the TTL of last event", op, path_str)
                            } else if self.on_event_sender.send(path_str.to_string()).is_ok() {
                                println!(
                                    "Recording event {:?} notified against '{}'",
                                    op, path_str
                                );
                                self.record_notify(path_str);
                            }
                        }
                    }
                }
                Ok(e) => eprintln!("FS watcher returned a broken event: {:?}", e),
                Err(e) => eprintln!("FS watcher returned an error: {}", e),
            }

            if let Ok(k) = self.unwatch_receiver.try_recv() {
                if k {
                    break;
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn unwatch(unwatch_sender: &Sender<bool>) -> Option<Error> {
        if let Err(e) = unwatch_sender.send(true) {
            return Some(Error::new(format!(
                "Could not notify FS watcher to stop: {}",
                e
            )));
        }
        None
    }
}
