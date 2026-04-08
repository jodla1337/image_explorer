use std::{collections::HashMap, env::current_exe, fs::File, path::PathBuf};

use iced::widget::image::Allocation;

pub const RESOURCES: [&str; 9] = [
    "arrow_left.png",
    "arrow_right.png",
    "arrow_up.png",
    "arrow_down.png",
    "clear.png",
    "filter.png",
    "sort.png",
    "trash.png",
    "back.png",
];

#[derive(Default, Debug)]
pub struct Resources {
    pub arrow_left: Option<Allocation>,
    pub arrow_right: Option<Allocation>,
    pub arrow_up: Option<Allocation>,
    pub arrow_down: Option<Allocation>,
    pub clear: Option<Allocation>,
    pub filter: Option<Allocation>,
    pub sort: Option<Allocation>,
    pub trash: Option<Allocation>,
    pub back: Option<Allocation>,
}

// instance methods
impl Resources {
    pub fn add(&mut self, key: &str, allocation: Allocation) -> bool {
        let a = Some(allocation);
        match key {
            "arrow_left.png" => self.arrow_left = a,
            "arrow_right.png" => self.arrow_right = a,
            "arrow_up.png" => self.arrow_up = a,
            "arrow_down.png" => self.arrow_down = a,
            "filter.png" => self.filter = a,
            "sort.png" => self.sort = a,
            "clear.png" => self.clear = a,
            "trash.png" => self.trash = a,
            "back.png" => self.back = a,
            _ => return false,
        };
        true
    }
}

// static methods
impl Resources {
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
}
