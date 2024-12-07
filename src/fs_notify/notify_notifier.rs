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
    #[error("An error was thrown by the filesystem notification system")]
    Notify(#[from] notify::Error),
    #[error("An error was thrown while trying to interract with the notification system")]
    Send(#[from] std::sync::mpsc::SendError<bool>),
}

pub(crate) struct NotifyNotifier {
    stop_cancellation_token: CancellationToken,
}

impl NotifyNotifier {
    pub(crate) fn new() -> Self {
        NotifyNotifier {
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

impl Notifier for NotifyNotifier {
    fn start_watching(
        &mut self,
        paths: &HashSet<PathBuf>,
        notification_sender: Sender<Notification>,
    ) -> Result<(), super::Error> {
        let stop_cancellation_token = self.stop_cancellation_token.clone();
        let local_paths = paths.clone();
        thread::spawn(move || {
            let (watcher_sender, watcher_receiver) = channel();
            let mut watcher: RecommendedWatcher =
                match RecommendedWatcher::new(watcher_sender, Config::default()) {
                    Ok(w) => w,
                    Err(e) => panic!("NotifyNotifier returned an error while initializing: {}", e),
                };

            for cur_path in local_paths {
                println!("Watching path {:?}", cur_path);
                if let Err(e) = watcher.watch(&cur_path, RecursiveMode::Recursive) {
                    panic!("NotifyNotifier returned an error while attempting to watch a directory: {}", e);
                }
            }
            loop {
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
                    Ok(e) => eprintln!("NotifyNotifier returned a broken event: {:?}", e),
                    Err(e) => {
                        panic!("NotifyNotifier returned an error: {}", e);
                    }
                }

                if stop_cancellation_token.is_cancelled() {
                    println!("Cancelling watching using NotifyNotifier");
                    break;
                }
            }
        });

        Ok(())
    }

    fn stop_watching(&mut self) {
        self.stop_cancellation_token.cancel();
    }

    fn is_supported(&self) -> bool {
        if OS == "windows" {
            return false;
        }

        true
    }
}
