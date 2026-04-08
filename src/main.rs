pub mod config;
pub mod elements;
pub mod img;
pub mod resources;
pub mod subscriptions;
pub mod utils;

use iced::window::Settings;
use img::ImageData;
use std::{fmt::Debug, path::PathBuf, process::Command, sync::Arc};

pub use iced::{
    Color, Element, Length, Task, clipboard,
    widget::{
        Column, button, column,
        container::Container,
        image::{self, Allocation, Handle, Image, Viewer},
        row, text, text_input,
    },
};

use crate::{
    config::{Config, Filter, FilterVariations, Order, Query, SortBy},
    elements::mycontextmenu::ContextMenuOpt,
    img::LoadState,
    resources::{RESOURCES, Resources},
    utils::data_from_pagepos,
};

#[derive(Debug, Clone)]
pub enum Message {
    None,
    Init,
    AllocatedResource(String, Result<Allocation, image::Error>),
    AllocateOne(usize),
    AllocatedOne(usize, Result<Allocation, image::Error>),
    Page(u32),
    Allocate(u32 /* page */, u32 /* offset */, Query),
    ImageAllocated(
        Result<Allocation, image::Error>,
        usize, /* real index */
        u32,   /* page */
        u32,   /* offset */
        Query,
    ),
    Mode(Mode),
    PageInput(PageInput),
    SortBy(Option<SortBy>),
    SortOrder(Order),
    Filter(FilterVariations),
    ClearFilters,
    ClearQuery,
    ContextMenuOpt(ContextMenuOpt),
    HomeImage,
    HomeImageAllocated(usize, Result<Allocation, image::Error>),
}

#[derive(Debug, Clone)]
pub enum PageInput {
    OnInput(String),
    Submit,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum Mode {
    Home,
    Viewer(usize),
    Explorer,
}

pub struct AppState {
    mode: Mode,
    page: u32,
    images: Vec<ImageData>,
    bycreation: Vec<usize>,
    bymodification: Vec<usize>,
    bysize: Vec<usize>,
    config: Config,
    default_img: ImageData,
    home_image: Option<usize>,
    page_input: Option<String>,
    resources: Resources,
}

impl AppState {
    fn max_pages(&self) -> usize {
        let len = if self.config.query.filter.any() {
            self.config.images.len()
        } else {
            match self.config.query.sortedby {
                Some(SortBy::Size) => self.bysize.len(),
                Some(SortBy::TimeCreated) => self.bycreation.len(),
                Some(SortBy::TimeModified) => self.bymodification.len(),
                None => self.images.len(),
            }
        };

        let mut max_pages =
            len / PAGESIZE as usize + { if len % PAGESIZE as usize != 0 { 1 } else { 0 } };

        // here if there are no pages available still show the first page
        // - 1 because indices start from zero so if there is 1 page available
        // i want the maximum available page to be of index 0
        max_pages = { if max_pages == 0 { 1 } else { max_pages } } - 1;

        max_pages
    }
}

impl Default for AppState {
    fn default() -> Self {
        let (images, bysize, bycreation, bymodification) =
            img::find_images().expect("Error getting images from the disk");

        let state = Self {
            mode: Mode::Explorer,
            page: 0u32,
            images,
            bycreation,
            bymodification,
            bysize,
            config: Default::default(),
            default_img: Default::default(),
            home_image: None,
            page_input: None,
            resources: Resources::default(),
        };

        state
    }
}

const PAGESIZE: u32 = 12;

fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::None => Task::none(),
        Message::Init => {
            let dir = Resources::dir();

            let tasks = RESOURCES
                .iter()
                .map(|x| (Handle::from_path(&dir.join(x)), *x))
                .map(|(handle, key)| {
                    image::allocate(handle)
                        .map(|res| Message::AllocatedResource(key.to_string(), res))
                })
                .reduce(|x1, x2| x1.chain(x2))
                .unwrap_or(Task::none());

            Task::done(Message::Allocate(0, 0, Query::default())).chain(tasks)
        }
        Message::AllocatedResource(key, res) => {
            if let Ok(allocation) = res {
                let _ = state.resources.add(&key, allocation);
            }

            Task::none()
        }
        Message::AllocateOne(index) => {
            if let Some(data) = state.images.get(index) {
                if data.allocation.is_none() {
                    image::allocate(Handle::from_path(&data.path))
                        .map(move |res| Message::AllocatedOne(index, res))
                } else {
                    Task::none()
                }
            } else {
                Task::none()
            }
        }
        Message::AllocatedOne(index, res) => {
            if let Some(data) = state.images.get_mut(index) {
                data.allocation = Some({
                    match res {
                        Ok(allocation) => LoadState::Allocated(allocation),
                        Err(err) => LoadState::Error(err),
                    }
                })
            }

            Task::none()
        }

