mod away;
#[cfg(feature = "updater")]
mod backend;

#[cfg(any(feature = "updater", test))]
mod config;
#[cfg(feature = "updater")]
mod install;
#[cfg(feature = "updater")]
mod validation;

#[cfg_attr(not(feature = "updater"), allow(dead_code))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum State {
    Disconnected,
    Idle,
    AwaitingRelease,
    Checking,
    Current,
    Available(String),
    Downloading,
    Restarting,
    Ready {
        version: String,
        archive: std::path::PathBuf,
    },
    Error(String),
}

pub(crate) struct Service {
    pub state: State,
    away: away::Away,
    receiver: Option<std::sync::mpsc::Receiver<State>>,
    last_automatic: Option<std::time::Instant>,
}

impl Default for Service {
    fn default() -> Self {
        Self {
            state: if connected() {
                State::Idle
            } else {
                State::Disconnected
            },
            away: away::Away::default(),
            receiver: None,
            last_automatic: None,
        }
    }
}

pub(crate) fn connected() -> bool {
    #[cfg(feature = "updater")]
    {
        static CONNECTED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *CONNECTED.get_or_init(|| config::Config::embedded().is_ok())
    }
    #[cfg(not(feature = "updater"))]
    {
        false
    }
}

impl Service {
    pub fn poll(&mut self, automatic: bool) {
        if let Some(receiver) = &self.receiver {
            match receiver.try_recv() {
                Ok(state) => {
                    self.state = state;
                    self.receiver = None;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.state =
                        State::Error("The update worker stopped. Please try again.".into());
                    self.receiver = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
        }
        if automatic
            && connected()
            && self.receiver.is_none()
            && self
                .last_automatic
                .is_none_or(|last| last.elapsed().as_secs() >= 86400)
        {
            self.last_automatic = Some(std::time::Instant::now());
            #[cfg(feature = "updater")]
            if backend::automatic_check_due() {
                self.check();
            }
        }
    }
    pub fn check(&mut self) {
        #[cfg(feature = "updater")]
        if connected() && self.receiver.is_none() {
            self.start(State::Checking, backend::check);
        }
    }
    pub fn download(&mut self) {
        #[cfg(feature = "updater")]
        if let State::Available(version) = &self.state {
            let version = version.clone();
            self.start(State::Downloading, move || backend::download(&version));
        }
    }
    #[cfg(feature = "updater")]
    fn start(
        &mut self,
        state: State,
        work: impl FnOnce() -> Result<State, String> + Send + 'static,
    ) {
        let (sender, receiver) = std::sync::mpsc::channel();
        match std::thread::Builder::new()
            .name("cinnamoroll-updates".into())
            .spawn(move || {
                let _ = sender.send(work().unwrap_or_else(State::Error));
            }) {
            Ok(_) => {
                self.state = state;
                self.receiver = Some(receiver);
            }
            Err(error) => self.state = State::Error(error.to_string()),
        }
    }
    pub fn while_away(&mut self, idle: Option<f64>, enabled: bool, safe: bool) -> bool {
        match self
            .away
            .tick(idle, enabled && connected(), safe, &self.state)
        {
            away::Action::Check => self.check(),
            away::Action::Download => self.download(),
            away::Action::Install => return self.install(true),
            away::Action::None => {}
        }
        false
    }
    pub fn install(&mut self, automatic: bool) -> bool {
        #[cfg(not(feature = "updater"))]
        let _ = automatic;
        #[cfg(feature = "updater")]
        if let State::Ready { version, archive } = &self.state {
            match install::launch(archive, version, automatic) {
                Ok(launched) => {
                    if launched {
                        self.state = State::Restarting;
                    }
                    return launched;
                }
                Err(error) => self.state = State::Error(error),
            }
        }
        false
    }
}

pub(crate) fn apply(arguments: &[String]) -> Option<Result<(), String>> {
    let index = arguments
        .iter()
        .position(|argument| argument == "--apply-signed-update")?;
    #[cfg(feature = "updater")]
    {
        Some(install::apply(&arguments[index + 1..]))
    }
    #[cfg(not(feature = "updater"))]
    {
        let _ = index;
        Some(Err("This build has no updater transport.".into()))
    }
}

pub(crate) fn verify_release(archive: &std::path::Path) -> Result<(), String> {
    #[cfg(feature = "updater")]
    {
        validation::verify_release(archive)?;
        println!(
            "Signed release verified: Cinnamoroll {}",
            env!("CARGO_PKG_VERSION")
        );
        Ok(())
    }
    #[cfg(not(feature = "updater"))]
    {
        let _ = archive;
        Err("Build with the updater feature to verify releases.".into())
    }
}
