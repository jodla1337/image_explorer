pub mod bar;

use std::{path::PathBuf, time::SystemTime};

use chrono::{DateTime, NaiveDate, NaiveDateTime};
use iced::{
    Color, Element, Length, Rectangle, Theme,
    widget::{
        Column, Container, Image, Text, button, column, container,
        image::{self, Allocation, Viewer},
        row, space, text, text_input,
    },
};
use iced_aw::{ContextMenu, Menu, menu_bar};

use crate::{
    config::{Filter, FilterVariations, Order, SortBy},
    elements::bar::{footer, mymenu},
    img::LoadState,
};

use super::{AppState, Message, Mode, PAGESIZE, img::ImageData};

pub fn home(state: &AppState) -> Element<'_, Message> {
    fn get_allocation(state: &AppState) -> Option<&LoadState> {
        state.images.get(state.home_image?)?.allocation.as_ref()
    }

    let container = match get_allocation(state) {
        Some(LoadState::Allocated(allocation)) => Container::new(
            Image::new(allocation.handle())
                .width(Length::Fill)
                .height(Length::Fill),
        ),
        _ => Container::new(text(""))
            .style(|_| container::Style::default().background(Color::from_rgb(1.0, 0.0, 0.0))),
    };

    container.width(Length::Fill).height(Length::Fill).into()
}

pub fn viewer(state: &AppState, index: usize) -> Element<'_, Message> {
    let data_opt: Option<&ImageData> = if state.config.query.filter.any() {
        state.config.images.get(index).map(|i| {
            state
                .images
                .get(*i)
                .expect("config.images contains only valid indices")
        })
    } else {
        if let Some(sortby) = &state.config.query.sortedby {
            let sorted_arr = match sortby {
                SortBy::Size => &state.bysize,
                SortBy::TimeCreated => &state.bycreation,
                SortBy::TimeModified => &state.bymodification,
            };

            if index < sorted_arr.len() {
                let position = match state.config.query.sort_order {
                    Order::Ascending => index,
                    Order::Descending => sorted_arr.len() - 1 - index,
                };

                sorted_arr.get(position).map(|i| {
                    state
                        .images
                        .get(*i)
                        .expect("sorted_arr contains only valid indices")
                })
            } else {
                None
            }
        } else {
            state.images.get(index)
        }
    };

    let content: Element<'_, Message> = if let Some(data) = data_opt {
        let ctxmenu = ContextMenu::new(
            container(match &data.allocation {
                Some(LoadState::Allocated(allocation)) => {
                    <Viewer<image::Handle> as Into<Element<'_, Message>>>::into(
                        Viewer::new(allocation.handle())
                            .width(Length::Fill)
                            .height(Length::Fill),
                    )
                }
                Some(LoadState::Error(err)) => error_message(err),
                None => space().width(Length::Fill).height(Length::Fill).into(),
            })
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10),
            move || mycontextmenu::overlay(data.path.clone(), index, false),
        );

        row![ctxmenu, image_info(data)].into()
    } else {
        container(text("Not available").center())
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    };

    column![
        mymenu::header_viewer(state, index),
        content,
        footer::footer_viewer(state, index)
    ]
    .into()
}

pub fn explorer(state: &AppState) -> Element<'_, Message> {
    column![
        mymenu::header_explorer(state),
        images_column(state),
        footer::footer_explorer(state)
    ]
    .width(Length::Fill)
    .into()
}

pub fn image_info(data: &ImageData) -> Element<'_, Message> {
    fn subtext<'a>(content: impl text::IntoFragment<'a>) -> Text<'a> {
        text(content)
    }

    let content = column![
        subtext(format!("File name: {}", data.file_name)),
        subtext(format!(
            "Path: {}",
            data.path.parent().unwrap_or(&data.path).to_string_lossy()
        )),
        subtext(format!(
            "Size: {:#}",
            pretty_bytes::converter::convert(data.size as f64)
        )),
        subtext(format!(
            "Time created: {}",
            DateTime::from_timestamp_secs(data.time_created as i64).unwrap_or_default()
        )),
        subtext(format!(
            "Time modified: {}",
            DateTime::from_timestamp_secs(data.time_modified as i64).unwrap_or_default()
        )),
    ]
    .padding(10)
    .spacing(10);

    container(content)
        .style(|theme| {
            container::primary(theme)
                .background(theme.extended_palette().primary.base.color)
                .color(theme.extended_palette().primary.base.text)
        })
        .width(250)
        .height(Length::Fill)
        .into()
}

