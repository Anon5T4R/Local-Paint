use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

fn temp(tag: &str) -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let d = std::env::temp_dir().join(format!("localraster-legacy-dirs-{tag}-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn old_folder(root: &Path) -> PathBuf {
    let old = root.join("LocalPaint");
    std::fs::create_dir_all(old.join("Presets")).unwrap();
    std::fs::write(old.join("preferences.json"), b"{\"theme\":\"dark\"}").unwrap();
    std::fs::write(old.join("Presets").join("brush.json"), b"brush").unwrap();
    old
}

#[test]
fn the_old_name_of_each_folder_convention() {
    assert_eq!(old_dir_of(Path::new("/a/LocalRaster")), Some(PathBuf::from("/a/LocalPaint")));
    assert_eq!(old_dir_of(Path::new("/a/.config/localraster")), Some(PathBuf::from("/a/.config/localpaint")));
    assert_eq!(old_dir_of(Path::new("/a/Other")), None);
}

#[test]
fn an_old_folder_is_moved_to_the_new_name_with_its_contents() {
    let root = temp("moved");
    let old = old_folder(&root);
    let new = root.join("LocalRaster");
    assert_eq!(migrate_from_old_name(&new), Outcome::Moved);
    assert!(!old.exists());
    assert_eq!(std::fs::read(new.join("preferences.json")).unwrap(), b"{\"theme\":\"dark\"}");
    assert_eq!(std::fs::read(new.join("Presets/brush.json")).unwrap(), b"brush");
    // One time: a second start finds the new folder and does nothing.
    assert_eq!(migrate_from_old_name(&new), Outcome::Nothing);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn an_existing_new_folder_is_never_overwritten() {
    let root = temp("exists");
    let old = old_folder(&root);
    let new = root.join("LocalRaster");
    std::fs::create_dir_all(&new).unwrap();
    std::fs::write(new.join("preferences.json"), b"new").unwrap();
    assert_eq!(migrate_dir(&old, &new), Outcome::Nothing);
    assert_eq!(std::fs::read(new.join("preferences.json")).unwrap(), b"new");
    assert!(old.join("preferences.json").is_file(), "the old folder stays");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn no_old_folder_is_not_an_error() {
    let root = temp("none");
    assert_eq!(migrate_from_old_name(&root.join("LocalRaster")), Outcome::Nothing);
    assert!(!root.join("LocalRaster").exists(), "nothing is created");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_copy_path_keeps_the_old_folder_and_never_writes_over_a_folder() {
    let root = temp("copy");
    let old = old_folder(&root);
    let new = root.join("LocalRaster");
    let stage = root.join("stage");
    copy_tree(&old, &stage, 0).unwrap();
    std::fs::rename(&stage, &new).unwrap();
    assert!(old.join("preferences.json").is_file());
    assert_eq!(std::fs::read(new.join("Presets/brush.json")).unwrap(), b"brush");
    assert!(copy_tree(&old, &new, 0).is_err(), "copying onto an existing folder is an error");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn the_portable_data_folder_moves_beside_the_exe() {
    let root = temp("portable");
    let old = root.join(OLD_PORTABLE_DATA_DIR);
    std::fs::create_dir_all(&old).unwrap();
    std::fs::write(old.join("preferences.json"), b"p").unwrap();
    let new = root.join("LocalRasterData");
    assert_eq!(migrate_dir(&old, &new), Outcome::Moved);
    assert_eq!(std::fs::read(new.join("preferences.json")).unwrap(), b"p");
    let _ = std::fs::remove_dir_all(&root);
}
