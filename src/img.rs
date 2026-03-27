use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    error::Error,
    fs::{self, DirEntry, Metadata},
    os::windows::fs::MetadataExt,
    path::PathBuf,
    str::FromStr,
    time::Instant,
};

use iced::widget::{
    Image,
    image::{self, Allocation},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageFormat {
    Jpg,
    Png,
    Bmp,
    Webp,
}

impl ImageFormat {
    fn from_string(extension: &String) -> Option<ImageFormat> {
        match extension.as_ref() {
            "jpg" | "jpeg" => Some(ImageFormat::Jpg),
            "png" => Some(ImageFormat::Png),
            "bmp" => Some(ImageFormat::Bmp),
            "webp" => Some(ImageFormat::Webp),
            _ => None,
        }
    }

    fn to_string(&self) -> String {
        match self {
            ImageFormat::Jpg => "jpg".to_string(),
            ImageFormat::Png => "png".to_string(),
            ImageFormat::Bmp => "bmp".to_string(),
            ImageFormat::Webp => "webp".to_string(),
        }
    }

    fn add_to_counter(&self, counter: &mut Counter) {
        match self {
            ImageFormat::Jpg => counter.jpg += 1,
            ImageFormat::Png => counter.png += 1,
            ImageFormat::Bmp => counter.bmp += 1,
            ImageFormat::Webp => counter.webp += 1,
        };
    }
}

pub struct Counter {
    jpg: u32,
    png: u32,
    bmp: u32,
    webp: u32,
}

#[derive(Debug, Clone)]
pub struct ImageData {
    pub file_name: String,
    pub format: ImageFormat,
    pub path: PathBuf,
    pub allocation: Option<Allocation>,
    pub size: u64,
    pub time_created: u64,
    pub index: usize,
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
            index: 0,
        }
    }
}

struct SortedBy {
    index: usize,
    by: u64,
}

impl Ord for SortedBy {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.by.cmp(&other.by)
    }
}

impl Eq for SortedBy {}

impl PartialOrd for SortedBy {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.by.partial_cmp(&other.by)
    }
}

impl PartialEq for SortedBy {
    fn eq(&self, other: &Self) -> bool {
        self.by == other.by
    }
}

pub fn find_images<'a>()
-> Result<(Vec<ImageData>, Vec<usize>, Vec<usize>), Box<dyn std::error::Error>> {
    let now = Instant::now();

    let mut directories = vec![PathBuf::from(r"\")];

    let mut images = Vec::<ImageData>::new();
    let mut sortedbydate = BTreeSet::<SortedBy>::new();
    let mut sortedbysize = BTreeSet::<SortedBy>::new();

    let mut counter = Counter {
        jpg: 0,
        png: 0,
        bmp: 0,
        webp: 0,
    };

    let mut i = 0;

    let mut index: usize = 0;

    while !directories.is_empty() {
        let entries = directories
            .iter()
            .filter_map(|path| match fs::read_dir(path) {
                Ok(x) => Some(x),
                Err(_) => None,
            })
            .flatten()
            .filter(|entry| entry.is_ok())
            .map(|entry| entry.unwrap())
            .collect::<Vec<DirEntry>>();

        directories.clear();

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

                            let (file_size, time_created) = match entry.metadata() {
                                Ok(metadata) => (metadata.len(), metadata.creation_time()),
                                Err(_) => (0, 0),
                            };

                            let imgdata = ImageData {
                                file_name: file_name.to_string(),
                                format: extension,
                                path: path.clone(),
                                allocation: None,
                                size: file_size,
                                time_created,
                                index,
                            };

                            if file_size != 0 && time_created != 0 {
                                sortedbydate.insert(SortedBy {
                                    index,
                                    by: time_created,
                                });
                                sortedbysize.insert(SortedBy {
                                    index,
                                    by: file_size,
                                });
                            };

                            images.push(imgdata);

                            index += 1;
                        }
                    }
                }
            };
        }

        // i += 1;
        // if i == 7 {
        //     break;
        // };
    }

    let time_elapsed = now.elapsed().as_secs_f64();

    println!(
        "Executed in {:.2}s. Found {} items.",
        time_elapsed,
        images.len()
    );

    println!("jpg: {}", counter.jpg);
    println!("png: {}", counter.png);
    println!("webp: {}", counter.webp);
    println!("bmp: {}", counter.bmp);

    let sortedbydate: Vec<usize> = sortedbydate.iter().map(|x| x.index).collect();
    let sortedbysize: Vec<usize> = sortedbysize.iter().map(|x| x.index).collect();

    Ok((images, sortedbydate, sortedbysize))
}
