use std::{
    collections::{BTreeMap, VecDeque},
    fmt::Display,
    fs::{self, DirEntry},
    path::PathBuf,
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

    while !directories.is_empty() {
        let dir = directories
            .pop_front()
            .expect("it should contain directories because of the loop condition");

        let entries: Vec<DirEntry> = match fs::read_dir(dir) {
            Ok(x) => x.filter_map(|entry| entry.ok()).collect(),
            Err(_) => continue,
        };

        for entry in &entries {
            let entry_file_type = if let Ok(x) = entry.file_type() {
                x
            } else {
                continue;
            };

            match entry_file_type.is_dir() {
                true => {
                    directories.push_back(entry.path());
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

                            let (file_size, time_created, time_modified) =
                                if let Ok(metadata) = entry.metadata() {
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
        let _ = tx.send(images.len()).await;
    }

    let bycreation: Vec<usize> = bycreation.values().flatten().copied().collect();
    let bymodification: Vec<usize> = bymodification.values().flatten().copied().collect();
    let bysize: Vec<usize> = bysize.values().flatten().copied().collect();

    (images, bysize, bycreation, bymodification, counter)
}
