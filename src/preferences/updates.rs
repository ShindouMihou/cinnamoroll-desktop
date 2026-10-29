use super::{
    layout::{Action, LEFT, Page, RIGHT},
    paint::{INK, MUTED, SOFT},
};
use crate::{
    command::Command,
    settings::Settings,
    updates::{self, State},
};

impl Page<'_, '_> {
    pub(super) fn updates_page(&mut self, settings: &Settings) {
        let connected = updates::connected();
        self.painter.text("Launch at login", LEFT, 118., 14., INK);
        let startup = self.startup;
        self.note(
            144.,
            match startup {
                crate::native::startup::Status::Unavailable => {
                    "Requires the bundled app on macOS 13 or later."
                }
                crate::native::startup::Status::Approval => {
                    "Enable Cinnamoroll in your system startup settings."
                }
                _ => "A little company when you sign in.",
            },
        );
        self.button(
            [RIGHT - 64., 114., 64., 32.],
            if startup.registered() { "On" } else { "Off" },
            Action::Command(Command::LaunchAtLogin),
            startup.registered(),
            startup != crate::native::startup::Status::Unavailable,
        );
        if startup == crate::native::startup::Status::Approval {
            self.button(
                [LEFT, 169., 148., 26.],
                "Startup settings",
                Action::Command(Command::OpenLoginSettings),
                false,
                true,
            );
        }
        self.divider(211.);
        self.section(232., &format!("Updates · {}", env!("CARGO_PKG_VERSION")));
        self.painter
            .pill([LEFT, 264., RIGHT - LEFT, 125.], 12., SOFT, None);
        let title = match self.updates {
            State::Disconnected => "Updates aren’t connected yet".into(),
            State::AwaitingRelease => "The first release is getting ready".into(),
            State::Idle => "Ready when you are".into(),
            State::Checking => "Looking for something new…".into(),
            State::Current => "You’re all up to date".into(),
            State::Available(version) => format!("Version {version} is available"),
            State::Restarting => "Getting ready to restart…".into(),
            State::Downloading => "Downloading and checking its signature…".into(),
            State::Ready { version, .. } => format!("Version {version} is ready"),
            State::Error(_) => "We couldn’t finish that update".into(),
        };
        self.painter
            .fitted(&title, [LEFT + 18., 280., 468., 26.], 17., INK);
        let help = match self.updates {
            State::Disconnected => "Updates will be available when a release source is connected.",
            State::AwaitingRelease => {
                "Your update settings are saved. There’s no published release yet."
            }
            State::Ready { .. } => "Your preferences stay with you. Restart when you’re ready.",
            State::Downloading | State::Checking => {
                "You can keep Cinnamoroll company while this runs."
            }
            State::Error(error) => error,
            _ => "Only verified updates for your computer can be installed.",
        };
        self.painter
            .fitted(help, [LEFT + 18., 312., 468., 24.], 12., MUTED);
        self.button(
            [LEFT + 18., 343., 146., 30.],
            "Check for updates",
            Action::Command(Command::CheckUpdates),
            false,
            connected
                && !matches!(
                    self.updates,
                    State::Checking | State::Downloading | State::Restarting
                ),
        );
        if matches!(self.updates, State::Available(_)) {
            self.button(
                [RIGHT - 177., 343., 159., 30.],
                "Download update",
                Action::Command(Command::DownloadUpdate),
                true,
                true,
            );
        } else if matches!(self.updates, State::Ready { .. }) {
            self.button(
                [RIGHT - 177., 343., 159., 30.],
                "Install & restart",
                Action::Command(Command::InstallUpdate),
                true,
                true,
            );
        }
        self.painter
            .text("Automatic update checks", LEFT, 415., 14., INK);
        self.note(440., "Check at most once a day, quietly in the background.");
        self.button(
            [RIGHT - 64., 411., 64., 32.],
            if settings.automatic_update_checks {
                "On"
            } else {
                "Off"
            },
            Action::Command(Command::AutomaticUpdateChecks),
            settings.automatic_update_checks,
            true,
        );
        self.painter.text("Update while away", LEFT, 482., 14., INK);
        self.note(
            507.,
            "Install & relaunch after 1 hour without computer input.",
        );
        self.button(
            [RIGHT - 64., 478., 64., 32.],
            if settings.update_while_away {
                "On"
            } else {
                "Off"
            },
            Action::Command(Command::UpdateWhileAway),
            settings.update_while_away,
            true,
        );
    }
}
