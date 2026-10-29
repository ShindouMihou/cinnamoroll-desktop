use crate::{behavior::Activity, companion::preferences::Accessory};
use std::{
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) fn save(activity: Activity, accessory: Accessory) -> Result<PathBuf, String> {
    let folder = dirs::picture_dir()
        .or_else(dirs::document_dir)
        .ok_or("Cannot locate Pictures or Documents")?
        .join("Cinnamoroll");
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let path = folder.join(format!("cinnamoroll-{stamp}-{:04x}.png", fastrand::u16(..)));
    let mut image = super::render(activity, 0.5, 1.)?;
    super::accessories::apply(&mut image, accessory, activity, 4, 1.)?;
    let bytes = image.encode_png().map_err(|e| e.to_string())?;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;
    Ok(path)
}
