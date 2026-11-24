//! Environment variables under Local Paint's old name.
//!
//! The app was renamed from Local Paint to Local Raster, and its variables from `LOCALPAINT_*` to
//! `LOCALRASTER_*`. Scripts, agent configs and shell profiles that still set the old names keep
//! working: every reader of a `LOCALRASTER_*` variable that goes through this module also accepts the
//! `LOCALPAINT_*` twin, and the new name wins when both are set. (Build-time variables CI sets, such as
//! `LOCALRASTER_BUILD_SHA`, and the debug and benchmark switches that only tests read, have no
//! fallback.)

use std::env::VarError;
use std::ffi::OsString;

/// The prefix of the variables since the rename.
pub const NEW_PREFIX: &str = "LOCALRASTER_";
/// The prefix of the variables before the rename.
pub const OLD_PREFIX: &str = "LOCALPAINT_";

/// The old-name twin of `name`, when `name` is a `LOCALRASTER_*` variable.
pub fn old_name(name: &str) -> Option<String> {
    name.strip_prefix(NEW_PREFIX).map(|rest| format!("{OLD_PREFIX}{rest}"))
}

/// `name`, or its old-name twin when only that one is set (see the module docs).
pub fn var_os(name: &str) -> Option<OsString> {
    var_os_from(&|k| std::env::var_os(k), name)
}

/// [`var_os`] over the environment `get`.
pub fn var_os_from(get: &dyn Fn(&str) -> Option<OsString>, name: &str) -> Option<OsString> {
    get(name).or_else(|| old_name(name).and_then(|old| get(&old)))
}

/// [`std::env::var`] for `name`, falling back to its old-name twin when `name` isn't set.
pub fn var(name: &str) -> Result<String, VarError> {
    match std::env::var(name) {
        Err(VarError::NotPresent) => match old_name(name) {
            Some(old) => std::env::var(old),
            None => Err(VarError::NotPresent),
        },
        found => found,
    }
}

/// The variable to read for `name`: `name` itself, unless only its old-name twin is set. For clap's
/// `Arg::env`, which takes one name.
pub fn effective_name(name: &str) -> String {
    effective_name_from(&|k| std::env::var_os(k), name)
}

/// [`effective_name`] over the environment `get`.
pub fn effective_name_from(get: &dyn Fn(&str) -> Option<OsString>, name: &str) -> String {
    match old_name(name) {
        Some(old) if get(name).is_none() && get(&old).is_some() => old,
        _ => name.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> + use<> {
        let m: HashMap<String, OsString> = pairs.iter().map(|(k, v)| ((*k).to_owned(), OsString::from(v))).collect();
        move |k| m.get(k).cloned()
    }

    #[test]
    fn old_names_are_the_twins_of_the_new_ones() {
        assert_eq!(old_name("LOCALRASTER_CONTROL_PORT").as_deref(), Some("LOCALPAINT_CONTROL_PORT"));
        assert_eq!(old_name("HOME"), None);
        assert_eq!(old_name("LOCALPAINT_CONTROL_PORT"), None);
    }

    #[test]
    fn the_old_name_is_read_when_the_new_one_is_unset() {
        let get = env(&[("LOCALPAINT_CONFIG_DIR", "/old"), ("HOME", "/home")]);
        assert_eq!(var_os_from(&get, "LOCALRASTER_CONFIG_DIR"), Some(OsString::from("/old")));
        assert_eq!(effective_name_from(&get, "LOCALRASTER_CONFIG_DIR"), "LOCALPAINT_CONFIG_DIR");
        assert_eq!(var_os_from(&get, "HOME"), Some(OsString::from("/home")));
        assert_eq!(var_os_from(&get, "LOCALRASTER_NEW_INSTANCE"), None);
    }

    #[test]
    fn the_new_name_wins_when_both_are_set() {
        let get = env(&[("LOCALPAINT_CONFIG_DIR", "/old"), ("LOCALRASTER_CONFIG_DIR", "/new")]);
        assert_eq!(var_os_from(&get, "LOCALRASTER_CONFIG_DIR"), Some(OsString::from("/new")));
        assert_eq!(effective_name_from(&get, "LOCALRASTER_CONFIG_DIR"), "LOCALRASTER_CONFIG_DIR");
    }
}
