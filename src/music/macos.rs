use std::{
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

unsafe extern "C" {
    fn Cinna_BrowserAudioStatus() -> std::ffi::c_int;
}

pub fn is_playing() -> Result<bool, String> {
    if unsafe { Cinna_BrowserAudioStatus() } == 1 {
        return Ok(true);
    }
    let mut first_error = None;
    for player in ["Music", "Spotify"] {
        let running = Command::new("/usr/bin/pgrep")
            .args(["-x", player])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| e.to_string())?;
        if !running.success() {
            continue;
        }
        match query_player(player) {
            Ok(true) => return Ok(true),
            Ok(false) => {}
            Err(error) => {
                first_error.get_or_insert(error);
            }
        }
    }
    first_error.map_or(Ok(false), Err)
}

fn query_player(player: &str) -> Result<bool, String> {
    let script = format!(
        "if application \"{player}\" is running then\n tell application \"{player}\" to return player state as string\nend if"
    );
    let mut child = Command::new("/usr/bin/osascript")
        .args(["-e", &script])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = child.wait_with_output().map_err(|e| e.to_string())?;
                if !status.success() {
                    return Err(format!(
                        "Allow {player} access in Privacy & Security → Automation"
                    ));
                }
                return Ok(String::from_utf8_lossy(&output.stdout).trim() == "playing");
            }
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(30)),
            result => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(match result {
                    Err(error) => error.to_string(),
                    _ => format!("{player} did not respond; check Automation permission"),
                });
            }
        }
    }
}