        Message::PageInput(page_input) => match page_input {
            PageInput::OnInput(pg_str) => {
                if pg_str.parse::<u32>().is_ok() && pg_str != "0" || pg_str.is_empty() {
                    state.page_input = Some(pg_str)
                }

                Task::none()
            }
            PageInput::Submit => {
                if let Some(pg_input) = &state.page_input {
                    if let Ok(pg) = pg_input.parse::<u32>() {
                        // converting user-friendly input to "index"
                        let pg = if pg == 0 { 1 } else { pg } - 1;

                        if state.config.query.filter.any() {
                            // this distinction is made because if you go to the page over limit in the
                            // case of a filter it will automatically go back to the last valid page regardless
                            Task::done(Message::Page(pg))
                        } else {
                            let max_pages = state.max_pages();
                            let page_to_go = if pg as usize > max_pages {
                                max_pages as u32
                            } else {
                                pg
                            };

                            Task::done(Message::Page(page_to_go))
                        }
                    } else {
                        state.page_input = None;
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            }
        },
        Message::Page(pg) => {
            if state.config.query.filter.any() {
                if state.config.finished_searching && pg as usize > state.max_pages() {
                    return Task::done(Message::Page(state.max_pages() as u32));
                }
            } else if pg as usize > state.max_pages() {
                return Task::done(Message::Page(state.max_pages() as u32));
            }

            state.page = pg;
            state.page_input = None;

            Task::done(Message::Allocate(pg, 0, state.config.query.clone()))
        }
        Message::Allocate(pg, offset, query) => {
            if pg != state.page || offset >= PAGESIZE || query != state.config.query {
                return Task::none();
            }

            let page_position: usize = (pg * PAGESIZE + offset) as usize;

            if query.filter.any() {
                if let Some(index) = state.config.images.get(page_position) {
                    let data = state
                        .images
                        .get(*index)
                        .expect("config.images should contain only valid indices");
                    let index_cpy = index.clone();
                    // let filter_opts_cpy = filter_opts.clone();
                    match data.allocation {
                        None => image::allocate(Handle::from_path(&data.path)).map(move |x| {
                            Message::ImageAllocated(x, index_cpy, pg, offset, query.clone())
                        }),
                        Some(_) => Task::done(Message::Allocate(pg, offset + 1, query)),
                    }
                } else {
                    while page_position + 1 > state.config.images.len() {
                        let data_opt = if let Some(sortby) = &query.sortedby {
                            let sorted_arr = match sortby {
                                SortBy::Size => &state.bysize,
                                SortBy::TimeCreated => &state.bycreation,
                                SortBy::TimeModified => &state.bymodification,
                            };

                            // dont want to go below 0 on usize
                            if sorted_arr.len() < state.config.curr_index + 1 {
                                state.config.finished_searching = true;
                                break;
                            }

                            let index = match query.sort_order {
                                Order::Ascending => state.config.curr_index,
                                Order::Descending => sorted_arr.len() - 1 - state.config.curr_index,
                            };

                            sorted_arr.get(index).map(|i| {
                                state
                                    .images
                                    .get(*i)
                                    .expect("sorted_arr should contain only valid indices")
                            })
                        } else {
                            state.images.get(state.config.curr_index)
                        };

                        state.config.curr_index += 1;

                        if let Some(data) = data_opt {
                            if query.filter.matches(data) {
                                state.config.images.push(data.index);
                            }
                        } else {
                            state.config.finished_searching = true;
                            break;
                        }
                    }

                    let data_opt = state.config.images.get(page_position).map(|i| {
                        state
                            .images
                            .get(*i)
                            .expect("config.images should contain only valid indices")
                    });

                    if let Some(data) = data_opt {
                        let index_cpy = data.index;
                        match data.allocation {
                            None => {
                                image::allocate(Handle::from_path(&data.path)).map(move |res| {
                                    Message::ImageAllocated(
                                        res,
                                        index_cpy,
                                        pg,
                                        offset,
                                        query.clone(),
                                    )
                                })
                            }
                            Some(_) => Task::done(Message::Allocate(pg, offset + 1, query)),
                        }
                    } else {
                        let max_pages = state.max_pages();
                        if state.page as usize > max_pages {
                            Task::done(Message::Page(max_pages as u32))
                        } else {
                            Task::none()
                        }
                    }
                }
            } else {
                let index_opt = if let Some(sortby) = &query.sortedby {
                    let sorted_arr = match sortby {
                        SortBy::Size => &state.bysize,
                        SortBy::TimeCreated => &state.bycreation,
                        SortBy::TimeModified => &state.bymodification,
                    };

                    // moved -1 to 1 on the opposite side because dont want to go below 0 on usize
                    if sorted_arr.len() < (pg * PAGESIZE + offset) as usize + 1 {
                        return Task::none();
                    }

                    let page_position = match query.sort_order {
                        Order::Ascending => (pg * PAGESIZE + offset) as usize,
                        Order::Descending => {
                            sorted_arr.len() - 1 - (pg * PAGESIZE + offset) as usize
                        }
                    };

                    sorted_arr.get(page_position).copied()
                } else {
                    Some((pg * PAGESIZE + offset) as usize)
                };

                if let Some(index) = index_opt {
                    if let Some(data) = state.images.get(index) {
                        let handle = Handle::from_path(&data.path);

                        let index_cpy = data.index;

                        match data.allocation {
                            Some(_) => Task::done(Message::Allocate(pg, offset + 1, query.clone())),
                            None => image::allocate(handle).map(move |x| {
                                Message::ImageAllocated(x, index_cpy, pg, offset, query.clone())
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
            if let Some(data) = state.images.get_mut(index) {
                data.allocation = Some({
                    match result {
                        Ok(allocation) => LoadState::Allocated(allocation),
                        Err(err) => {
                            println!("{:?}", err);
                            LoadState::Error(err)
                        }
                    }
                })
            }

            if state.page != pg && filter_opts != state.config.query {
                return Task::none();
            }

            Task::done(Message::Allocate(pg, offset + 1, filter_opts))
        }
        Message::Mode(mode) => {
            if let Mode::Viewer(page_position) = mode {
                if state.config.query.filter.any() && !state.config.finished_searching {
                    while page_position + 1 > state.config.images.len() {
                        let data_opt = if let Some(sortby) = &state.config.query.sortedby {
                            let sorted_arr = match sortby {
                                SortBy::Size => &state.bysize,
                                SortBy::TimeCreated => &state.bycreation,
                                SortBy::TimeModified => &state.bymodification,
                            };

                            // so i dont get usize below 0 as well
                            if state.config.curr_index + 1 > sorted_arr.len() {
                                state.config.finished_searching = true;
                                break;
                            }

                            let index = match &state.config.query.sort_order {
                                Order::Ascending => state.config.curr_index,
                                Order::Descending => sorted_arr.len() - 1 - state.config.curr_index,
                            };

                            sorted_arr.get(index).map(|i| {
                                state
                                    .images
                                    .get(*i)
                                    .expect("sorted_arr should always have valid indices")
                            })
                        } else {
                            state.images.get(state.config.curr_index)
                        };

                        state.config.curr_index += 1;

                        if let Some(data) = data_opt {
                            if state.config.query.filter.matches(data) {
                                state.config.images.push(data.index);
                            }
                        } else {
                            state.config.finished_searching = true;
                            break;
                        }
                    }

                    if state.config.images.get(page_position).is_some() {
                        state.mode = mode;
                    } else {
                        if state.config.images.len() > 0 {
                            state.mode = Mode::Viewer(state.config.images.len() - 1)
                        }
                    }
                } else if data_from_pagepos(state, page_position).is_some() {
                    state.mode = mode;
                };

                if let Mode::Viewer(curr_pagepos) = state.mode {
                    let page_togo = (curr_pagepos / PAGESIZE as usize) as u32;

                    if page_togo != state.page {
                        Task::done(Message::Page(page_togo))
                    } else {
                        Task::none()
                    }
                } else {
                    Task::none()
                }
            } else {
                state.mode = mode;
                Task::none()
            }
        }
        Message::SortBy(sortby) => {
            state.config.query.sortedby = sortby;

            state.config.reset();

            Task::done(Message::Page(0))
        }
        Message::SortOrder(order) => {
            state.config.query.sort_order = order;

            state.config.reset();

            Task::done(Message::Page(0))
        }
        Message::Filter(variation) => {
            state.config.query.filter.filter(variation);

            state.config.reset();

            Task::done(Message::Page(0))
        }
        Message::ClearFilters => {
            state.config.query.filter.clear();

            state.config.reset();

            Task::none()
        }
        Message::ClearQuery => {
            Task::done(Message::ClearFilters).chain(Task::done(Message::SortBy(None)))
        }
        Message::HomeImage => match state.mode {
            Mode::Home => {
                let index = rand::random_range(0..state.images.len());

                let data = &state.images[index];

                if let None = &data.allocation {
                    image::allocate(Handle::from_path(&data.path))
                        .map(move |a| Message::HomeImageAllocated(index, a))
                } else {
                    Task::none()
                }
            }
            _ => Task::none(),
        },
        Message::HomeImageAllocated(index, result) => {
            match result {
                Ok(allocation) => {
                    state.home_image = Some(index);
                    state
                        .images
                        .get_mut(index)
                        .expect("home_image is always valid")
                        .allocation = Some(LoadState::Allocated(allocation));
                }
                Err(err) => {
                    state
                        .images
                        .get_mut(index)
                        .expect("index is always valid")
                        .allocation = Some(LoadState::Error(err))
                }
            };

            Task::none()
        }
        Message::ContextMenuOpt(context_menu_opt) => match context_menu_opt {
            ContextMenuOpt::View(index) => Task::done(Message::Mode(Mode::Viewer(index))),
            ContextMenuOpt::CopyPath(path) => {
                clipboard::write(path.to_str().unwrap_or_default().to_string())
            }
            ContextMenuOpt::FileExplorer(path) => {
                if cfg!(target_os = "windows") {
                    let get_path_arg = |p: PathBuf| Some(p.as_os_str().to_str()?.to_string());

                    let path_arg = match get_path_arg(path) {
                        Some(p) => p,
                        None => {
                            println!("Error getting path");
                            return Task::none();
                        }
                    };

                    // println!("{}", path_arg);

                    if let Err(_) = Command::new("explorer")
                        .arg(format!("/select,{}", path_arg))
                        .spawn()
                    {
                        println!("Error launching the file explorer");
                    };
                };

                Task::none()
            }
        },
    }
}

fn view(state: &AppState) -> Element<'_, Message> {
    // println!("Images: {}", state.images.len());
    // println!("By Date: {}", state.bydate.len());
    // println!("By Size: {}", state.bysize.len());

    // println!("Config images: {}", state.config.images.len());

    match &state.mode {
        Mode::Home => elements::home(state),
        Mode::Viewer(index) => elements::viewer(state, *index),
        Mode::Explorer => elements::explorer(state),
    }
}

fn main() -> iced::Result {
    iced::application(boot, update, view)
        .window(Settings {
            ..Settings::default()
        })
        .subscription(subscriptions::slideshow)
        .subscription(subscriptions::keyboard_input)
        .theme(iced::Theme::CatppuccinMocha /* text color #1d2a3e */)
        .run()
}

fn boot() -> (AppState, Task<Message>) {
    (AppState::default(), Task::done(Message::Init))
}
