use crate::config::FsWatch;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use ttl_cache::TtlCache;

mod notify_notifier;
mod fanotify_notifier;

#[cfg(test)]
#[cfg(target_family = "unix")]
mod tests_supported_os;
#[cfg(test)]
#[cfg(target_family = "windows")]
mod tests_unsupported_os;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("An error was thrown by NotifyNotifier")]
    NotifyNotifier(#[from] notify_notifier::Error),
    #[error("An error was thrown by NotifyNotifier")]
    FanotifyNotifier(#[from] fanotify_notifier::Error),
    #[error("An error was thrown while trying to interract with the notification system")]
    Send(#[from] std::sync::mpsc::SendError<bool>),
    #[error("The watching feature is unsupported")]
    FeatureNotSupported(),
}

pub struct Notify<'a> {
    on_event_sender: Sender<String>,
    unwatch_receiver: Receiver<bool>,
    notify_ttl: TtlCache<String, ()>,
    config: &'a Option<FsWatch>,
    paths: HashSet<PathBuf>,
}

bitflags! {
/// Holds a set of bit flags representing the actions for the event.
/// For a list of possible values, have a look at the [notify::op](index.html) documentation.
/// Multiple actions may be delivered in a single event.

    #[derive(Debug)]
    pub struct FsOp: u32 {
/// Removed
        const REMOVE       = 0b000_0001;
/// Catch-all for any other
        const OTHER      = 0b000_0010;
    }
}

#[derive(Debug)]
struct Notification {
    path: PathBuf,
    op: FsOp,
}

trait Notifier {
    fn start_watching(
        &mut self,
        paths: &HashSet<PathBuf>,
        notification_sender: Sender<Notification>,
    ) -> Result<(), Error>;
    fn stop_watching(&mut self);
    fn is_supported(&self) -> bool;
}

impl<'a> Notify<'a> {
    #[allow(dead_code)]
    pub fn new(
        config: &'a Option<FsWatch>,
        paths: HashSet<PathBuf>,
        on_event_sender: Sender<String>,
    ) -> Result<(Notify<'a>, Sender<bool>), Error> {
        let (unwatch_sender, unwatch_receiver) = channel();
        let notify_ttl: TtlCache<String, ()> = TtlCache::new(100000);
        Ok((
            Notify {
                on_event_sender,
                unwatch_receiver,
                notify_ttl,
                config,
                paths,
            },
            unwatch_sender,
        ))
    }

    fn should_notify(&self, path: &str) -> bool {
        let config = match self.config {
            Some(c) => c,
            None => return true,
        };

        let min_command_exec_freq = match config.min_command_exec_freq {
            Some(n) => n,
            None => return true,
        };

        if min_command_exec_freq == 0 {
            return true;
        }

        !self.notify_ttl.contains_key(&path.to_string())
    }

    fn record_notify(&mut self, path: &str) {
        let config = match self.config {
            Some(c) => c,
            None => return,
        };

        let min_command_exec_freq = match config.min_command_exec_freq {
            Some(n) => n,
            None => return,
        };

        self.notify_ttl.insert(
            path.to_string(),
            (),
            Duration::from_secs(min_command_exec_freq),
        );
    }

    #[allow(dead_code)]
    pub fn watch(&mut self) -> Result<(), Error> {
        let (notification_sender, notification_receiver) = channel();
        let mut watching = false;
        let mut i = notify_notifier::NotifyNotifier::new();
        if i.is_supported() {
            i.start_watching(&self.paths, notification_sender.clone())?;
            watching = true;
        }
        let mut fa = fanotify_notifier::FanotifyNotifier::new();
        if fa.is_supported() {
            fa.start_watching(&self.paths, notification_sender)?;
            watching = true;
        }

        if !watching {
            return Err(Error::FeatureNotSupported());
        }

        loop {
            match notification_receiver.recv() {
                Ok(Notification { path, op }) => {
                    if !path.is_dir() && !op.contains(FsOp::REMOVE) {
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
                Err(e) => {
                    panic!("FS watcher returned an error: {}", e);
                }
            }

            if let Ok(k) = self.unwatch_receiver.try_recv() {
                if k {
                    i.stop_watching();
                    break;
                }
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn unwatch(unwatch_sender: &Sender<bool>) -> Result<(), Error> {
        unwatch_sender.send(true)?;
        Ok(())
    }
}
