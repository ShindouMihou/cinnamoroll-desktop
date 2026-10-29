#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use crate::command::Signal;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

pub struct Monitor {
    enabled: AtomicBool,
    control: Sender<bool>,
    worker: Option<JoinHandle<()>>,
}

impl Monitor {
    pub fn new(signals: Sender<Signal>) -> Self {
        let (control, receiver) = mpsc::channel();
        let worker = thread::spawn(move || {
            let mut enabled = false;
            let mut playing = false;
            let mut last_error: Option<String> = None;
            loop {
                let message = if enabled {
                    receiver.recv_timeout(Duration::from_secs(4))
                } else {
                    receiver
                        .recv()
                        .map_err(|_| mpsc::RecvTimeoutError::Disconnected)
                };
                match message {
                    Ok(value) => {
                        enabled = value;
                        playing = false;
                        last_error = None;
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
                if !enabled {
                    continue;
                }
                match is_playing() {
                    Ok(current) => {
                        if current && !playing {
                            let _ = signals.send(Signal::MusicStarted);
                        }
                        if last_error.take().is_some() {
                            let _ =
                                signals.send(Signal::MusicStatus("Music reactions ready".into()));
                        }
                        playing = current;
                    }
                    Err(error) => {
                        if last_error.as_ref() != Some(&error) {
                            let _ = signals.send(Signal::MusicStatus(error.clone()));
                            last_error = Some(error);
                        }
                        playing = false;
                    }
                }
            }
        });
        Self {
            enabled: AtomicBool::new(false),
            control,
            worker: Some(worker),
        }
    }

    pub fn enable(&self, enabled: bool) {
        if self.enabled.swap(enabled, Ordering::Relaxed) != enabled {
            let _ = self.control.send(enabled);
        }
    }
}

impl Drop for Monitor {
    fn drop(&mut self) {
        let (replacement, _) = mpsc::channel();
        let original = std::mem::replace(&mut self.control, replacement);
        drop(original);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn is_playing() -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        macos::is_playing()
    }
    #[cfg(target_os = "windows")]
    {
        windows::is_playing()
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Err("Music reactions are available on macOS and Windows".into())
    }
}
