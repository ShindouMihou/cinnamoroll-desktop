#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
use macos::Provider;
#[cfg(target_os = "windows")]
use windows::Provider;

use crate::command::Signal;
use std::{
    cell::Cell,
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

pub struct Monitor {
    control: Sender<bool>,
    worker: Option<JoinHandle<()>>,
    enabled: Cell<bool>,
}

impl Monitor {
    pub fn new(signals: Sender<Signal>) -> Self {
        let (control, receiver) = mpsc::channel();
        let worker = thread::spawn(move || {
            let mut provider: Option<Provider> = None;
            let mut enabled = false;
            let mut previous: Option<(f32, f32)> = None;
            let mut last_status = String::new();
            loop {
                let message = if enabled {
                    receiver.recv_timeout(Duration::from_millis(500))
                } else {
                    receiver
                        .recv()
                        .map_err(|_| mpsc::RecvTimeoutError::Disconnected)
                };
                match message {
                    Ok(value) => {
                        enabled = value;
                        previous = None;
                        last_status.clear();
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
                if !enabled {
                    provider = None;
                    continue;
                }
                let result = if let Some(provider) = provider.as_mut() {
                    provider.position()
                } else {
                    match Provider::new() {
                        Ok(value) => {
                            provider = Some(value);
                            continue;
                        }
                        Err(error) => Err(error),
                    }
                };
                let status = match result {
                    Ok(point) => {
                        if let (Some(current), Some(old)) = (point, previous)
                            && ((current.0 - old.0).hypot(current.1 - old.1) >= 2.
                                || crate::system::idle_seconds().is_some_and(|idle| idle < 1.))
                        {
                            let _ = signals.send(Signal::CaretMoved(current));
                        }
                        previous = point;
                        if point.is_some() {
                            "Typing cursor found · location only".into()
                        } else {
                            "Waiting for a supported text cursor".into()
                        }
                    }
                    Err(error) => {
                        previous = None;
                        error
                    }
                };
                if status != last_status {
                    let _ = signals.send(Signal::CaretStatus(status.clone()));
                    last_status = status;
                }
            }
        });
        Self {
            control,
            worker: Some(worker),
            enabled: Cell::new(false),
        }
    }
    pub fn enable(&self, enabled: bool) {
        if self.enabled.replace(enabled) != enabled {
            let _ = self.control.send(enabled);
        }
    }
}

impl Drop for Monitor {
    fn drop(&mut self) {
        let (replacement, _) = mpsc::channel();
        drop(std::mem::replace(&mut self.control, replacement));
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
struct Provider;
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
impl Provider {
    fn new() -> Result<Self, String> {
        Err("Typing cursor following supports macOS and Windows".into())
    }
    fn position(&mut self) -> Result<Option<(f32, f32)>, String> {
        Ok(None)
    }
}
