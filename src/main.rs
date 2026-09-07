mod localization;
mod minigame;

use axum::{
    Router,
    extract::{self, RawQuery},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use base64::prelude::*;
use bilrost::{Enumeration, Message, OwnedMessage};
use laser_pdf::{
    Element, LineCapStyle, LinkTarget, Metadata, Pdf,
    elements::{
        align_preferred_height_bottom::AlignPreferredHeightBottom,
        break_list::BreakList,
        break_whole::BreakWhole,
        center_in_preferred_height::CenterInPreferredHeight,
        column::Column,
        expand_to_preferred_height::ExpandToPreferredHeight,
        h_align::{HAlign, HorizontalAlignment},
        line::Line,
        link::Link,
        padding::Padding,
        page::Page,
        rich_text::{RichText, Span},
        row::{Flex, Row},
        stack::Stack,
        styled_box::StyledBox,
        text::{Text, TextAlign},
        titled::Titled,
        v_gap::VGap,
    },
    fonts::truetype::TruetypeFont as Font,
};
use mimalloc::MiMalloc;

use crate::{localization::WorkSection, minigame::Minigame};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const REGULAR: &[u8] = include_bytes!("../assets/Lexend/Lexend-Regular.ttf");
const BOLD: &[u8] = include_bytes!("../assets/Lexend/Lexend-Bold.ttf");
const SEMI_BOLD: &[u8] = include_bytes!("../assets/Lexend/Lexend-SemiBold.ttf");
const EMOJI: &[u8] = include_bytes!("../assets/Noto_Emoji/static/NotoEmoji-Bold.ttf");

const LOGO: &[u8] = include_bytes!("../assets/logo.png");

fn bullet_line(state: &State, bold: &Font, content: impl Element) -> impl Element {
    BreakWhole(Row::new(move |row_content| {
        row_content.add(
            &HAlign(
                HorizontalAlignment::Center,
                Text {
                    color: state.text_color(),
                    ..Text::new("•", bold, 16.)
                }
                .with_vertical_padding(-1.),
            ),
            Flex::Fixed(8.),
        );
        row_content.add(&content, Flex::Expand(1));
    }))
}

fn work_section_element<'a>(
    state: &State,
    regular: &Font,
    bold: &Font,
    work_section: &WorkSection,
) -> impl Element {
    Column::new(|content| {
        let mut content = content
            .add(
                &Row::new(|content| {
                    content.add(
                        &Text {
                            color: state.text_color(),
                            ..Text::new(work_section.title, bold, 12.)
                        },
                        Flex::Expand(1),
                    );
                    content.add(
                        &AlignPreferredHeightBottom(Text {
                            color: state.text_color(),
                            ..Text::new(work_section.date_range, bold, 11.)
                        }),
                        Flex::SelfSized,
                    );
                })
                .expand(),
            )?
            .add(&Row::new(|content| {
                content.add(
                    &Text {
                        color: state.text_color(),
                        ..Text::new(work_section.company, regular, 11.)
                    },
                    Flex::Expand(1),
                );
                content.add(
                    &Text {
                        color: state.text_color(),
                        ..Text::new(work_section.location, regular, 11.)
                    },
                    Flex::SelfSized,
                );
            }))?
            .add(&VGap(1.))?;

        for &project in work_section.projects {
            content = content.add(&bullet_line(
                state,
                bold,
                RichText::new(project.iter().map(|span| Span {
                    link: span.link.map(LinkTarget::Uri),
                    color: if span.link.is_some() {
                        state.link_color()
                    } else {
                        state.text_color()
                    },
                    ..Span::new(span.text, if span.bold { bold } else { regular }, 11.)
                })),
            ))?;
        }

        None
    })
}

fn technology_line<'a>(
    state: &State,
    regular: &'a Font,
    bold: &'a Font,
    title: &'a str,
    content: &'a str,
) -> impl Element {
    bullet_line(
        state,
        bold,
        Row::new(|row_content| {
            row_content.add(
                &Text {
                    color: state.text_color(),
                    ..Text::new(title, bold, 11.)
                },
                Flex::Expand(1),
            );
            row_content.add(
                &Text {
                    color: state.text_color(),
                    ..Text::new(content, regular, 11.)
                },
                Flex::Expand(5),
            );
        }),
    )
}

