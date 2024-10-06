use super::{Notification, Notifier};
#[cfg(target_os = "linux")]
use {
    fanotify::{high_level::{Fanotify, FanotifyMode,FanEvent}, low_level::{FAN_CLOSE_WRITE, FAN_CREATE, FAN_MODIFY, FAN_MOVE_SELF}},
    nix::poll::{poll, PollFd, PollFlags},
    std::thread,
    super::FsOp,
    std::os::fd::AsFd,
    std::os::fd::AsRawFd,
};
use std::{
    collections::HashSet,
    env::consts::OS,
    path::PathBuf,
    sync::mpsc::Sender,
};
use tokio_util::sync::CancellationToken;

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The feature '{0}' is unsupported")]
    UnsupportedFeature(String),
    //#[error("An error was thrown by the filesystem notification system")]
    //Faotify(#[from] fanotify::Error),
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
        thread::spawn(move || {
            let fd = match Fanotify::new_nonblocking(FanotifyMode::NOTIF) {
                Ok(f) => f,
                Err(e) => panic!("An error occurred while trying to initialise the fanotify watcher: {}", e),
            };
            for cur_path in local_paths {
                fd.add_mountpoint(
                    FAN_CREATE | FAN_CLOSE_WRITE | FAN_MOVE_SELF | FAN_MODIFY,
                    (&cur_path).into(),
                ).unwrap();
            }
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
