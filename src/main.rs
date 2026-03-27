pub mod config;
pub mod elements;
pub mod img;

use img::ImageData;
use std::{fmt::Debug, ops::Deref, path::PathBuf};

pub use iced::{
    Color, Element, Length, Task,
    widget::{
        Column, button, column,
        container::Container,
        image::{self, Allocation, Handle, Image, Viewer},
        row, text, text_input,
    },
};

use crate::{
    config::{Config, Filter, FilterOptions, SortBy},
    img::ImageFormat,
};

#[derive(Debug, Clone)]
enum Message {
    Page(u32),
    Allocate(u32 /* page */, u32 /* offset */, FilterOptions),
    ImageAllocated(
        Result<Allocation, image::Error>,
        usize, /* real index */
        u32,   /* page */
        u32,   /* offset */
        FilterOptions,
    ),
    Mode(Mode),
    PageInput(String),
    SortBy(SortBy),
    Filter(Option<Filter>),
}

#[derive(Debug, Clone)]
enum Mode {
    Viewer(PathBuf),
    Explorer,
}

struct AppState {
    mode: Mode,
    page: u32,
    images: Vec<ImageData>,
    bydate: Vec<usize>,
    bysize: Vec<usize>,
    config: Config,
    default_img: ImageData,
}

impl Default for AppState {
    fn default() -> Self {
        let (images, bydate, bysize) = img::find_images().unwrap();

        Self {
            mode: Mode::Explorer,
            page: 0u32,
            images,
            bydate,
            bysize,
            config: Default::default(),
            default_img: Default::default(),
        }
    }
}

const PAGESIZE: u32 = 16;

fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::PageInput(pg_str) => {
            if let Ok(pg) = pg_str.parse::<u32>() {
                Task::done(Message::Page(pg))
            } else {
                Task::none()
            }
        }
        Message::Page(pg) => {
            state.page = pg;

            Task::done(Message::Allocate(pg, 0, state.config.filter_opts.clone()))
        }
        Message::Allocate(pg, offset, filter_opts) => {
            if pg != state.page || offset >= 16 || filter_opts != state.config.filter_opts {
                return Task::none();
            }

            let page_position: usize = (pg * PAGESIZE + offset) as usize;

            if let Some(filter) = &filter_opts.filter {
                if let Some(index) = state.config.images.get(page_position) {
                    if let Some(data) = state.images.get(*index) {
                        let index_cpy = index.clone();
                        // let filter_opts_cpy = filter_opts.clone();
                        match data.allocation {
                            None => image::allocate(Handle::from_path(&data.path)).map(move |x| {
                                Message::ImageAllocated(
                                    x,
                                    index_cpy,
                                    pg,
                                    offset,
                                    filter_opts.clone(),
                                )
                            }),
                            Some(_) => Task::done(Message::Allocate(pg, offset + 1, filter_opts)),
                        }
                    } else {
                        Task::none()
                    }
                } else {
                    while state.config.images.len() <= page_position {
                        let data_opt = match filter_opts.sortedby {
                            SortBy::None => state.images.get(state.config.curr_index),
                            SortBy::TimeCreated => {
                                state.bydate.get(state.config.curr_index).map(|i| {
                                    state
                                        .images
                                        .get(*i)
                                        .expect("should be there if its already in state.bydate")
                                })
                            }
                            SortBy::Size => state.bysize.get(state.config.curr_index).map(|i| {
                                state
                                    .images
                                    .get(*i)
                                    .expect("should be there if its already in state.bysize")
                            }),
                        };

                        state.config.curr_index += 1;

                        let data = match data_opt {
                            Some(x) => x,
                            None => return Task::none(),
                        };

                        if filter.matches(data) {
                            // println!("Format: {:?}", data.format);
                            state.config.images.push(data.index);
                        }
                    }

                    let data_opt = state.config.images.get(page_position).map(|i| {
                        state
                            .images
                            .get(*i)
                            .expect("should get cuz only valid indices are in images")
                    });

                    if let Some(data) = data_opt {
                        let index_cpy = data.index;
                        match data.allocation {
                            None => image::allocate(Handle::from_path(&data.path)).map(move |x| {
                                Message::ImageAllocated(
                                    x,
                                    index_cpy,
                                    pg,
                                    offset,
                                    filter_opts.clone(),
                                )
                            }),
                            Some(_) => Task::done(Message::Allocate(pg, offset + 1, filter_opts)),
                        }
                    } else {
                        Task::none()
                    }
                }
            } else {
                let index_opt: Option<usize> = match filter_opts.sortedby {
                    config::SortBy::None => Some((pg * PAGESIZE + offset) as usize),
                    config::SortBy::TimeCreated => {
                        state.bydate.get((pg * PAGESIZE + offset) as usize).copied()
                    }

                    config::SortBy::Size => {
                        state.bysize.get((pg * PAGESIZE + offset) as usize).copied()
                    }
                };

                if let Some(index) = index_opt {
                    if let Some(data) = state.images.get(index) {
                        let handle = Handle::from_path(&data.path);

                        let index_cpy = data.index;

                        match data.allocation {
                            Some(_) => Task::done(Message::Allocate(pg, offset + 1, filter_opts)),
                            None => image::allocate(handle).map(move |x| {
                                Message::ImageAllocated(
                                    x,
                                    index_cpy,
                                    pg,
                                    offset,
                                    filter_opts.clone(),
                                )
                            }),
                        }
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
        }
        Message::ImageAllocated(result, index, pg, offset, filter_opts) => {
            if let Ok(allocation) = result {
                let dataopt = state.images.get_mut(index);

                if let Some(data) = dataopt {
                    data.allocation = Some(allocation);
                }
                // }
            } else if let Err(e) = result {
                println!("{:?}", e);
            };

            if state.page != pg && filter_opts != state.config.filter_opts {
                return Task::none();
            }

            Task::done(Message::Allocate(pg, offset + 1, filter_opts))
        }
        Message::Mode(mode) => {
            state.mode = mode;
            Task::none()
        }
        Message::SortBy(sortby) => {
            state.config.filter_opts.sortedby = sortby;
            Task::done(Message::Filter(state.config.filter_opts.filter.clone()))
        }
        Message::Filter(filter_opt) => {
            state.config.filter_opts.filter = filter_opt;
            state.config.curr_index = 0;
            state.config.images.clear();

            Task::done(Message::Page(0))
        }
    }
}

fn view(state: &AppState) -> Element<'_, Message> {
    // println!("Images: {}", state.images.len());
    // println!("By Date: {}", state.bydate.len());
    // println!("By Size: {}", state.bysize.len());

    // println!("Config images: {}", state.config.images.len());

    match &state.mode {
        Mode::Viewer(path) => elements::viewer(state, path),
        Mode::Explorer => elements::explorer(state),
    }
}

fn main() -> iced::Result {
    iced::application(AppState::default, update, view)
        .theme(iced::Theme::CatppuccinMocha)
        .run()
}
