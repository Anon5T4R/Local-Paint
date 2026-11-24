//! Moves the folders Local Paint left behind to where Local Raster looks for them.
//!
//! The app was renamed from Local Paint, and its per-user folders with it (`%APPDATA%\LocalPaint`,
//! `~/.config/localpaint`, `~/.local/state/localpaint`, `<exe dir>/LocalPaintData` in portable mode).
//! On the first start under the new name, an old folder whose new twin doesn't exist yet is moved
//! there ([`migrate_dir`]), so preferences, brush presets, the UI layout and crash-recovery
//! autosaves carry over. It happens once, is logged, and never overwrites anything: if the new folder
//! exists the old one is left alone, and when the move has to be a copy (another volume, or a file
//! still open in an old process) the old folder stays as it was.

use std::path::{Path, PathBuf};

/// The portable data folder of Local Paint, beside the executable.
pub const OLD_PORTABLE_DATA_DIR: &str = "LocalPaintData";

/// How deep [`copy_tree`] follows folders: settings are a few levels deep at most.
const MAX_DEPTH: u32 = 16;

/// What [`migrate_dir`] did.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// There is no old folder, or the new one exists already: nothing to do.
    Nothing,
    /// The old folder was renamed to the new one.
    Moved,
    /// The contents were copied; the old folder is still there.
    Copied,
    /// Neither worked; the old folder is untouched and the next start tries again.
    Failed(String),
}

/// The folder Local Paint used in place of `new` (the last name swapped), if `new` carries the
/// app's folder name.
pub fn old_dir_of(new: &Path) -> Option<PathBuf> {
    let old = match new.file_name()?.to_str()? {
        "LocalRaster" => "LocalPaint",
        "localraster" => "localpaint",
        _ => return None,
    };
    Some(new.with_file_name(old))
}

/// [`migrate_dir`] from the folder Local Paint used in place of `new`, when there is one.
pub fn migrate_from_old_name(new: &Path) -> Outcome {
    old_dir_of(new).map_or(Outcome::Nothing, |old| migrate_dir(&old, new))
}

/// Move `old` to `new`, once: only when `old` is a folder and `new` doesn't exist. A rename on one
/// volume; otherwise the contents are copied to a staging folder beside `new` that is renamed into
/// place when complete, so `new` never exists half-copied. Nothing at `new` is ever replaced.
pub fn migrate_dir(old: &Path, new: &Path) -> Outcome {
    if !old.is_dir() || std::fs::symlink_metadata(new).is_ok() {
        return Outcome::Nothing;
    }
    if let Some(parent) = new.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        log::warn!("couldn't move {} to {}: {e}", old.display(), new.display());
        return Outcome::Failed(e.to_string());
    }
    if std::fs::rename(old, new).is_ok() {
        log::info!("moved the Local Paint folder {} to {}", old.display(), new.display());
        return Outcome::Moved;
    }
    let name = new.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let stage = new.with_file_name(format!(".{name}.migrating-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&stage);
    let copied = copy_tree(old, &stage, 0).and_then(|()| {
        if std::fs::symlink_metadata(new).is_ok() {
            Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "the new folder appeared meanwhile"))
        } else {
            std::fs::rename(&stage, new)
        }
    });
    match copied {
        Ok(()) => {
            log::info!("copied the Local Paint folder {} to {} (the old one stays)", old.display(), new.display());
            Outcome::Copied
        }
        Err(e) => {
            let _ = std::fs::remove_dir_all(&stage);
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                return Outcome::Nothing;
            }
            log::warn!("couldn't copy {} to {}: {e}; the old settings stay where they are", old.display(), new.display());
            Outcome::Failed(e.to_string())
        }
    }
}

/// Copy the folder `from` to the new folder `to`: plain files and folders only (links are skipped).
fn copy_tree(from: &Path, to: &Path, depth: u32) -> std::io::Result<()> {
    if depth > MAX_DEPTH {
        return Err(std::io::Error::other(format!("{} is nested too deep", from.display())));
    }
    std::fs::create_dir(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_dir() {
            copy_tree(&entry.path(), &target, depth.saturating_add(1))?;
        } else if kind.is_file() {
            let mut src = std::fs::File::open(entry.path())?;
            let mut dst = std::fs::OpenOptions::new().write(true).create_new(true).open(&target)?;
            std::io::copy(&mut src, &mut dst)?;
            dst.sync_all()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