pub fn images_column(state: &AppState) -> Column<'_, Message> {
    const PADDING: u16 = 10;
    const SPACING: u32 = 10;

    let mut col: Column<'_, Message> = column![];

    let mut i: usize = 0;

    let page_position = (state.page * PAGESIZE) as usize;

    while i < PAGESIZE as usize {
        let mut row_ = row![];
        for _ in 0..4 {
            let data = if state.config.query.filter.any() {
                state
                    .config
                    .images
                    .get(page_position + i)
                    .map(|index| {
                        state
                            .images
                            .get(*index)
                            .expect("should be there if already in config.images")
                    })
                    .unwrap_or(&state.default_img)
            } else {
                let index_opt_fn = || {
                    if let Some(sortby) = &state.config.query.sortedby {
                        let sorted_arr = match sortby {
                            SortBy::Size => &state.bysize,
                            SortBy::TimeCreated => &state.bycreation,
                            SortBy::TimeModified => &state.bymodification,
                        };

                        if sorted_arr.len() <= page_position + i {
                            return None;
                        }

                        let index = match &state.config.query.sort_order {
                            Order::Ascending => page_position + i,
                            Order::Descending => sorted_arr.len() - 1 - page_position - i,
                        };

                        sorted_arr.get(index).copied()
                    } else {
                        Some(page_position + i)
                    }
                };

                let index_opt = index_opt_fn();

                if let Some(index) = index_opt {
                    state.images.get(index).unwrap_or(&state.default_img)
                } else {
                    &state.default_img
                }
            };

            // let handle = data.allocation.as_ref().map(|x| x.handle());

            let content: Element<'_, Message> = match &data.allocation {
                Some(LoadState::Allocated(allocation)) => Image::new(allocation.handle())
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into(),
                Some(LoadState::Error(err)) => error_message(err),
                None => space().width(Length::Fill).height(Length::Fill).into(),
            };

            let mut frame = button(content)
                .style(|theme, _| {
                    button::Style::default().with_background(theme.palette().background)
                })
                .width(Length::FillPortion(1))
                .height(Length::FillPortion(1))
                .padding(0);

            if data.allocation.is_some() {
                frame = frame.on_press(Message::Mode(Mode::Viewer(page_position + i)))
            }

            // let mut frame: Element<'_, Message> = match &data.allocation {
            //     Some(allocation) => button(
            //         Container::new(Image::new(allocation.handle()).width(200).height(200))
            //             .width(200)
            //             .height(200),
            //     )

            //     .into(),
            //     None => Container::new(
            //         text("Error")
            //             .color(Color::from_rgb(1.0, 0.02, 0.02))
            //             .center(),
            //     )
            //     .width(200)
            //     .height(200)
            //     .into(),
            // };

            let path = data.path.clone();

            let child: Element<'_, Message> = if data.allocation.is_some() {
                ContextMenu::new(frame, move || {
                    mycontextmenu::overlay(path.clone(), data.index, true)
                })
                .into()
            } else {
                frame.into()
            };
            row_ = row_.push(child);

            i += 1;
        }
        col = col.push(row_.spacing(SPACING));
    }

    col.padding(PADDING).spacing(SPACING)
}

pub fn error_message(_: &image::Error) -> Element<'static, Message> {
    container(
        text("The image could not be decoded")
            .width(Length::Fill)
            .height(Length::Fill)
            .color(Color::from_rgb(1.0, 0.1, 0.1))
            .center(),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

const BORDER_RADIUS: u32 = 10;

pub mod mycontextmenu {
    use iced::{Border, Color, border::Radius, widget::container};

    use crate::Mode;

    use super::{
        BORDER_RADIUS, Container, Element, Length, Message, PathBuf, Theme, button, column,
    };

    pub fn overlay(path: PathBuf, index: usize, isexplorer: bool) -> Element<'static, Message> {
        let mut col = column![];

        if isexplorer {
            col = col.push(
                button("View")
                    .style(|theme, status| {
                        let mut style = button_style(theme, status);
                        style.border = Border::default().rounded(Radius::new(0).top(BORDER_RADIUS));
                        style
                    })
                    .width(Length::Fill)
                    .on_press(Message::ContextMenuOpt(ContextMenuOpt::View(index))),
            );
        };

        col = col.extend([
            button("Copy path")
                .style(button_style)
                .width(Length::Fill)
                .on_press(Message::ContextMenuOpt(ContextMenuOpt::CopyPath(
                    path.clone(),
                )))
                .into(),
            button("Open in file explorer")
                .style(|theme, status| {
                    let mut style = button_style(theme, status);
                    style.border = Border::default().rounded(Radius::new(0).bottom(BORDER_RADIUS));
                    style
                })
                .width(Length::Fill)
                .on_press(Message::ContextMenuOpt(ContextMenuOpt::FileExplorer(
                    path.clone(),
                )))
                .into(),
        ]);

        Container::new(col)
            .padding(0)
            .style(|theme| {
                container::Style::default()
                    .background(theme.palette().background)
                    .border(Border::default().rounded(BORDER_RADIUS))
            })
            .width(200)
            .into()
    }

    pub fn button_style(theme: &Theme, status: button::Status) -> button::Style {
        match status {
            button::Status::Active | button::Status::Disabled => button::Style {
                background: Some(theme.extended_palette().primary.base.color).map(Color::into),
                text_color: theme.extended_palette().primary.base.text,
                border: Border::default(),
                shadow: Default::default(),
                snap: Default::default(),
            },
            button::Status::Hovered | button::Status::Pressed => button::Style {
                background: Some(theme.extended_palette().primary.strong.color).map(Color::into),
                text_color: theme.extended_palette().primary.strong.text,
                border: Default::default(),
                shadow: Default::default(),
                snap: Default::default(),
            },
        }
    }

    #[derive(Debug, Clone)]
    pub enum ContextMenuOpt {
        View(usize),
        CopyPath(PathBuf),
        FileExplorer(PathBuf),
    }
}
