use super::{FsOp, Notification, Notifier};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    collections::HashSet,
    env::consts::OS,
    path::PathBuf,
    sync::mpsc::{channel, Sender},
    thread,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The feature '{0}' is unsupported")]
    UnsupportedFeature(String),
    #[error("An error was thrown by the filesystem notification system")]
    Notify(#[from] notify::Error),
    #[error("An error was thrown while trying to interract with the notification system")]
    Send(#[from] std::sync::mpsc::SendError<bool>),
}

pub(crate) struct Inotify {
    stop_cancellation_token: CancellationToken,
}

impl Inotify {
    pub(crate) fn init() -> Self {
        Inotify {
            stop_cancellation_token: CancellationToken::new(),
        }
    }
    fn convert_op(event_kind: EventKind) -> FsOp {
        let mut fs_op = FsOp::all();
        if !event_kind.is_remove() {
            fs_op.remove(FsOp::REMOVE);
        }

        fs_op
    }
}

impl Notifier for Inotify {
    fn start_watching(
        &mut self,
        paths: &HashSet<PathBuf>,
        notification_sender: Sender<Notification>,
    ) -> Result<(), super::Error> {
        if OS == "windows" {
            return Err(Error::UnsupportedFeature("directory watching".to_string()).into());
        }

        let (watcher_sender, watcher_receiver) = channel();
        let mut watcher: RecommendedWatcher =
            RecommendedWatcher::new(watcher_sender, Config::default())?;

        for cur_path in paths {
            println!("Watching path {:?}", cur_path);
            watcher.watch(&cur_path, RecursiveMode::Recursive)?;
        }

        let stop_cancellation_token = self.stop_cancellation_token.clone();
        thread::spawn(move || loop {
            match watcher_receiver.recv() {
                Ok(Ok(Event {
                    kind,
                    paths,
                    attrs: _,
                })) => {
                    for path in paths {
                        let notification = Notification {
                            path,
                            op: Self::convert_op(kind),
                        };
                        if let Err(e) = notification_sender.send(notification) {
                            eprint!("Unable to notify upwards a filesystem event: {:?}", e);
                        }
                    }
                }
                Ok(e) => eprintln!("FS watcher returned a broken event: {:?}", e),
                Err(e) => {
                    panic!("Inotify watcher returned an error: {}", e);
                }
            }

            if stop_cancellation_token.is_cancelled() {
                break;
            }
        });

        Ok(())
    }

    fn stop_watching(&mut self) {
        self.stop_cancellation_token.cancel();
    }
}
