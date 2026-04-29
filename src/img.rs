use std::{
    collections::{BTreeMap, VecDeque},
    fmt::Display,
    fs::{self, DirEntry},
    path::PathBuf,
    sync::mpsc,
    time::{Duration, Instant, SystemTime},
};

use iced::widget::image::{self, Allocation};
use sysinfo::Disks;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ImageFormat {
    Jpg,
    Png,
    Bmp,
    Webp,
    Gif,
    Pnm,
    Tiff,
    Tga,
    Dds,
    Ico,
    Hdr,
    Exr,
    Avif,
    Qoi,
}

impl ImageFormat {
    fn from_string(extension: &String) -> Option<ImageFormat> {
        match extension.as_ref() {
            "jpg" | "jpeg" => Some(ImageFormat::Jpg),
            "png" => Some(ImageFormat::Png),
            "bmp" => Some(ImageFormat::Bmp),
            "webp" => Some(ImageFormat::Webp),
            "gif" => Some(ImageFormat::Gif),
            "pnm" => Some(ImageFormat::Pnm),
            "tif" => Some(ImageFormat::Tiff),
            "tga" => Some(ImageFormat::Tga),
            "dds" => Some(ImageFormat::Dds),
            "ico" => Some(ImageFormat::Ico),
            "hdr" => Some(ImageFormat::Hdr),
            "exr" => Some(ImageFormat::Exr),
            "avif" => Some(ImageFormat::Avif),
            "qoi" => Some(ImageFormat::Qoi),
            _ => None,
        }
    }

    #[allow(dead_code)]
    fn to_string(&self) -> String {
        match self {
            ImageFormat::Jpg => "jpg".to_string(),
            ImageFormat::Png => "png".to_string(),
            ImageFormat::Bmp => "bmp".to_string(),
            ImageFormat::Webp => "webp".to_string(),
            ImageFormat::Gif => "gif".to_string(),
            ImageFormat::Pnm => "pnm".to_string(),
            ImageFormat::Tiff => "tif".to_string(),
            ImageFormat::Tga => "tga".to_string(),
            ImageFormat::Dds => "dds".to_string(),
            ImageFormat::Ico => "ico".to_string(),
            ImageFormat::Hdr => "hdr".to_string(),
            ImageFormat::Exr => "exr".to_string(),
            ImageFormat::Avif => "avif".to_string(),
            ImageFormat::Qoi => "qoi".to_string(),
        }
    }

    fn add_to_counter(&self, counter: &mut Counter) {
        match self {
            ImageFormat::Jpg => counter.jpg += 1,
            ImageFormat::Png => counter.png += 1,
            ImageFormat::Bmp => counter.bmp += 1,
            ImageFormat::Webp => counter.webp += 1,
            ImageFormat::Gif => counter.gif += 1,
            ImageFormat::Pnm => counter.pnm += 1,
            ImageFormat::Tiff => counter.tif += 1,
            ImageFormat::Tga => counter.tga += 1,
            ImageFormat::Dds => counter.dds += 1,
            ImageFormat::Ico => counter.ico += 1,
            ImageFormat::Hdr => counter.hdr += 1,
            ImageFormat::Exr => counter.exr += 1,
            ImageFormat::Avif => counter.avif += 1,
            ImageFormat::Qoi => counter.qoi += 1,
        };
    }
}

#[derive(Default, Debug, Clone)]
pub struct Counter {
    jpg: u32,
    png: u32,
    bmp: u32,
    webp: u32,
    gif: u32,
    pnm: u32,
    tif: u32,
    tga: u32,
    dds: u32,
    ico: u32,
    hdr: u32,
    exr: u32,
    avif: u32,
    qoi: u32,
}

impl Display for Counter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "JPEG: {}\nPNG: {}\nBMP: {}\nWebP: {}\nGIF: {}\nPNM: {}\nTIFF: {}\nTGA: {}\nDDS: {}\nICO: {}\nHDR: {}\nOpenEXR: {}\nAVIF: {}\nQOI: {}",
            self.jpg,
            self.png,
            self.bmp,
            self.webp,
            self.gif,
            self.pnm,
            self.tif,
            self.tga,
            self.dds,
            self.ico,
            self.hdr,
            self.exr,
            self.avif,
            self.qoi
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

