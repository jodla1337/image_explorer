use crate::img::{ImageData, ImageFormat};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortBy {
    Size,
    TimeCreated,
    TimeModified,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Order {
    Ascending,
    Descending,
}

impl Order {
    pub fn switch(&self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FilterVariations {
    None,
    Extension(Option<ImageFormat>),
    StartsWith(Option<String>),
    Contains(Option<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Filter {
    pub extension: Option<ImageFormat>,
    pub startswith: Option<String>,
    pub contains: Option<String>,
}

impl Filter {
    pub fn matches(&self, data: &ImageData) -> bool {
        self.extension.is_none_or(|format| format == data.format)
            && self
                .startswith
                .as_ref()
                .is_none_or(|phrase| data.file_name.starts_with(phrase))
            && self
                .contains
                .as_ref()
                .is_none_or(|phrase| data.file_name.contains(phrase))
    }

    pub fn any(&self) -> bool {
        self.extension.is_some() || self.startswith.is_some() || self.contains.is_some()
    }

    pub fn filter(&mut self, variation: FilterVariations) {
        match variation {
            FilterVariations::None => *self = Filter::default(),
            FilterVariations::Extension(image_format) => self.extension = image_format,
            FilterVariations::StartsWith(phrase_opt) => self.startswith = phrase_opt,
            FilterVariations::Contains(phrase_opt) => self.contains = phrase_opt,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

pub struct Config {
    pub query: Query,
    pub curr_index: usize,
    pub images: Vec<usize>,
    pub finished_searching: bool,
}

impl Config {
    pub fn reset(&mut self) {
        self.curr_index = 0;
        self.images.clear();
        self.finished_searching = false;
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            query: Default::default(),
            curr_index: 0usize,
            images: Vec::new(),
            finished_searching: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub sortedby: Option<SortBy>,
    pub sort_order: Order,
    pub filter: Filter,
}

impl Default for Query {
    fn default() -> Self {
        Self {
            sortedby: None,
            sort_order: Order::Ascending,
            filter: Filter::default(),
        }
    }
}
