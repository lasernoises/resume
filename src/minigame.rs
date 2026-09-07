use bilrost::Message;
use laser_pdf::{
    Element, LinkTarget,
    elements::{
        center_in_preferred_height::CenterInPreferredHeight,
        column::Column,
        empty::Empty,
        expand_to_preferred_height::ExpandToPreferredHeight,
        none::NoneElement,
        rich_text::{RichText, Span},
        row::{Flex, Row},
        styled_box::StyledBox,
        text::{Text, TextAlign},
        v_gap::VGap,
    },
    fonts::truetype::TruetypeFont as Font,
};

use crate::{Config, State};

#[derive(Copy, Clone, Message)]
pub struct Minigame {
    #[bilrost(encoding = "fixed")]
    state: u64,
}

enum Direction {
    Left,
    Right,
    Up,
    Down,
}

struct Cheated;

impl Minigame {
    pub fn new() -> Self {
        let mut game = Minigame { state: 0 };

        let value_a = if rand::random_range(0..10) == 0 { 2 } else { 1 };
        let value_b = if rand::random_range(0..10) == 0 { 2 } else { 1 };

        let pos_a = rand::random_range(0..16);
        let pos_b = rand::random_range(0..15);
        let pos_b = if pos_b == pos_a { 15 } else { pos_b };

        for (pos, value) in [(pos_a, value_a), (pos_b, value_b)] {
            let x = pos % 4;
            let y = pos / 4;

            game.set((x, y), value);
        }

        game
    }

    pub fn get(&self, (x, y): (u8, u8)) -> u8 {
        assert!(x < 4);
        assert!(y < 4);

        let pos = (y * 4 + x) * 4;
        ((self.state >> pos) & 15) as u8
    }

    pub fn set(&mut self, (x, y): (u8, u8), value: u8) {
        assert!(x < 4);
        assert!(y < 4);
        // 2048 == 2 ^ 11
        assert!(value < 12);

        let pos = (y * 4 + x) * 4;
        self.state = (self.state & !(15 << pos)) | ((value as u64) << pos);
    }

