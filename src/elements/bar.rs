use iced::{
    Border, Color, Element, Theme,
    widget::{Button, Image, button, container, image::Handle, space, text, text_input},
};

use crate::{Message, elements::BORDER_RADIUS};

const BAR_HEIGHT: u32 = 50;
const BUTTON_WIDTH: u32 = 125;
const SPACE: u32 = 5;
const PAD: u16 = 5;

fn bar_style(theme: &Theme) -> container::Style {
    container::primary(theme).background(theme.extended_palette().primary.base.color)
}

pub fn button_style(theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Active | button::Status::Disabled => button::Style {
            background: Some(theme.extended_palette().primary.base.color).map(Color::into),
            text_color: theme.extended_palette().primary.base.text,
            border: Border::default().rounded(BORDER_RADIUS),
            shadow: Default::default(),
            snap: Default::default(),
        },
        button::Status::Hovered | button::Status::Pressed => button::Style {
            background: Some(theme.extended_palette().primary.strong.color).map(Color::into),
            text_color: theme.extended_palette().primary.strong.text,
            border: Border::default().rounded(BORDER_RADIUS),
            shadow: Default::default(),
            snap: Default::default(),
        },
    }
}

pub fn text_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    text_input::Style {
        border: Border::default().rounded(BORDER_RADIUS),
        ..text_input::default(theme, status)
    }
}

const ICON_SIZE: u32 = 20;

pub fn icon_button<'a>(handle: Option<&Handle>, on_press: Message) -> Button<'a, Message> {
    let content: Element<'_, Message> = image_elem(handle);

    button(content).style(button_style).on_press(on_press)
}

pub fn image_elem(handle: Option<&Handle>) -> Element<'static, Message> {
    if let Some(h) = handle {
        Image::new(h).width(ICON_SIZE).height(ICON_SIZE).into()
    } else {
        space().width(ICON_SIZE).height(ICON_SIZE).into()
    }
}

pub mod footer {
    use iced::{
        Alignment, Element, Length,
        widget::{Text, button, column, container, row, text, text_input},
    };

    use super::{
        BAR_HEIGHT, BUTTON_WIDTH, PAD, SPACE, bar_style, button_style, icon_button,
        text_input_style,
    };

    use crate::{AppState, Message, Mode, PageInput};

    fn footer<'a>(content: impl IntoIterator<Item = Element<'a, Message>>) -> Element<'a, Message> {
        let r = row![]
            .height(Length::Fill)
            .padding(PAD)
            .spacing(SPACE)
            .align_y(Alignment::Center)
            .extend(content); // here i add content

        container(r)
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .height(BAR_HEIGHT)
            .style(bar_style)
            .into()
    }

    pub fn footer_viewer(state: &AppState, index: usize) -> Element<'_, Message> {
        let content = [
            icon_button(
                state.resources.arrow_left.as_ref().map(|x| x.handle()),
                if index > 0 {
                    Message::Mode(Mode::Viewer(index - 1))
                } else {
                    Message::None
                },
            )
            .into(),
            icon_button(
                state.resources.arrow_right.as_ref().map(|x| x.handle()),
                if index + 1 < state.images.len() {
                    Message::Mode(Mode::Viewer(index + 1))
                } else {
                    Message::None
                },
            )
            .into(),
        ];

        footer(content)
    }

    pub fn footer_explorer(state: &AppState) -> Element<'_, Message> {
        let max_pages = state.max_pages();

        let content: [Element<'_, Message>; 4] = [
            icon_button(
                state.resources.arrow_left.as_ref().map(|x| x.handle()),
                if state.page > 0 {
                    Message::Page(state.page - 1)
                } else {
                    Message::None
                },
            )
            .into(),
            icon_button(
                state.resources.arrow_right.as_ref().map(|x| x.handle()),
                Message::Page(state.page + 1),
            )
            .into(),
            text_input(
                "",
                &state
                    .page_input
                    .clone()
                    .unwrap_or((state.page + 1).to_string()),
            )
            .style(text_input_style)
            .on_input(|x| Message::PageInput(PageInput::OnInput(x)))
            .on_submit(Message::PageInput(PageInput::Submit))
            .width(BUTTON_WIDTH / 2)
            .into(),
            text(format!(
                "of {}",
                if state.config.query.filter.any() && !state.config.finished_searching {
                    "?".to_string()
                } else {
                    (state.max_pages() + 1).to_string()
                }
            ))
            .into(),
        ];

        footer(content)
    }
}

