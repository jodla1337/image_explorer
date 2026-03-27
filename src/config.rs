use crate::img::{ImageData, ImageFormat};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortBy {
    None,
    TimeCreated,
    Size,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    Extension(ImageFormat),
    StartsWith(String),
    Contains(String),
}

impl Filter {
    pub fn matches(&self, data: &ImageData) -> bool {
        match self {
            Filter::Extension(image_format) => data.format == image_format.clone(),
            Filter::StartsWith(phrase) => data.file_name.starts_with(phrase),
            Filter::Contains(phrase) => data.file_name.contains(phrase),
        }
    }
}

pub struct Config {
    pub filter_opts: FilterOptions,
    pub curr_index: usize,
    pub images: Vec<usize>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            filter_opts: Default::default(),
            curr_index: 0usize,
            images: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterOptions {
    pub sortedby: SortBy,
    pub filter: Option<Filter>,
}

impl Default for FilterOptions {
    fn default() -> Self {
        Self {
            sortedby: SortBy::None,
            filter: None,
        }
    }
}