fn open_source_line<'a>(
    state: &State,
    regular: &'a Font,
    bold: &'a Font,
    spans: &'a [localization::Span],
) -> impl Element {
    bullet_line(
        state,
        bold,
        RichText::new(spans.iter().map(|span| Span {
            link: span.link.map(LinkTarget::Uri),
            color: if span.link.is_some() {
                state.link_color()
            } else {
                state.text_color()
            },
            ..Span::new(span.text, if span.bold { bold } else { regular }, 11.)
        })),
    )
}

enum SwitchItemState<'a> {
    Active,
    Inactive { link: &'a str },
}

struct SwitchItem<'a> {
    label: &'a str,
    state: SwitchItemState<'a>,
}

fn switch(regular: &Font, items: &[SwitchItem<'_>]) -> impl Element {
    Row::new(move |content| {
        for item in items {
            match item.state {
                SwitchItemState::Active => content.add(
                    &StyledBox {
                        fill: Some(0xff_ff_ff_ff),
                        padding_left: 2.,
                        padding_right: 2.,
                        padding_top: 2.,
                        padding_bottom: 2.,
                        border_radius: 1.,
                        ..StyledBox::new(Text {
                            ..Text::new(item.label, regular, 10.)
                        })
                    },
                    Flex::SelfSized,
                ),
                SwitchItemState::Inactive { link } => content.add(
                    &Link {
                        element: Padding {
                            left: 2.,
                            right: 2.,
                            top: 2.,
                            bottom: 2.,
                            element: Text {
                                color: 0xff_ff_ff_ff,
                                ..Text::new(item.label, regular, 10.)
                            },
                        },
                        target: LinkTarget::Uri(link),
                    },
                    Flex::SelfSized,
                ),
            }
        }
    })
}

fn section(state: &State, bold: &Font, title: &str, content: impl Element) -> impl Element {
    Titled {
        title: Column::new(|content| {
            content
                .add(&Text {
                    color: state.text_color(),
                    ..Text::new(title, bold, 12.)
                })?
                .add(&Line {
                    style: laser_pdf::LineStyle {
                        thickness: 0.,
                        color: state.text_color(),
                        dash_pattern: None,
                        cap_style: LineCapStyle::Butt,
                    },
                })?;
            None
        })
        .with_gap(1.),
        content,
        gap: 2.,
        collapse_on_empty_content: true,
    }
}

fn resume(
    config: &Config,
    state: &State,
    regular: &Font,
    bold: &Font,
    semi_bold: &Font,
    emoji: &Font,
) -> impl Element {
    let secret_pass = state.secret == Some(config.secret);

    // let localization = match state.language {
    //     Language::English => &localization::EN,
    //     Language::German => &localization::DE,
    // };
    let localization = &localization::EN;

    StyledBox {
        fill: state.background_color(),
        ..StyledBox::new(Page {
            primary: Column::new(move |content| {
                let mut content = content.add(
                    &Stack {
                        expand: true,
                        content: |content| {
                            content.add(&Text {
                                align: TextAlign::Center,
                                color: state.text_color(),
                                ..Text::new("Florian Plattner", regular, 24.)
                            });

                            content.add(&CenterInPreferredHeight(HAlign(
                                HorizontalAlignment::Right,
                                Row::new(|content| {
                                    content.add(
                                        &Link {
                                            target: LinkTarget::Uri(
                                                "https://github.com/lasernoises/resume",
                                            ),
                                            element: StyledBox {
                                                padding_left: 2.,
                                                padding_right: 2.,
                                                padding_top: 2.,
                                                padding_bottom: 2.,
                                                border_radius: 1.,
                                                fill: Some(0x21_4e_7a_ff),
                                                ..StyledBox::new(ExpandToPreferredHeight(
                                                    Text {
                                                        color: 0xff_ff_ff_ff,
                                                        align: TextAlign::Center,
                                                        extra_character_spacing: -0.5,
                                                        ..Text::new("</>", regular, 12.)
                                                    }
                                                    .with_padding_top(0.3),
                                                ))
                                            },
                                        },
                                        Flex::SelfSized,
                                    );

                                    content.add(
                                        &Link {
                                            target: LinkTarget::Uri(
                                                &State {
                                                    open: !state.open,
                                                    ..*state
                                                }
                                                .to_link(config),
                                            ),
                                            element: StyledBox {
                                                padding_left: 2.,
                                                padding_right: 2.,
                                                padding_top: 2.,
                                                padding_bottom: 2.,
                                                border_radius: 1.,
                                                fill: Some(0x21_4e_7a_ff),
                                                ..StyledBox::new(Text {
                                                    color: 0xff_ff_ff_ff,
                                                    align: TextAlign::Center,
                                                    ..Text::new("⚙", emoji, 16.)
                                                })
                                            },
                                        },
                                        Flex::SelfSized,
                                    );
                                })
                                .with_gap(2.)
                                .expand(),
                            )));
                        },
                    }
                    .with_padding_bottom(2.),
                )?;

                if state.open {
                    let _language_switch = [
                        SwitchItem {
                            label: "EN",
                            state: if state.language == Language::English {
                                SwitchItemState::Active
                            } else {
                                SwitchItemState::Inactive {
                                    link: &State {
                                        language: Language::English,
                                        ..*state
                                    }
                                    .to_link(config),
                                }
                            },
                        },
                        SwitchItem {
                            label: "DE",
                            state: if state.language == Language::German {
                                SwitchItemState::Active
                            } else {
                                SwitchItemState::Inactive {
                                    link: &State {
                                        language: Language::German,
                                        ..*state
                                    }
                                    .to_link(config),
                                }
                            },
                        },
                    ];

                    let theme_switch = [
                        SwitchItem {
                            label: localization.settings_theme_light,
                            state: if state.theme == Theme::Light {
                                SwitchItemState::Active
                            } else {
                                SwitchItemState::Inactive {
                                    link: &State {
                                        theme: Theme::Light,
                                        ..*state
                                    }
                                    .to_link(config),
                                }
                            },
                        },
                        SwitchItem {
                            label: localization.settings_theme_dark,
                            state: if state.theme == Theme::Dark {
                                SwitchItemState::Active
                            } else {
                                SwitchItemState::Inactive {
                                    link: &State {
                                        theme: Theme::Dark,
                                        ..*state
                                    }
                                    .to_link(config),
                                }
                            },
                        },
                    ];

                    let minigame_switch = [
                        SwitchItem {
                            label: localization.settings_minigame_off,
                            state: if state.minigame.is_some() {
                                SwitchItemState::Inactive {
                                    link: &State {
                                        minigame: None,
                                        ..*state
                                    }
                                    .to_link(config),
                                }
                            } else {
                                SwitchItemState::Active
                            },
                        },
                        SwitchItem {
                            label: localization.settings_minigame_on,
                            state: if state.minigame.is_none() {
                                SwitchItemState::Inactive {
                                    link: &State {
                                        minigame: Some(Minigame::new()),
                                        ..*state
                                    }
                                    .to_link(config),
                                }
                            } else {
                                SwitchItemState::Active
                            },
                        },
                    ];

                    content = content.add(
                        &StyledBox {
                            fill: Some(0x21_4e_7a_ff),
                            padding_left: 10.,
                            padding_right: 10.,
                            padding_top: 5.,
                            padding_bottom: 5.,
                            ..StyledBox::new(BreakList {
                                gap: 8.,
                                content: |content| {
                                    content
                                        // .add(
                                        //     &Column::new(|content| {
                                        //         content
                                        //             .add(&Text {
                                        //                 color: 0xff_ff_ff_ff,
                                        //                 ..Text::new(
                                        //                     localization.settings_language_label,
                                        //                     bold,
                                        //                     12.,
                                        //                 )
                                        //             })?
                                        //             .add(&switch(regular, &language_switch))?;
                                        //         None
                                        //     })
                                        //     .with_gap(2.),
                                        // )?
                                        .add(
                                            &Column::new(|content| {
                                                content
                                                    .add(&Text {
                                                        color: 0xff_ff_ff_ff,
                                                        ..Text::new(
                                                            localization.settings_theme_label,
                                                            bold,
                                                            12.,
                                                        )
                                                    })?
                                                    .add(&switch(regular, &theme_switch))?;

                                                None
                                            })
                                            .with_gap(2.),
                                        )?
                                        .add(
                                            &Column::new(|content| {
                                                content
                                                    .add(&Text {
                                                        color: 0xff_ff_ff_ff,
                                                        ..Text::new(
                                                            localization.settings_minigame_label,
                                                            bold,
                                                            12.,
                                                        )
                                                    })?
                                                    .add(&switch(regular, &minigame_switch))?;

                                                None
                                            })
                                            .with_gap(2.),
                                        )?;

                                    None
                                },
                            })
                        }
                        .with_horizontal_padding(-10.),
                    )?;
                }

                content = content.add(
                    &RichText {
                        align: TextAlign::Center,
                        spans: std::iter::once(Span {
                            color: state.link_color(),
                            underline: true,
                            link: Some(LinkTarget::Uri(config.email_address_mailto)),
                            ..Span::new(config.email_address, regular, 10.5)
                        })
                        .chain(
                            secret_pass
                                .then(|| {
                                    [
                                        Span {
                                            color: state.text_color(),
                                            ..Span::new(" • ", regular, 10.5)
                                        },
                                        Span {
                                            color: state.text_color(),
                                            ..Span::new(config.phone_number, regular, 10.5)
                                        },
                                    ]
                                })
                                .into_iter()
                                .flatten(),
                        )
                        .chain([
                            Span {
                                color: state.text_color(),
                                ..Span::new(" • ", regular, 10.5)
                            },
                            Span {
                                color: state.text_color(),
                                ..Span::new(localization.location, regular, 10.5)
                            },
                            Span {
                                color: state.text_color(),
                                ..Span::new(" • ", regular, 10.5)
                            },
                            Span {
                                color: state.link_color(),
                                underline: true,
                                link: Some(LinkTarget::Uri(
                                    "https://www.linkedin.com/in/florian-plattner-ba40bb175/",
                                )),
                                ..Span::new("LinkedIn", regular, 10.5)
                            },
                            Span {
                                color: state.text_color(),
                                ..Span::new(" • ", regular, 10.5)
                            },
                            Span {
                                color: state.link_color(),
                                underline: true,
                                link: Some(LinkTarget::Uri("https://github.com/lasernoises")),
                                ..Span::new("GitHub", regular, 10.5)
                            },
                        ]),
                    }
                    .with_padding_bottom(2.),
                )?;

                content.add(
                    &Column::new(|mut content| {
                        if let Some(minigame) = state.minigame {
                            content = content.add(&section(
                                state,
                                bold,
                                localization.minigame_title,
                                minigame.draw(config, state, semi_bold, emoji),
                            ))?;
                        }

                        content
                            .add(&section(
                                state,
                                bold,
                                localization.work_experience_title,
                                Column::new(|mut content| {
                                    for work_section in localization.work_sections {
                                        content = content.add(&work_section_element(
                                            state,
                                            regular,
                                            bold,
                                            work_section,
                                        ))?;
                                    }

                                    None
                                })
                                .with_gap(2.),
                            ))?
                            .add(&section(
                                state,
                                bold,
                                localization.technologies_title,
                                Column::new(|mut content| {
                                    for line in localization.technology_lines {
                                        content = content.add(&technology_line(
                                            state,
                                            regular,
                                            bold,
                                            line.title,
                                            line.content,
                                        ))?;
                                    }

                                    None
                                }),
                            ))?
                            .add(&section(
                                state,
                                bold,
                                localization.contributions_title,
                                Column::new(|mut content| {
                                    for line in localization.contributions_lines {
                                        content = content
                                            .add(&open_source_line(state, regular, bold, line))?;
                                    }

                                    None
                                }),
                            ))?
                            .add(&section(
                                state,
                                bold,
                                localization.projects_title,
                                Column::new(|mut content| {
                                    for line in localization.projects_lines {
                                        content = content
                                            .add(&open_source_line(state, regular, bold, line))?;
                                    }

                                    None
                                }),
                            ))?
                            .add(&section(
                                state,
                                bold,
                                localization.blog_title,
                                Column::new(|mut content| {
                                    for line in localization.blog_lines {
                                        content = content
                                            .add(&open_source_line(state, regular, bold, line))?;
                                    }

                                    None
                                }),
                            ))?
                            .add(&section(
                                state,
                                bold,
                                localization.education_title,
                                BreakWhole(Column::new(|content| {
                                    content
                                        .add(
                                            &Row::new(|content| {
                                                content.add(
                                                    &Text {
                                                        color: state.text_color(),
                                                        ..Text::new(
                                                            localization.education.title,
                                                            bold,
                                                            12.,
                                                        )
                                                    },
                                                    Flex::Expand(1),
                                                );
                                                content.add(
                                                    &AlignPreferredHeightBottom(Text {
                                                        color: state.text_color(),
                                                        ..Text::new(
                                                            localization.education.date_range,
                                                            bold,
                                                            11.,
                                                        )
                                                    }),
                                                    Flex::SelfSized,
                                                );
                                            })
                                            .expand(),
                                        )?
                                        .add(&Row::new(|content| {
                                            content.add(
                                                &Text {
                                                    color: state.text_color(),
                                                    ..Text::new(
                                                        localization.education.company,
                                                        regular,
                                                        11.,
                                                    )
                                                },
                                                Flex::Expand(1),
                                            );
                                            content.add(
                                                &Text {
                                                    color: state.text_color(),
                                                    ..Text::new(
                                                        localization.education.location,
                                                        regular,
                                                        11.,
                                                    )
                                                },
                                                Flex::SelfSized,
                                            );
                                        }))?;

                                    None
                                })),
                            ))?;

                        None
                    })
                    .with_gap(5.),
                )?;

                None
            })
            .with_gap(2.),
            border_left: 10.,
            border_right: 10.,
            border_top: 10.,
            border_bottom: 10.,
            decoration_elements: |_content, _page, _page_count| {},
        })
    }
}

#[derive(Copy, Clone)]
struct Config {
    // These are all leaked to make them 'static for convenience. Since this happens once at startup
    // it's not a problem. And it's fun.
    base_url: &'static str,
    easteregg: &'static str,
    email_address: &'static str,
    /// The same as [Self::email_address], but with the "mailto:" prefix. Stored here so we don't
    /// have to concat it each time and make allocations.
    email_address_mailto: &'static str,
    phone_number: &'static str,
    secret: [u8; 8],
}

async fn root(
    extract::State(config): extract::State<Config>,
    RawQuery(query): RawQuery,
) -> Result<
    ([(&'static str, &'static str); 1], Vec<u8>),
    (StatusCode, [(&'static str, &'static str); 1], Vec<u8>),
> {
    let state = if let Some(query) = query {
        let decoded = BASE64_URL_SAFE_NO_PAD
            .decode(query)
            .map_err(|_| error(config, StatusCode::BAD_REQUEST))?;

        State::decode(decoded.as_slice()).map_err(|_| error(config, StatusCode::BAD_REQUEST))?
    } else {
        State {
            open: false,
            theme: Theme::Light,
            minigame: None,
            secret: None,
            reference: false,
            language: Language::English,
        }
    };

    let mut pdf = Pdf::new(Metadata {
        title: "Florian Plattner".to_string(),
        language: "en".to_string(),
        ..Metadata::new()
    });

    let stream_id = pdf.alloc();
    pdf.pdf.stream(stream_id, config.easteregg.as_bytes());

    let regular = Font::new(&mut pdf, REGULAR);
    let bold = Font::new(&mut pdf, BOLD);
    let semi_bold = Font::new(&mut pdf, SEMI_BOLD);
    let emoji = Font::new(&mut pdf, EMOJI);

    pdf.add_element(
        (210., 297.),
        resume(&config, &state, &regular, &bold, &semi_bold, &emoji),
    );

    Ok(([("Content-Type", "application/pdf")], pdf.finish()))
}

async fn not_found(
    extract::State(config): extract::State<Config>,
) -> (StatusCode, [(&'static str, &'static str); 1], Vec<u8>) {
    error(config, StatusCode::NOT_FOUND)
}

fn error(
    config: Config,
    status: StatusCode,
) -> (StatusCode, [(&'static str, &'static str); 1], Vec<u8>) {
    let mut pdf = Pdf::new(Metadata {
        title: status.to_string(),
        language: "en".to_string(),
        ..Metadata::new()
    });

    let stream_id = pdf.alloc();
    pdf.pdf.stream(stream_id, config.easteregg.as_bytes());

    let bold = Font::new(&mut pdf, BOLD);

    pdf.add_element(
        (210., 148.),
        RichText {
            align: TextAlign::Center,
            spans: [
                Span::new(status.as_str(), &bold, 128.),
                Span::new("\n", &bold, 64.),
                Span::new(status.canonical_reason().unwrap(), &bold, 64.),
            ]
            .into_iter(),
        }
        .with_padding_top(16.),
    );

    (status, [("Content-Type", "application/pdf")], pdf.finish())
}

async fn favicon() -> impl IntoResponse {
    (
        [
            ("Content-Type", "image/png"),
            ("Cache-Control", "max-age=3600, public"),
        ],
        LOGO,
    )
}

#[derive(Copy, Clone, PartialEq, Eq, Enumeration)]
enum Theme {
    Light = 0,
    Dark = 1,
}

impl Theme {
    const fn colors(self) -> Colors {
        match self {
            Theme::Light => COLORS_LIGHT,
            Theme::Dark => COLORS_DARK,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Enumeration)]
enum Language {
    English = 0,
    German = 1,
}

#[derive(Message)]
struct State {
    open: bool,
    theme: Theme,
    minigame: Option<Minigame>,
    #[bilrost(encoding = "fixed")]
    secret: Option<[u8; 8]>,
    /// Not used at the moment, but we keep it in there for now for the field offset and in case we
    /// wanna add it back.
    reference: bool,
    language: Language,
}

impl State {
    fn to_link(&self, config: &Config) -> String {
        format!(
            "{}?{}",
            config.base_url,
            BASE64_URL_SAFE_NO_PAD.encode(&self.encode_to_vec()),
        )
    }

    fn background_color(&self) -> Option<u32> {
        self.theme.colors().background
    }

    fn text_color(&self) -> u32 {
        self.theme.colors().text
    }

    fn link_color(&self) -> u32 {
        self.theme.colors().link
    }
    fn disabled_link_color(&self) -> u32 {
        self.theme.colors().disabled_link
    }
}

struct Colors {
    background: Option<u32>,
    text: u32,
    link: u32,
    disabled_link: u32,
}

const COLORS_LIGHT: Colors = Colors {
    background: None,
    text: 0x00_00_00_ff,
    link: 0x11_55_cc_ff,
    disabled_link: 0x77_77_77_ff,
};

const COLORS_DARK: Colors = Colors {
    background: Some(0x1a_20_31_ff),
    text: 0xff_ff_ff_ff,
    link: 0x4c_b5_ff_ff,
    disabled_link: 0x77_77_77_ff,
};

#[tokio::main]
async fn main() {
    let secret = *BASE64_STANDARD
        .decode(load_env("RESUME_SECRET", "abcdefghijk="))
        .expect("Environment variable RESUME_SECRET does not contain valid base64!")
        .as_array()
        .expect("Environment variable RESUME_SECRET has the wrong length!");

    // Not strictly a secret, since it's also in the commit history, but I have the option on where
    // GitHub hides it in the UI and I think it's a good idea to minimize it being scraped.
    let email_address = load_env("RESUME_EMAIL_ADDRESS", "example@example.com");

    let config = Config {
        base_url: load_env("RESUME_BASE_URL", "http://localhost:8080"),
        easteregg: load_env("RESUME_EASTEREGG", ""),
        email_address,
        email_address_mailto: format!("mailto:{email_address}").leak(),
        phone_number: load_env("RESUME_PHONE_NUMBER", "+1 555 0128"),
        secret,
    };

    eprintln!(
        "Link with secret: {}",
        State {
            open: false,
            theme: Theme::Light,
            minigame: None,
            secret: Some(secret),
            reference: false,
            language: Language::English,
        }
        .to_link(&config),
    );

    let app = Router::new()
        .route("/", get(root))
        .route("/favicon.ico", get(favicon))
        .fallback(not_found)
        .with_state(config);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn load_env(var: &str, default: &'static str) -> &'static str {
    match std::env::var(var) {
        Ok(value) => value.leak(),
        Err(std::env::VarError::NotPresent) => default,
        Err(std::env::VarError::NotUnicode(_)) => {
            panic!("Environment variable {var} does not contain valid unicode!")
        }
    }
}