pub mod mymenu {
    use iced::{
        Alignment, Border, Color, Element, Length, Renderer, Shadow, Theme, Vector,
        border::Radius,
        overlay::menu,
        widget::{Button, button, column, container, row, space, text, text_input},
    };
    use iced_aw::{Menu, MenuBar, core::renderer, menu_bar, menu_items, style::menu_bar};

    use super::{BAR_HEIGHT, BUTTON_WIDTH, PAD, SPACE, bar_style, button_style, text_input_style};

    use crate::{
        AppState, Message, Mode,
        config::{FilterVariations, Order, SortBy},
        elements::{
            BORDER_RADIUS,
            bar::{ICON_SIZE, icon_button, image_elem},
        },
        img::ImageFormat,
    };

    pub fn header_viewer(state: &AppState, _index: usize) -> Element<'_, Message> {
        let content = column![icon_button(
            state.resources.back.as_ref().map(|x| x.handle()),
            Message::Mode(Mode::Explorer)
        )];

        header_tpl(content.into()).into()
    }

    pub fn header_explorer(state: &AppState) -> Element<'static, Message> {
        let menu_template = |items: Vec<iced_aw::menu::Item<'static, _, Theme, _>>| {
            Menu::<Message, Theme, Renderer>::new(items)
                .offset(10.0)
                .spacing(SPACE)
                .width(BUTTON_WIDTH)
        };

        let mb = menu_bar!(
            (icon_button(
                state.resources.clear.as_ref().map(|x| x.handle()),
                Message::SortBy(None)
            )),
            (
                menu_button(
                    row![
                        image_elem(state.resources.sort.as_ref().map(|x| x.handle())),
                        "Sort by"
                    ]
                    .spacing(SPACE),
                    None,
                    state.config.query.sortedby != None
                ),
                menu_template(menu_items!(
                    (menu_button(
                        "Size",
                        Some(Message::SortBy(Some(SortBy::Size))),
                        state.config.query.sortedby == Some(SortBy::Size)
                    )),
                    (menu_button(
                        "Creation time",
                        Some(Message::SortBy(Some(SortBy::TimeCreated))),
                        state.config.query.sortedby == Some(SortBy::TimeCreated)
                    )),
                    (menu_button(
                        "Modification time",
                        Some(Message::SortBy(Some(SortBy::TimeModified))),
                        state.config.query.sortedby == Some(SortBy::TimeModified)
                    ))
                )) // .width(BUTTON_WIDTH + SPACE + ICON_SIZE)
            ),
            (icon_button(
                {
                    match &state.config.query.sort_order {
                        Order::Ascending => state.resources.arrow_up.as_ref().map(|x| x.handle()),
                        Order::Descending => {
                            state.resources.arrow_down.as_ref().map(|x| x.handle())
                        }
                    }
                },
                Message::SortOrder(state.config.query.sort_order.switch())
            )),
            (icon_button(
                state.resources.clear.as_ref().map(|x| x.handle()),
                Message::ClearFilters
            )),
            (
                menu_button(
                    row![
                        image_elem(state.resources.filter.as_ref().map(|x| x.handle())),
                        "Filter"
                    ]
                    .spacing(SPACE),
                    None,
                    state.config.query.filter.any()
                ),
                menu_template(menu_items!(
                    (
                        menu_button(
                            "Extension",
                            None,
                            state.config.query.filter.extension.is_some()
                        ),
                        menu_template(menu_items!(
                            (menu_button(
                                text("JPG").center(),
                                Some(Message::Filter(FilterVariations::Extension(Some(
                                    ImageFormat::Jpg
                                )))),
                                state
                                    .config
                                    .query
                                    .filter
                                    .extension
                                    .is_some_and(|x| x == ImageFormat::Jpg)
                            )
                            .width(75)),
                            (menu_button(
                                text("PNG").center(),
                                Some(Message::Filter(FilterVariations::Extension(Some(
                                    ImageFormat::Png
                                )))),
                                state
                                    .config
                                    .query
                                    .filter
                                    .extension
                                    .is_some_and(|x| x == ImageFormat::Png)
                            )
                            .width(75)),
                            (menu_button(
                                text("BMP").center(),
                                Some(Message::Filter(FilterVariations::Extension(Some(
                                    ImageFormat::Bmp
                                )))),
                                state
                                    .config
                                    .query
                                    .filter
                                    .extension
                                    .is_some_and(|x| x == ImageFormat::Bmp)
                            )
                            .width(75)),
                            (menu_button(
                                text("WEBP").center(),
                                Some(Message::Filter(FilterVariations::Extension(Some(
                                    ImageFormat::Webp
                                )))),
                                state
                                    .config
                                    .query
                                    .filter
                                    .extension
                                    .is_some_and(|x| x == ImageFormat::Webp)
                            )
                            .width(75)),
                            (menu_button(
                                text("GIF").center(),
                                Some(Message::Filter(FilterVariations::Extension(Some(
                                    ImageFormat::Gif
                                )))),
                                state
                                    .config
                                    .query
                                    .filter
                                    .extension
                                    .is_some_and(|x| x == ImageFormat::Gif)
                            )
                            .width(75)),
                            (menu_button(
                                image_elem(state.resources.clear.as_ref().map(|x| x.handle())),
                                Some(Message::Filter(FilterVariations::Extension(None))),
                                false
                            )
                            .width(75))
                        ))
                        .width(75)
                    ),
                    (text_input(
                        "Starts with...",
                        state
                            .config
                            .query
                            .filter
                            .startswith
                            .as_ref()
                            .unwrap_or(&"".to_string())
                    )
                    .style(text_input_style)
                    .on_input(|x| Message::Filter(
                        FilterVariations::StartsWith({
                            if !x.is_empty() { Some(x) } else { None }
                        })
                    ))),
                    (text_input(
                        "Contains...",
                        state
                            .config
                            .query
                            .filter
                            .contains
                            .as_ref()
                            .unwrap_or(&"".to_string())
                    )
                    .style(text_input_style)
                    .on_input(|x| Message::Filter(FilterVariations::Contains({
                        if !x.is_empty() { Some(x) } else { None }
                    }))))
                ))
            ),
            (menu_button(
                row![
                    image_elem(state.resources.trash.as_ref().map(|x| x.handle())),
                    "Clear all"
                ]
                .spacing(SPACE),
                Some(Message::ClearQuery),
                false
            ))
        )
        .width(Length::Shrink)
        .height(Length::Fill)
        .style(menubar_style)
        .spacing(SPACE)
        .close_on_background_click(true);

        header_tpl(mb.into())
    }

    fn header_tpl<'a>(content: Element<'a, Message>) -> Element<'a, Message> {
        let r = row![content, space::horizontal().width(Length::Fill)];

        container(r)
            .width(Length::Fill)
            .height(BAR_HEIGHT)
            .padding(PAD)
            .align_y(Alignment::Center)
            .style(bar_style)
            .into()
    }

    fn menubar_style(theme: &Theme, _: iced_aw::style::Status) -> menu_bar::Style {
        menu_bar::Style {
            bar_background: theme.extended_palette().primary.base.color.into(),
            bar_border: Default::default(),
            bar_shadow: Default::default(),
            menu_background: theme.extended_palette().primary.base.color.into(),
            menu_border: Border::default().rounded(Radius::new(0).bottom(BORDER_RADIUS)),
            menu_shadow: Shadow {
                color: Color::BLACK.scale_alpha(0.6),
                offset: Vector::new(7.0, 7.0),
                blur_radius: 10.0,
            },
            path: Color::WHITE.into(),
            path_border: Default::default(),
        }
    }

    pub fn menu_button<'a>(
        content: impl Into<Element<'a, Message>>,
        on_press: Option<Message>,
        condition: bool,
    ) -> Button<'a, Message> {
        button(content)
            .width(BUTTON_WIDTH)
            .on_press(on_press.unwrap_or(Message::None))
            .style(move |t, s| {
                let mut but = button_style(t, s);
                if condition {
                    but = but.with_background(Color::from_rgb(0.7, 0.1, 0.1));
                }
                but
            })
    }
}