    pub(crate) fn draw(
        &self,
        config: &Config,
        state: &State,
        regular: &Font,
        emoji: &Font,
    ) -> impl Element {
        let vars = self
            .won()
            .map(|won| {
                let directions = [
                    Direction::Left,
                    Direction::Right,
                    Direction::Up,
                    Direction::Down,
                ]
                .map(|direction| {
                    let mut game = *self;
                    let moved = game.move_(direction);

                    if moved {
                        Some(
                            State {
                                minigame: Some(game),
                                ..*state
                            }
                            .to_link(config),
                        )
                    } else {
                        None
                    }
                });

                let lost = directions.iter().all(Option::is_none).then(|| {
                    State {
                        minigame: Some(Minigame::new()),
                        ..*state
                    }
                    .to_link(config)
                });

                let won = won.then(|| {
                    State {
                        minigame: Some(Minigame::new()),
                        ..*state
                    }
                    .to_link(config)
                });

                (directions, won, lost)
            })
            .map_err(|_| {
                State {
                    minigame: Some(Minigame::new()),
                    ..*state
                }
                .to_link(config)
            });

        Row::new(move |content| {
            let ([left, right, up, down], won, lost) = match &vars {
                Ok(vars) => vars,
                Err(reset_link) => {
                    content.add(
                        &RichText {
                            spans: [
                                Span {
                                    color: state.text_color(),
                                    extra_line_height: 4.,
                                    ..Span::new("Looks like you're trying to cheat!", regular, 24.)
                                },
                                Span {
                                    color: state.text_color(),
                                    ..Span::new(
                                        "\nIf you hire me I might forgive you.\nClick ",
                                        regular,
                                        12.,
                                    )
                                },
                                Span {
                                    color: state.link_color(),
                                    link: Some(LinkTarget::Uri(&reset_link)),
                                    ..Span::new("here", regular, 12.)
                                },
                                Span {
                                    color: state.text_color(),
                                    ..Span::new(" to reset.", regular, 12.)
                                },
                            ]
                            .into_iter(),
                            align: TextAlign::Center,
                        },
                        Flex::Expand(1),
                    );

                    return;
                }
            };

            content.add(
                &StyledBox {
                    fill: Some(0x21_4e_7a_ff),
                    padding_left: 2.,
                    padding_right: 2.,
                    padding_top: 2.,
                    padding_bottom: 2.,
                    border_radius: 3.,
                    ..StyledBox::new(
                        Column::new(|mut content| {
                            for y in 0..4 {
                                content = content.add(
                                    &Row::new(|content| {
                                        content.add(&VGap(16.), Flex::SelfSized);

                                        for x in 0..4 {
                                            let value = self.get((x, y));
                                            if let Some((text, background_color, text_color)) =
                                                TILE_PROPERTIES[value as usize]
                                            {
                                                content.add(
                                                    &StyledBox {
                                                        fill: Some(background_color),
                                                        border_radius: 2.,

                                                        ..StyledBox::new(CenterInPreferredHeight(
                                                            Text {
                                                                color: text_color,
                                                                align: TextAlign::Center,
                                                                ..Text::new(text, regular, 14.)
                                                            },
                                                        ))
                                                    },
                                                    Flex::Expand(1),
                                                );
                                            } else {
                                                content.add(
                                                    &StyledBox {
                                                        fill: Some(0x2d_6d_ab_ff),
                                                        border_radius: 2.,

                                                        ..StyledBox::new(ExpandToPreferredHeight(
                                                            Empty,
                                                        ))
                                                    },
                                                    Flex::Expand(1),
                                                );
                                            }
                                        }
                                    })
                                    .with_gap(2.)
                                    .expand(),
                                )?;
                            }

                            None
                        })
                        .with_gap(2.),
                    )
                },
                Flex::Fixed(74.),
            );

            content.flex_gap(1);

            if let Some(reset_link) = won {
                content.add(
                    &CenterInPreferredHeight(RichText {
                        spans: [
                            Span {
                                color: state.text_color(),
                                extra_line_height: 4.,
                                ..Span::new("You win!", regular, 24.)
                            },
                            Span {
                                color: state.text_color(),
                                ..Span::new("\nWill you hire me now?", regular, 12.)
                            },
                            Span {
                                color: state.text_color(),
                                ..Span::new("\nOr you can ", regular, 12.)
                            },
                            Span {
                                color: state.link_color(),
                                link: Some(LinkTarget::Uri(reset_link)),
                                ..Span::new("reset", regular, 12.)
                            },
                            Span {
                                color: state.text_color(),
                                ..Span::new(".", regular, 12.)
                            },
                        ]
                        .into_iter(),
                        align: TextAlign::Center,
                    }),
                    Flex::Expand(2),
                );
            } else if let Some(reset_link) = lost {
                content.add(
                    &CenterInPreferredHeight(RichText {
                        spans: [
                            Span {
                                color: state.text_color(),
                                extra_line_height: 4.,
                                ..Span::new("You loose!", regular, 24.)
                            },
                            Span {
                                color: state.text_color(),
                                ..Span::new("\nIf you promise to hire me you can ", regular, 12.)
                            },
                            Span {
                                color: state.link_color(),
                                link: Some(LinkTarget::Uri(reset_link)),
                                ..Span::new("try again", regular, 12.)
                            },
                            Span {
                                color: state.text_color(),
                                ..Span::new(".", regular, 12.)
                            },
                        ]
                        .into_iter(),
                        align: TextAlign::Center,
                    }),
                    Flex::Expand(2),
                );
            } else {
                content.add(
                    &CenterInPreferredHeight(
                        Column::new(|mut content| {
                            for y in 0..3 {
                                content = content.add(&Row::new(|content| {
                                    for x in 0..3 {
                                        if (x == 1 && y != 1) || (y == 1 && x != 1) {
                                            let target = match (x, y) {
                                                (1, 0) => &up,
                                                (1, 2) => &down,
                                                (0, 1) => &left,
                                                (2, 1) => &right,
                                                _ => unreachable!(),
                                            };

                                            content.add(
                                                &Text {
                                                    color: if target.is_some() {
                                                        state.link_color()
                                                    } else {
                                                        state.disabled_link_color()
                                                    },
                                                    link: target.as_deref().map(LinkTarget::Uri),
                                                    ..Text::new(
                                                        match (x, y) {
                                                            (1, 0) => "⬆︎",
                                                            (1, 2) => "⬇︎",
                                                            (0, 1) => "⬅︎",
                                                            (2, 1) => "➡︎",
                                                            _ => unreachable!(),
                                                        },
                                                        emoji,
                                                        24.,
                                                    )
                                                },
                                                Flex::Expand(1),
                                            );
                                        } else {
                                            content.add(&NoneElement, Flex::Expand(1));
                                        }
                                    }
                                }))?;
                            }

                            None
                        })
                        .with_gap(4.),
                    ),
                    Flex::Fixed(44.),
                );
            }

            content.flex_gap(1);
        })
        .expand()
    }

