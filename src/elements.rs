use std::path::PathBuf;

use iced::{
    Color, Element, Length,
    widget::{
        Column, Container, Image, button, column,
        image::{self, Viewer},
        row, text, text_input,
    },
};

use crate::config::{Filter, SortBy};

use super::{
    AppState, Message, Mode, PAGESIZE,
    img::{ImageData, ImageFormat},
};

pub fn viewer<'a>(state: &'a AppState, path: &'a PathBuf) -> Element<'a, Message> {
    row![
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
    .into()
}

pub fn explorer(state: &AppState) -> Element<'_, Message> {
    row![
        images_column(state),
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
            ],
            text("filter by:"),
            row![
                text("Extension:"),
                button("JPG").on_press(Message::Filter(Some(Filter::Extension(ImageFormat::Jpg)))),
                button("PNG").on_press(Message::Filter(Some(Filter::Extension(ImageFormat::Png)))),
                button("BMP").on_press(Message::Filter(Some(Filter::Extension(ImageFormat::Bmp))))
            ],
            text_input("Starts with", {
                if let Some(Filter::StartsWith(x)) = &state.config.filter_opts.filter {
                    x
                } else {
                    ""
                }
            })
            .on_input(|x| Message::Filter(Some(Filter::StartsWith(x)))),
            text_input("Contains", {
                if let Some(Filter::Contains(x)) = &state.config.filter_opts.filter {
                    x
                } else {
                    ""
                }
            })
            .on_input(|x| Message::Filter(Some(Filter::Contains(x)))),
            button("none").on_press(Message::Filter(None)),
        ]
    ]
    .padding(10)
    .into()
}

pub fn images_column(state: &AppState) -> Column<'_, Message> {
    let mut col: Column<'_, Message> = column![];

    let mut i: usize = 0;

    let page_position = (state.page * PAGESIZE) as usize;

    while i < 16 {
        let mut row_ = row![];
        for _ in 0..4 {
            let data = if let Some(_) = state.config.filter_opts.filter {
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
                let index_opt = match state.config.filter_opts.sortedby {
                    crate::config::SortBy::None => Some(page_position + i),
                    crate::config::SortBy::TimeCreated => {
                        state.bydate.get(page_position + i).copied()
                    }
                    crate::config::SortBy::Size => state.bysize.get(page_position + i).copied(),
                };

                if let Some(index) = index_opt {
                    state.images.get(index).unwrap_or(&state.default_img)
                } else {
                    &ImageData::default()
                }
            };

            // println!("Format: {:?}", data.format);

            let frame: Element<'_, Message> = match &data.allocation {
                Some(allocation) => button(
                    Container::new(Image::new(allocation.handle()).width(200).height(200))
                        .width(200)
                        .height(200),
                )
                .on_press(Message::Mode(Mode::Viewer(data.path.clone())))
                .into(),
                None => Container::new(
                    text("Error")
                        .color(Color::from_rgb(1.0, 0.02, 0.02))
                        .center(),
                )
                .width(200)
                .height(200)
                .into(),
            };

            row_ = row_.push(frame);

            i += 1;
        }
        col = col.push(row_);
    }

    col
}
