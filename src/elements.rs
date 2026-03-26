use iced::{
    Color, Element,
    widget::{Column, Container, Image, button, column, row, text},
};

use super::{AppState, Message, Mode, PAGESIZE, img::ImageData};

pub fn images_column(state: &AppState) -> Column<'_, Message> {
    let mut col: Column<'_, Message> = column![];

    let mut i = 0;

    while i < 16 {
        let mut row_ = row![];
        for _ in 0..4 {
            let index_opt = match state.config.filter_opts.sortedby {
                crate::config::SortBy::None => Some((state.page * PAGESIZE + i) as usize),
                crate::config::SortBy::TimeCreated => state
                    .bydate
                    .get((state.page * PAGESIZE + i) as usize)
                    .copied(),
                crate::config::SortBy::Size => state
                    .bysize
                    .get((state.page * PAGESIZE + i) as usize)
                    .copied(),
            };

            let data = if let Some(index) = index_opt {
                if let Some(data_ptr) = state.images.get(index) {
                    data_ptr
                } else {
                    &ImageData::default()
                }
            } else {
                &ImageData::default()
            };

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
