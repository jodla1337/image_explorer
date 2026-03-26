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

use crate::config::{Config, Filter, FilterOptions, SortBy};

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

            if let Some(filter) = filter_opts.filter {
                if let Some(index) = state.config.images.get((pg * PAGESIZE + offset) as usize) {
                    if let Some(data) = state.images.get(*index) {
                        let index_cpy = index.clone();
                        match &data.allocation {
                            None => image::allocate(Handle::from_path(&data.path)).map(move |x| {
                                Message::ImageAllocated(x, index_cpy, pg, offset, filter_opts)
                            }),
                            Some(allocation) => {
                                Task::done(Message::Allocate(pg, offset + 1, filter_opts))
                            }
                        }
                    } else {
                        Task::none()
                    }
                } else {
                    let images_len = state.images.len();
                    while state.config.curr_index < images_len {
                        let data = state.images.get(state.config.curr_index).expect("should always get a value because the index is lower than images.len()");

                        if filter.matches(data) {
                            state.config.images.push(state.config.curr_index);
                            state.config.curr_index += 1;
                            let index_cpy = data.index;
                            return image::allocate(Handle::from_path(&data.path)).map(move |x| {
                                Message::ImageAllocated(x, index_cpy, pg, offset, filter_opts)
                            });
                        }
                        state.config.curr_index += 1;
                    }
                    Task::none()
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
                                Message::ImageAllocated(x, index_cpy, pg, offset, filter_opts)
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
                // let index_opt: Option<usize> = match filter_opts.sortedby {
                //     config::SortBy::None => Some((pg * PAGESIZE + offset) as usize),
                //     config::SortBy::TimeCreated => {
                //         state.bydate.get((pg * PAGESIZE + offset) as usize).copied()
                //     }
                //     config::SortBy::Size => {
                //         state.bysize.get((pg * PAGESIZE + offset) as usize).copied()
                //     }
                // };

                // if let Some(index) = index_opt {
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
            Task::done(Message::Filter(state.config.filter_opts.filter))
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
    match &state.mode {
        Mode::Viewer(path) => row![
            Container::new(
                Viewer::new(image::Handle::from_path(path))
                    .width(Length::Fill)
                    .height(Length::Fill)
            )
            .width(Length::FillPortion(9)),
            Container::new(
                button("Back")
                    .width(Length::FillPortion(1))
                    .height(150)
                    .on_press(Message::Mode(Mode::Explorer))
            )
            .width(Length::FillPortion(1))
            .center_y(200)
        ]
        .padding(10)
        .into(),
        Mode::Explorer => row![
            elements::images_column(state),
            column![
                button("Next")
                    .width(150)
                    .height(150)
                    .on_press(Message::Page(state.page + 1)),
                button("Previous")
                    .width(150)
                    .height(150)
                    .on_press(Message::Page(if state.page == 0 {
                        0
                    } else {
                        state.page - 1
                    })),
                text_input("Page number", &state.page.to_string()).on_input(Message::PageInput),
                text("sort by: "),
                row![
                    button("By nothing")
                        .width(150)
                        .height(150)
                        .on_press(Message::SortBy(SortBy::None)),
                    button("By creation time")
                        .width(150)
                        .height(150)
                        .on_press(Message::SortBy(SortBy::TimeCreated)),
                    button("By file size")
                        .width(150)
                        .height(150)
                        .on_press(Message::SortBy(SortBy::Size))
                ]
            ]
        ]
        .padding(10)
        .into(),
    }
}

fn main() -> iced::Result {
    iced::application(AppState::default, update, view)
        .theme(iced::Theme::CatppuccinMocha)
        .run()
}