    fn move_(&mut self, direction: Direction) -> bool {
        use Direction::*;

        let mut empty_cells = 0;
        let mut moved = false;

        let to_pos = |a: u8, b: u8| {
            let b_pos = match direction {
                Left | Up => b,
                Right | Down => 3 - b,
            };
            match direction {
                Left | Right => (b_pos, a),
                Up | Down => (a, b_pos),
            }
        };

        for a in 0..4 {
            // a is the position on the axis perpendicular to the axis of movement.

            let mut current = 0;
            let mut current_value = self.get(to_pos(a, 0));

            for b in 1..4 {
                // b is the position on the axis of movement, but looking from the direction of
                // movement.

                let pos = to_pos(a, b);
                let value = self.get(pos);

                if value == 0 {
                    continue;
                }

                self.set(pos, 0);

                if value == current_value {
                    moved = true;
                    self.set(to_pos(a, current), value + 1);

                    // to prevent double merging
                    current += 1;
                    current_value = 0;
                } else if current_value == 0 {
                    moved = true;
                    self.set(to_pos(a, current), value);
                    current_value = value;
                } else {
                    current += 1;

                    if current < b {
                        moved = true;
                    }

                    self.set(to_pos(a, current), value);
                    current_value = value;
                }
            }

            if current_value != 0 {
                current += 1;
            }

            empty_cells += 4 - current;
        }

        if moved && empty_cells != 0 {
            'a: {
                let tile_to_set = rand::random_range(0..empty_cells);
                let value = if rand::random_range(0..10) == 0 { 2 } else { 1 };

                let mut current = 0;

                for x in 0..4 {
                    for y in 0..4 {
                        if self.get((x, y)) == 0 {
                            if current == tile_to_set {
                                self.set((x, y), value);
                                break 'a;
                            }

                            current += 1;
                        }
                    }
                }

                unreachable!();
            }
        }

        moved
    }

    fn won(&self) -> Result<bool, Cheated> {
        let mut won = false;

        for x in 0..4 {
            for y in 0..4 {
                let value = self.get((x, y));
                if value == 11 {
                    won = true;
                } else if value > 11 {
                    return Err(Cheated);
                }
            }
        }

        Ok(won)
    }
}

const TILE_PROPERTIES: &[Option<(&str, u32, u32)>] = &[
    None,
    Some(("2", 0xee_ee_dd_ff, 0x00_00_00_ff)),
    Some(("4", 0xdd_dd_cc_ff, 0x00_00_00_ff)),
    Some(("8", 0xff_66_00_ff, 0xff_ff_ff_ff)),
    Some(("16", 0xdd_44_00_ff, 0xff_ff_ff_ff)),
    Some(("32", 0xdd_22_00_ff, 0xff_ff_ff_ff)),
    Some(("64", 0xee_00_00_ff, 0xff_ff_ff_ff)),
    Some(("128", 0xff_dd_00_ff, 0xff_ff_ff_ff)),
    Some(("256", 0xff_cc_00_ff, 0xff_ff_ff_ff)),
    Some(("512", 0xff_cc_00_ff, 0xff_ff_ff_ff)),
    Some(("1024", 0xff_bb_00_ff, 0xff_ff_ff_ff)),
    Some(("2048", 0xff_aa_00_ff, 0xff_ff_ff_ff)),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {
        let game = Minigame {
            state: 0b1111_0000_0000_0000_0000_0000,
        };

        assert_eq!(game.get((1, 1)), 15);
    }

    #[test]
    fn test_set() {
        let mut game = Minigame { state: 0 };
        game.set((1, 1), 15);
        assert_eq!(game.state, 0b1111_0000_0000_0000_0000_0000);
    }
}
