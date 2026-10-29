use super::State;

#[derive(Default)]
pub(super) struct Away {
    checked: bool,
    installed: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Action {
    None,
    Check,
    Download,
    Install,
}

pub(super) fn long_idle(idle: Option<f64>) -> bool {
    idle.is_some_and(|seconds| seconds.is_finite() && seconds >= 3600.)
}

impl Away {
    pub fn tick(&mut self, idle: Option<f64>, enabled: bool, safe: bool, state: &State) -> Action {
        if !enabled || !long_idle(idle) {
            *self = Self::default();
            return Action::None;
        }
        if !safe {
            return Action::None;
        }
        match state {
            State::Idle | State::Current | State::AwaitingRelease | State::Error(_)
                if !self.checked =>
            {
                self.checked = true;
                Action::Check
            }
            State::Available(_) => Action::Download,
            State::Ready { .. } if !self.installed => {
                self.installed = true;
                Action::Install
            }
            _ => Action::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn away_update_waits_for_an_hour_and_checks_only_once_per_visit() {
        let mut away = Away::default();
        for idle in [None, Some(f64::NAN), Some(3599.)] {
            assert_eq!(away.tick(idle, true, true, &State::Idle), Action::None);
        }
        assert_eq!(
            away.tick(Some(3600.), true, false, &State::Idle),
            Action::None
        );
        assert_eq!(
            away.tick(Some(3601.), true, true, &State::Idle),
            Action::Check
        );
        assert_eq!(
            away.tick(Some(3602.), true, true, &State::Current),
            Action::None
        );
        assert_eq!(
            away.tick(Some(3603.), true, true, &State::Available("1.2.3".into())),
            Action::Download
        );
        let ready = State::Ready {
            version: "1.2.3".into(),
            archive: "update.zip".into(),
        };
        assert_eq!(away.tick(Some(3604.), true, true, &ready), Action::Install);
        assert_eq!(away.tick(Some(3605.), true, true, &ready), Action::None);
        assert_eq!(away.tick(Some(0.), true, true, &ready), Action::None);
        assert_eq!(away.tick(Some(4000.), true, true, &ready), Action::Install);
    }
}
