use super::{Notification, Notifier};
use std::{collections::HashSet, env::consts::OS, path::PathBuf, sync::mpsc::Sender};
use tokio_util::sync::CancellationToken;
#[cfg(target_os = "linux")]
use {
    super::FsOp,
    fanotify::high_level::{
        FanEvent, Fanotify, FanotifyMode, FAN_ACCESS, FAN_CLOSE, FAN_CLOSE_WRITE,
        FAN_EVENT_ON_CHILD, FAN_MODIFY, FAN_ONDIR,
    },
    nix::poll::{poll, PollFd, PollFlags},
    std::os::fd::AsFd,
    std::os::fd::AsRawFd,
    std::thread,
};

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The feature '{0}' is unsupported")]
    UnsupportedFeature(String),
    #[error("An error was thrown by the filesystem notification system: {0}")]
    Fanotify(String),
    #[error("An error was thrown while trying to interract with the notification system")]
    Send(#[from] std::sync::mpsc::SendError<bool>),
}

#[allow(dead_code)]
pub(crate) struct FanotifyNotifier {
    stop_cancellation_token: CancellationToken,
}

impl FanotifyNotifier {
    pub(crate) fn new() -> Self {
        FanotifyNotifier {
            stop_cancellation_token: CancellationToken::new(),
        }
    }

    #[cfg(target_os = "linux")]
    fn convert_op(events: Vec<FanEvent>) -> FsOp {
        let mut fs_op = FsOp::OTHER;
        for cur_event in events {
            if cur_event == FanEvent::Delete || cur_event == FanEvent::DeleteSelf {
                fs_op.insert(FsOp::REMOVE);
            }
        }

        fs_op
    }

    #[cfg(target_os = "linux")]
    fn start_watching_linux(
        &mut self,
        paths: &HashSet<PathBuf>,
        notification_sender: Sender<Notification>,
    ) -> Result<(), Error> {
        let stop_cancellation_token = self.stop_cancellation_token.clone();
        let local_paths = paths.clone();
        let fd = match Fanotify::new_nonblocking(FanotifyMode::NOTIF) {
            Ok(f) => f,
            Err(e) => {
                return Err(Error::Fanotify(
                    format!("While initializing Fanotify object: {}", e).to_string(),
                ))
            }
        };
        for cur_path in local_paths {
            fd.add_path(
                FAN_ACCESS
                    | FAN_CLOSE
                    | FAN_EVENT_ON_CHILD
                    | FAN_MODIFY
                    | FAN_ONDIR
                    | FAN_CLOSE_WRITE,
                &cur_path,
            )
            .map_err(|err| {
                Error::Fanotify(
                    format!("While adding path '{}': {}", cur_path.display(), err).to_string(),
                )
            })?;
        }
        thread::spawn(move || {
            let fd_handle = fd.as_fd();
            let mut fds = [PollFd::new(fd_handle.as_raw_fd(), PollFlags::POLLIN)];
            loop {
                let poll_num = poll(&mut fds, -1).unwrap();
                if poll_num > 0 {
                    for event in fd.read_event() {
                        let notification = Notification {
                            path: PathBuf::from(event.path),
                            op: Self::convert_op(event.events),
                        };
                        if let Err(e) = notification_sender.send(notification) {
                            eprint!("Unable to notify upwards a filesystem event: {:?}", e);
                        }
                    }
                } else {
                    eprintln!("poll_num <= 0!");
                    break;
                }

                if stop_cancellation_token.is_cancelled() {
                    println!("Cancelling watching using FanotifyNotifier");
                    break;
                }
            }
        });

        Ok(())
    }
}

impl Notifier for FanotifyNotifier {
    fn start_watching(
        &mut self,
        _paths: &HashSet<PathBuf>,
        _notification_sender: Sender<Notification>,
    ) -> Result<(), super::Error> {
        #[cfg(target_os = "linux")]
        self.start_watching_linux(_paths, _notification_sender)?;

        Ok(())
    }

    fn stop_watching(&mut self) {
        self.stop_cancellation_token.cancel();
    }

    fn is_supported(&self) -> bool {
        if OS != "linux" {
            return false;
        }

        true
    }
}
