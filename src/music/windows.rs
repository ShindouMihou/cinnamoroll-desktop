use std::{
    thread,
    time::{Duration, Instant},
};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSessionManager,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus,
};
use windows_future::AsyncStatus;

pub fn is_playing() -> Result<bool, String> {
    let request = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
        .map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(2);
    while request.Status().map_err(|e| e.to_string())? == AsyncStatus::Started {
        if Instant::now() >= deadline {
            let _ = request.Cancel();
            return Err("Windows media sessions did not respond".into());
        }
        thread::sleep(Duration::from_millis(20));
    }
    let manager = request.GetResults().map_err(|e| e.to_string())?;
    let sessions = manager.GetSessions().map_err(|e| e.to_string())?;
    for session in sessions {
        let info = session.GetPlaybackInfo().map_err(|e| e.to_string())?;
        if info.PlaybackStatus().map_err(|e| e.to_string())?
            == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing
        {
            return Ok(true);
        }
    }
    Ok(false)
}
