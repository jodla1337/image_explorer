use crate::{
    AppState,
    config::{Order, SortBy},
    img::ImageData,
};

pub fn data_from_pagepos(state: &AppState, page_position: usize) -> Option<&ImageData> {
    let data = if state.config.query.filter.any() {
        state.config.images.get(page_position).map(|index| {
            state
                .images
                .get(*index)
                .expect("config.images contains only valid indices")
        })?
        // .unwrap_or(&state.default_img)
    } else {
        let index = {
            if let Some(sortby) = &state.config.query.sortedby {
                let sorted_arr = match sortby {
                    SortBy::Size => &state.bysize,
                    SortBy::TimeCreated => &state.bycreation,
                    SortBy::TimeModified => &state.bymodification,
                };

                if sorted_arr.len() <= page_position {
                    return None;
                }

                let index = match &state.config.query.sort_order {
                    Order::Ascending => page_position,
                    Order::Descending => sorted_arr.len() - 1 - page_position,
                };

                *sorted_arr.get(index)?
            } else {
                page_position
            }
        };

        state.images.get(index)?
        // .unwrap_or(&state.default_img)
    };

    Some(data)
}
