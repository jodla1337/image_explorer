use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    fmt::Display,
    fs::{self, DirEntry},
    io::Write,
    os::windows::fs::MetadataExt,
    path::PathBuf,
    str::FromStr,
    sync::mpsc,
    thread,
    time::{Duration, Instant, SystemTime},
};

use iced::widget::image::{self, Allocation};
use sysinfo::Disks;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageFormat {
    Jpg,
    Png,
    Bmp,
    Webp,
    Gif,
}

impl ImageFormat {
    fn from_string(extension: &String) -> Option<ImageFormat> {
        match extension.as_ref() {
            "jpg" | "jpeg" => Some(ImageFormat::Jpg),
            "png" => Some(ImageFormat::Png),
            "bmp" => Some(ImageFormat::Bmp),
            "webp" => Some(ImageFormat::Webp),
            "gif" => Some(ImageFormat::Gif),
            _ => None,
        }
    }

    fn to_string(&self) -> String {
        match self {
            ImageFormat::Jpg => "jpg".to_string(),
            ImageFormat::Png => "png".to_string(),
            ImageFormat::Bmp => "bmp".to_string(),
            ImageFormat::Webp => "webp".to_string(),
            ImageFormat::Gif => "gif".to_string(),
        }
    }

    fn add_to_counter(&self, counter: &mut Counter) {
        match self {
            ImageFormat::Jpg => counter.jpg += 1,
            ImageFormat::Png => counter.png += 1,
            ImageFormat::Bmp => counter.bmp += 1,
            ImageFormat::Webp => counter.webp += 1,
            ImageFormat::Gif => counter.gif += 1,
        };
    }
}

#[derive(Default)]
pub struct Counter {
    jpg: u32,
    png: u32,
    bmp: u32,
    webp: u32,
    gif: u32,
}

impl Display for Counter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "jpg: {}\npng: {}\nbmp: {}\nwebp: {}\ngif: {}",
            self.jpg, self.png, self.bmp, self.webp, self.gif
        )
    }
}

