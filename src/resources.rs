use std::{env::current_exe, path::PathBuf};

pub fn dir() -> PathBuf {
    fn try_dir() -> Option<PathBuf> {
        let exe_dir = current_exe().expect("Can't get the current directory");
        Some(if cfg!(debug_assertions) {
            exe_dir.parent()?.parent()?.parent()?.join("resources")
        } else {
            exe_dir.parent()?.join("resources")
        })
    }

    try_dir().expect("Can't get the path to resources")
}
