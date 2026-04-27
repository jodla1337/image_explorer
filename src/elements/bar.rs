use iced::{
    Border, Color, Element, Theme,
    widget::{Button, Image, Svg, button, container, image::Handle, space, text_input},
};

use crate::{Message, elements::BORDER_RADIUS, resources};

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

pub fn icon_button(file_name: &'static str, on_press: Message) -> Button<'static, Message> {
    // let content: Element<'_, Message> = image_elem(handle);
    let content = svg_path(file_name);

    button(content).style(button_style).on_press(on_press)
}

pub fn image_elem(handle: Option<&Handle>) -> Element<'static, Message> {
    if let Some(h) = handle {
        Image::new(h).width(ICON_SIZE).height(ICON_SIZE).into()
    } else {
        space().width(ICON_SIZE).height(ICON_SIZE).into()
    }
}

pub fn svg_path<'a>(file_name: &'a str) -> Element<'a, Message> {
    Svg::from_path(resources::dir().join(file_name))
        .width(ICON_SIZE)
        .height(ICON_SIZE)
        .into()
}

pub mod footer {
    use iced::{
        Alignment, Element, Length,
        widget::{container, row, text, text_input},
    };

    use super::{BAR_HEIGHT, BUTTON_WIDTH, PAD, SPACE, bar_style, icon_button, text_input_style};

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
                "arrow_left.svg",
                if index > 0 {
                    Message::Mode(Mode::Viewer(index - 1))
                } else {
                    Message::None
                },
            )
            .into(),
            icon_button(
                "arrow_right.svg",
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
        let content: [Element<'_, Message>; 4] = [
            icon_button(
                "arrow_left.svg",
                if state.page > 0 {
                    Message::Page(state.page - 1)
                } else {
                    Message::None
                },
            )
            .into(),
            icon_button("arrow_right.svg", Message::Page(state.page + 1)).into(),
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
        Alignment, Border, Color, Element, Font, Length, Renderer, Shadow, Theme, Vector,
        border::{Radius, radius},
        font::Weight,
        widget::{
            Button, Tooltip, button, column, container, rich_text, row, space, span, text,
            text_input, tooltip,
        },
    };
    use iced_aw::{Menu, menu_bar, menu_items, style::menu_bar};

    use super::{
        BAR_HEIGHT, BUTTON_WIDTH, PAD, SPACE, bar_style, button_style, icon_button, svg_path,
        text_input_style,
    };

    use crate::{
        AppState, Message, Mode,
        config::{FilterVariations, Order, SortBy},
        elements::BORDER_RADIUS,
        img::ImageFormat,
    };

    pub fn header_viewer(_state: &AppState, _index: usize) -> Element<'_, Message> {
        let content = row![icon_button("back.svg", Message::Mode(Mode::Explorer)),];

        header_tpl(content.into(), None).into()
    }

    pub fn header_explorer(state: &AppState) -> Element<'_, Message> {
        let menu_template = |items: Vec<iced_aw::menu::Item<'static, _, Theme, _>>| {
            Menu::<Message, Theme, Renderer>::new(items)
                .offset(10.0)
                .spacing(SPACE)
                .width(BUTTON_WIDTH)
        };

        let mb = menu_bar!(
            (icon_button("clear.svg", Message::SortBy(None))),
            (
                menu_button(
                    row![svg_path("sort.svg"), "Sort by"].spacing(SPACE),
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
                        Order::Ascending => "arrow_up.svg",
                        Order::Descending => "arrow_down.svg",
                    }
                },
                Message::SortOrder(state.config.query.sort_order.switch())
            )),
            (icon_button("clear.svg", Message::ClearFilters)),
            (
                menu_button(
                    row![
                        svg_path("filter.svg"),
                        // image_elem(state.resources.filter.as_ref().map(|x| x.handle())),
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
                            state.config.query.filter.extensions.is_some()
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
                                    .extensions
                                    .as_ref()
                                    .is_some_and(|x| x.contains(&ImageFormat::Jpg))
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
                                    .extensions
                                    .as_ref()
                                    .is_some_and(|x| x.contains(&ImageFormat::Png))
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
                                    .extensions
                                    .as_ref()
                                    .is_some_and(|x| x.contains(&ImageFormat::Bmp))
                            )
                            .width(75)),
                            (menu_button(
                                text("WebP").center(),
                                Some(Message::Filter(FilterVariations::Extension(Some(
                                    ImageFormat::Webp
                                )))),
                                state
                                    .config
                                    .query
                                    .filter
                                    .extensions
                                    .as_ref()
                                    .is_some_and(|x| x.contains(&ImageFormat::Webp))
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
                                    .extensions
                                    .as_ref()
                                    .is_some_and(|x| x.contains(&ImageFormat::Gif))
                            )
                            .width(75)),
                            (menu_button(
                                text("PNM").center(),
                                Some(Message::Filter(FilterVariations::Extension(Some(
                                    ImageFormat::Pnm
                                )))),
                                state
                                    .config
                                    .query
                                    .filter
                                    .extensions
                                    .as_ref()
                                    .is_some_and(|x| x.contains(&ImageFormat::Pnm))
                            )
                            .width(75)),
                            (menu_button(
                                text("TIFF").center(),
                                Some(Message::Filter(FilterVariations::Extension(Some(
                                    ImageFormat::Tiff
                                )))),
                                state
                                    .config
                                    .query
                                    .filter
                                    .extensions
                                    .as_ref()
                                    .is_some_and(|x| x.contains(&ImageFormat::Tiff))
                            )
                            .width(75)),
                            (menu_button(
                                text("TGA").center(),
                                Some(Message::Filter(FilterVariations::Extension(Some(
                                    ImageFormat::Tga
                                )))),
                                state
                                    .config
                                    .query
                                    .filter
                                    .extensions
                                    .as_ref()
                                    .is_some_and(|x| x.contains(&ImageFormat::Tga))
                            )
                            .width(75)),
                            (menu_button(
                                text("DDS").center(),
                                Some(Message::Filter(FilterVariations::Extension(Some(
                                    ImageFormat::Dds
                                )))),
                                state
                                    .config
                                    .query
                                    .filter
                                    .extensions
                                    .as_ref()
                                    .is_some_and(|x| x.contains(&ImageFormat::Dds))
                            )
                            .width(75)),
                            (menu_button(
                                text("ICO").center(),
                                Some(Message::Filter(FilterVariations::Extension(Some(
                                    ImageFormat::Ico
                                )))),
                                state
                                    .config
                                    .query
                                    .filter
                                    .extensions
                                    .as_ref()
                                    .is_some_and(|x| x.contains(&ImageFormat::Ico))
                            )
                            .width(75)),
                            (menu_button(
                                text("HDR").center(),
                                Some(Message::Filter(FilterVariations::Extension(Some(
                                    ImageFormat::Hdr
                                )))),
                                state
                                    .config
                                    .query
                                    .filter
                                    .extensions
                                    .as_ref()
                                    .is_some_and(|x| x.contains(&ImageFormat::Hdr))
                            )
                            .width(75)),
                            (menu_button(
                                text("OpenEXR").center(),
                                Some(Message::Filter(FilterVariations::Extension(Some(
                                    ImageFormat::Exr
                                )))),
                                state
                                    .config
                                    .query
                                    .filter
                                    .extensions
                                    .as_ref()
                                    .is_some_and(|x| x.contains(&ImageFormat::Exr))
                            )
                            .width(75)),
                            (menu_button(
                                text("AVIF").center(),
                                Some(Message::Filter(FilterVariations::Extension(Some(
                                    ImageFormat::Avif
                                )))),
                                state
                                    .config
                                    .query
                                    .filter
                                    .extensions
                                    .as_ref()
                                    .is_some_and(|x| x.contains(&ImageFormat::Avif))
                            )
                            .width(75)),
                            (menu_button(
                                text("QOI").center(),
                                Some(Message::Filter(FilterVariations::Extension(Some(
                                    ImageFormat::Qoi
                                )))),
                                state
                                    .config
                                    .query
                                    .filter
                                    .extensions
                                    .as_ref()
                                    .is_some_and(|x| x.contains(&ImageFormat::Qoi))
                            )
                            .width(75)),
                            (menu_button(
                                svg_path("clear.svg"),
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
                row![svg_path("trash.svg"), "Clear all"].spacing(SPACE),
                Some(Message::ClearQuery),
                false
            )),
        )
        .width(Length::Shrink)
        .height(Length::Fill)
        .style(menubar_style)
        .spacing(SPACE)
        .close_on_background_click(true);

        row![header_tpl(
            mb.into(),
            Some(tooltip(
                svg_path("info.svg"),
                info_tooltip(state),
                tooltip::Position::FollowCursor
            ))
        ),]
        .into()
    }

    fn header_tpl<'a>(
        content: Element<'a, Message>,
        tooltip: Option<Tooltip<'a, Message>>,
    ) -> Element<'a, Message> {
        let mut r = row![content, space::horizontal().width(Length::Fill)];

        if let Some(tooltip) = tooltip {
            r = r.push(
                container(tooltip)
                    .align_y(Alignment::Center)
                    .height(Length::Fill)
                    .padding(PAD),
            );
        };

        container(r)
            .width(Length::Fill)
            .height(BAR_HEIGHT)
            .padding(PAD)
            .align_y(Alignment::Center)
            .style(bar_style)
            .into()
    }

    fn info_tooltip(state: &AppState) -> Element<'_, Message> {
        container(
            column![
                rich_text![
                    span::<'_, u8, Font>("Executed in "),
                    span::<'_, u8, Font>(format!(
                        "{:.2}s",
                        state.loading.time_loading.as_secs_f32()
                    ))
                    .font(Font {
                        weight: Weight::Bold,
                        ..Default::default()
                    })
                ],
                rich_text![
                    span::<'_, u8, Font>("Found "),
                    span::<'_, u8, Font>(format!("{}", state.images.len())).font(Font {
                        weight: Weight::Bold,
                        ..Default::default()
                    }),
                    span::<'_, u8, Font>(" items")
                ],
                text(format!("{}", state.loading.counter))
            ]
            .spacing(SPACE),
        )
        .style(|theme: &Theme| container::Style {
            text_color: Some(theme.extended_palette().primary.base.text),
            background: Some(theme.extended_palette().primary.base.color).map(Color::into),
            border: Border {
                color: theme.extended_palette().primary.base.text,
                width: PAD as f32,
                radius: radius(BORDER_RADIUS),
            },
            shadow: Shadow {
                color: Color::BLACK.scale_alpha(0.6),
                offset: Vector::new(7.0, 7.0),
                blur_radius: 10.0,
            },
            snap: Default::default(),
        })
        .padding(PAD * 2)
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