#[derive(Debug, Clone)]
pub struct ImageData {
    pub file_name: String,
    pub format: ImageFormat,
    pub path: PathBuf,
    pub allocation: Option<LoadState>,
    pub size: u64,
    pub time_created: u64,
    pub time_modified: u64,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub enum LoadState {
    Allocated(Allocation),
    Error(image::Error),
}

impl Default for ImageData {
    fn default() -> Self {
        let path = env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join(PathBuf::from_str("404.jpg").unwrap());

        Self {
            file_name: "404.jpg".to_string(),
            format: ImageFormat::Jpg,
            path: path.clone(),
            allocation: None,
            size: 0,
            time_created: 0,
            time_modified: 0,
            index: 0,
        }
    }
}

// struct SortedBy {
//     indices: Vec<usize>,
//     by: u64,
// }

// impl Ord for SortedBy {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.by.cmp(&other.by)
//     }
// }

// impl Eq for SortedBy {}

// impl PartialOrd for SortedBy {
//     fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//         self.by.partial_cmp(&other.by)
//     }
// }

// impl PartialEq for SortedBy {
//     fn eq(&self, other: &Self) -> bool {
//         self.by == other.by
//     }
// }

pub fn find_images<'a>()
-> Result<(Vec<ImageData>, Vec<usize>, Vec<usize>, Vec<usize>), Box<dyn std::error::Error>> {
    let now = Instant::now();

    let disks = Disks::new_with_refreshed_list();

    let mut directories: Vec<PathBuf> = disks
        .iter()
        .map(|disk| disk.mount_point().to_path_buf())
        .collect();

    let mut images = Vec::<ImageData>::new();

    let mut bysize = BTreeMap::<u64, Vec<usize>>::new();
    let mut bycreation = BTreeMap::<u64, Vec<usize>>::new();
    let mut bymodification = BTreeMap::<u64, Vec<usize>>::new();

    let mut counter = Counter::default();

    let mut i = 0;

    let mut index: usize = 0;
    let (finished_tx, finished_rx) = mpsc::channel::<bool>();
    let (progress_tx, progress_rx) = mpsc::channel::<usize>();

    let t = thread::spawn(move || {
        // print!("Loading");

        // 'outer: loop {
        //     for _ in 0..5 {
        //         print!(".");
        //         let _ = std::io::stdout().flush();
        //         thread::sleep(Duration::from_secs(1));
        //         if finished_rx.try_recv().is_ok_and(|x| x) {
        //             break 'outer;
        //         };
        //     }
        //     print!("\u{8}\u{8}\u{8}\u{8}\u{8}");
        //     print!("     ");
        //     print!("\u{8}\u{8}\u{8}\u{8}\u{8}");
        //     let _ = std::io::stdout().flush();
        // }

        loop {
            let progress = progress_rx.recv();
            print!("\r");
            let _ = std::io::stdout().flush();

            print!("Loading - ");

            if let Ok(length) = progress {
                print!("{} images", length);
            } else {
                break;
            }

            let _ = std::io::stdout().flush();
        }
    });

    while !directories.is_empty() {
        // let entries = directories
        //     .iter()
        //     .filter_map(|path| match fs::read_dir(path) {
        //         Ok(x) => Some(x),
        //         Err(_) => None,
        //     })
        //     .flatten()
        //     .filter_map(|x| x.ok())
        //     .collect::<Vec<DirEntry>>();

        let curr_directories = directories.clone();
        directories.clear();

        for dir in &curr_directories {
            let entries: Vec<DirEntry> = match fs::read_dir(dir) {
                Ok(x) => x.filter_map(|entry| entry.ok()).collect(),
                Err(_) => continue,
            };

            for entry in &entries {
                match entry.file_type()?.is_dir() {
                    true => {
                        directories.push(entry.path());
                    }
                    false => {
                        let path = entry.path();

                        let file_extension = path
                            .extension()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default()
                            .to_owned();
                        if let Some(file_name_osstring) = path.file_name() {
                            if let (Some(extension), Some(file_name)) = (
                                ImageFormat::from_string(&file_extension),
                                file_name_osstring.to_str(),
                            ) {
                                extension.add_to_counter(&mut counter);

                                let (file_size, time_created, time_modified) = if let Ok(metadata) =
                                    entry.metadata()
                                {
                                    let file_size = metadata.len();
                                    let time_created = if let Ok(created) = metadata.created() {
                                        if let Ok(since_epoch) =
                                            created.duration_since(SystemTime::UNIX_EPOCH)
                                        {
                                            since_epoch.as_secs()
                                        } else {
                                            0
                                        }
                                    } else {
                                        0
                                    };

                                    let time_modified = if let Ok(modified) = metadata.modified() {
                                        if let Ok(since_epoch) =
                                            modified.duration_since(SystemTime::UNIX_EPOCH)
                                        {
                                            since_epoch.as_secs()
                                        } else {
                                            0
                                        }
                                    } else {
                                        0
                                    };

                                    (file_size, time_created, time_modified)
                                } else {
                                    (0, 0, 0)
                                };

                                let imgdata = ImageData {
                                    file_name: file_name.to_string(),
                                    format: extension,
                                    path: path.clone(),
                                    allocation: None,
                                    size: file_size,
                                    time_created,
                                    time_modified,
                                    index,
                                };

                                if file_size != 0 {
                                    bysize
                                        .entry(file_size)
                                        .and_modify(|v| v.push(index))
                                        .or_insert(vec![index]);
                                }

                                if time_created != 0 {
                                    bycreation
                                        .entry(time_created)
                                        .and_modify(|v| v.push(index))
                                        .or_insert(vec![index]);
                                }

                                if time_modified != 0 {
                                    bymodification
                                        .entry(time_modified)
                                        .and_modify(|v| v.push(index))
                                        .or_insert(vec![index]);
                                }

                                images.push(imgdata);

                                index += 1;
                            }
                        }
                    }
                };
            }
            let _ = progress_tx.send(images.len());
        }
        i += 1;
        if i == 5 {
            break;
        };
    }

    drop(progress_tx);
    // finished_tx.send(true).unwrap();

    t.join().unwrap();

    let time_elapsed = now.elapsed().as_secs_f64();

    println!(
        "\rExecuted in {:.2}s. Found {} items.",
        time_elapsed,
        images.len()
    );

    println!("{counter}");

    let bycreation: Vec<usize> = bycreation.values().flatten().copied().collect();
    let bymodification: Vec<usize> = bymodification.values().flatten().copied().collect();
    let bysize: Vec<usize> = bysize.values().flatten().copied().collect();

    Ok((images, bysize, bycreation, bymodification))
}