pub struct LoadingInfo {
    pub images_loaded: usize,
    pub started: Instant,
    pub time_loading: Duration,
    pub counter: Counter,
}

impl Default for LoadingInfo {
    fn default() -> Self {
        Self {
            started: Instant::now(),
            images_loaded: Default::default(),
            time_loading: Default::default(),
            counter: Default::default(),
        }
    }
}

pub async fn find_images(
    mut tx: sipper::Sender<usize>,
) -> (Vec<ImageData>, Vec<usize>, Vec<usize>, Vec<usize>, Counter) {
    let disks = Disks::new_with_refreshed_list();

    let mut directories: VecDeque<PathBuf> = disks
        .iter()
        .map(|disk| disk.mount_point().to_path_buf())
        .collect();

    let mut images = Vec::<ImageData>::new();

    let mut bysize = BTreeMap::<u64, Vec<usize>>::new();
    let mut bycreation = BTreeMap::<u64, Vec<usize>>::new();
    let mut bymodification = BTreeMap::<u64, Vec<usize>>::new();

    let mut counter = Counter::default();

    let mut index: usize = 0;

    let cores = num_cpus::get_physical();

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(cores)
        .build()
        .unwrap();

    // pool.scope(|s| s.spawn);

    while !directories.is_empty() {
        let mut pooled_dirs = Vec::new();

        for _ in 0..cores {
            match directories.pop_front() {
                Some(dir) => pooled_dirs.push(dir),
                None => break,
            }
        }

        let (dirs_tx, dirs_rx) = mpsc::channel::<Vec<PathBuf>>();
        let (imgs_tx, imgs_rx) = mpsc::channel::<Vec<ImageData>>();

        pool.scope(|s| {
            for dir in &pooled_dirs {
                s.spawn(|_| {
                    let (found_dirs, found_imgs) = search_directory(dir);

                    let _ = dirs_tx.send(found_dirs);
                    let _ = imgs_tx.send(found_imgs);
                });
            }
        });

        loop {
            match dirs_rx.try_recv() {
                Ok(dirs) => {
                    for dir in dirs {
                        directories.push_back(dir);
                    }
                }
                Err(_) => break,
            }
        }

        loop {
            match imgs_rx.try_recv() {
                Ok(imgs) => {
                    for mut img in imgs {
                        img.format.add_to_counter(&mut counter);
                        img.index = index;
                        index += 1;

                        if img.size != 0 {
                            bysize
                                .entry(img.size)
                                .and_modify(|v| v.push(img.index))
                                .or_insert(vec![img.index]);
                        }

                        if img.time_created != 0 {
                            bycreation
                                .entry(img.time_created)
                                .and_modify(|v| v.push(img.index))
                                .or_insert(vec![img.index]);
                        }

                        if img.time_modified != 0 {
                            bymodification
                                .entry(img.time_modified)
                                .and_modify(|v| v.push(img.index))
                                .or_insert(vec![img.index]);
                        }

                        images.push(img);
                    }
                }

                Err(_) => break,
            }
        }

        let _ = tx.send(images.len()).await;
    }

    let bycreation: Vec<usize> = bycreation.values().flatten().copied().collect();
    let bymodification: Vec<usize> = bymodification.values().flatten().copied().collect();
    let bysize: Vec<usize> = bysize.values().flatten().copied().collect();

    (images, bysize, bycreation, bymodification, counter)
}

fn search_directory(dir: &PathBuf) -> (Vec<PathBuf>, Vec<ImageData>) {
    let entries: Vec<DirEntry> = match fs::read_dir(dir) {
        Ok(x) => x.filter_map(|entry| entry.ok()).collect(),
        Err(_) => return (vec![], vec![]),
    };

    let mut directories = vec![];
    let mut images = vec![];

    for entry in &entries {
        let metadata = match entry.metadata() {
            Ok(x) => x,
            Err(_) => continue,
        };

        match metadata.is_dir() {
            true => {
                if !metadata.is_symlink() {
                    directories.push(entry.path());
                }
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
                        let (file_size, time_created, time_modified) = {
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
                        };

                        let imgdata = ImageData {
                            file_name: file_name.to_string(),
                            format: extension,
                            path: path.clone(),
                            allocation: None,
                            size: file_size,
                            time_created,
                            time_modified,
                            index: 0,
                        };

                        images.push(imgdata);
                    }
                }
            }
        };
    }

    (directories, images)
}
