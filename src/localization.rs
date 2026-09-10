pub struct Localization {
    pub location: &'static str,
    pub settings_theme_label: &'static str,
    pub settings_theme_light: &'static str,
    pub settings_theme_dark: &'static str,
    pub settings_minigame_label: &'static str,
    pub settings_minigame_off: &'static str,
    pub settings_minigame_on: &'static str,
    pub minigame_title: &'static str,
    pub work_experience_title: &'static str,
    pub work_sections: &'static [WorkSection],
    pub technologies_title: &'static str,
    pub technology_lines: &'static [TechnologyLine],
    pub contributions_title: &'static str,
    pub contributions_lines: &'static [&'static [Span]],
    pub projects_title: &'static str,
    pub projects_lines: &'static [&'static [Span]],
    pub blog_title: &'static str,
    pub blog_lines: &'static [&'static [Span]],
    pub education_title: &'static str,
    pub education: Education,
}

pub struct WorkSection {
    pub title: &'static str,
    pub date_range: &'static str,
    pub company: &'static str,
    pub location: &'static str,
    pub projects: &'static [&'static [Span]],
}

pub struct TechnologyLine {
    pub title: &'static str,
    pub content: &'static str,
}

pub struct Education {
    pub title: &'static str,
    pub date_range: &'static str,
    pub company: &'static str,
    pub location: &'static str,
}

pub struct Span {
    pub text: &'static str,
    pub bold: bool,
    pub link: Option<&'static str>,
}

impl Span {
    const fn regular(text: &'static str) -> Self {
        Span {
            text,
            bold: false,
            link: None,
        }
    }

    const fn bold(text: &'static str) -> Self {
        Span {
            text,
            bold: true,
            link: None,
        }
    }

    const fn link_bold(text: &'static str, target: &'static str) -> Self {
        Span {
            text,
            bold: true,
            link: Some(target),
        }
    }
}

pub static EN: Localization = Localization {
    location: "Zürich, Switzerland",
    settings_theme_label: "Theme",
    settings_theme_light: "Light",
    settings_theme_dark: "Dark",
    settings_minigame_label: "Minigame",
    settings_minigame_off: "Off",
    settings_minigame_on: "On",
    minigame_title: "MINIGAME",
    work_experience_title: "WORK EXPERIENCE",
    work_sections: &[
        WorkSection {
            title: "Lead Developer",
            date_range: "2021 - January 2026",
            company: "Escola GmbH",
            location: "Zürich, Switzerland",
            projects: &[
                &[Span::regular("Lead a team growing to 11 engineers")],
                &[
                    Span::bold("Code review: "),
                    Span::regular(
                        "Introduced pull requests and code review, with 10’000+ PRs merged to \
                        date.",
                    ),
                ],
                &[
                    Span::link_bold("Laser-PDF", "https://github.com/laser-pdf/laser-pdf"),
                    Span::bold(": "),
                    Span::regular(
                        "Built an open source PDF generation library in Rust. Including precise \
                        layout using 28 different layout elements, rich text and font-fallback. \
                        Used for 300+ different PDF exports. Renders 1’000+ pages per second.",
                    ),
                ],
                &[
                    Span::bold("Security improvements: "),
                    Span::regular(
                        "Made various security improvements including systematically solving XSS \
                        and SQL injection vulnerabilities.",
                    ),
                ],
            ],
        },
        WorkSection {
            title: "Software Engineer",
            date_range: "2019-2021",
            company: "Escola GmbH",
            location: "Zürich, Switzerland",
            projects: &[
                &[
                    Span::bold("Forms engine: "),
                    Span::regular(
                        "Created a flexible system for creating custom forms for customer data \
                        entry. Has been used to create 500+ different forms.",
                    ),
                ],
                &[
                    Span::bold("Mobile app: "),
                    Span::regular(
                        "Built a mobile app used by 200’000+ users including a data-sync protocol \
                        and webviews.",
                    ),
                ],
            ],
        },
        WorkSection {
            title: "Software Development Internship",
            date_range: "2019",
            company: "Sitrox AG",
            location: "Zürich, Switzerland",
            projects: &[&[
                Span::bold("Ruby on Rails development: "),
                Span::regular("Worked on internal and customer facing Ruby on Rails projects."),
            ]],
        },
    ],
    technologies_title: "TECHNOLOGIES AND LANGUAGES",
    technology_lines: &[
        TechnologyLine {
            title: "Languages:",
            content: "Rust, TypeScript, JavaScript, PHP, Dart, C/C++, Python",
        },
        TechnologyLine {
            title: "Technologies:",
            content: "Git, Vue.js, Tailwind CSS, Flutter, MariaDB, Linux, Docker, Nix, GDB",
        },
        // TechnologyLine {
        //     title: "Other:",
        //     content: "Text Rendering, Debugging, Designing Abstractions",
        // },
    ],
    contributions_title: "OPEN SOURCE CONTRIBUTIONS",
    contributions_lines: &[
        &[
            Span::link_bold(
                "slint-ui/slint#12568",
                "https://github.com/slint-ui/slint/pull/12568",
            ),
            Span::bold(": "),
            Span::regular("Fixed a compiler bug in the Slint UI language."),
        ],
        &[
            Span::link_bold(
                "mitsuhiko/insta#610",
                "https://github.com/mitsuhiko/insta/pull/610",
            ),
            Span::bold(": "),
            Span::regular("Added binary snapshot support to the Insta snapshot testing library."),
        ],
        &[
            Span::link_bold(
                "php/php-src#20818",
                "https://github.com/php/php-src/issues/20818",
            ),
            Span::bold(": "),
            Span::regular("Found and analyzed a bug in the PHP tracing JIT compiler."),
        ],
    ],
    projects_title: "PERSONAL PROJECTS",
    projects_lines: &[
        &[
            Span::link_bold("fluorine", "https://github.com/lasernoises/fluorine"),
            Span::bold(": "),
            Span::regular("A Rust reactivity library with a novel design."),
        ],
        &[
            Span::link_bold("spicetime", "https://github.com/lasernoises/spicetime"),
            Span::bold(": "),
            Span::regular("A minimal generational cell type for Rust."),
        ],
        &[
            Span::link_bold("treeq", "https://github.com/lasernoises/treeq"),
            Span::bold(": "),
            Span::regular("A code rewriting tool using JQ syntax and Tree-sitter syntax trees."),
        ],
        &[
            Span::link_bold(
                "is-this-a-lisp",
                "https://github.com/lasernoises/is-this-a-lisp",
            ),
            Span::bold(": "),
            Span::regular("A Lisp implementation created from first principles."),
        ],
        &[
            Span::link_bold(
                "hacky-rubiks-solver",
                "https://github.com/lasernoises/hacky-rubiks-solver",
            ),
            Span::bold(": "),
            Span::regular("Using a property testing library and a fuzzer to solve a Rubik's cube."),
        ],
        &[
            Span::link_bold(
                "Various experiments",
                "https://lasernoises.com/blog/gui-experiments-links/",
            ),
            Span::regular(
                " around building GUI libraries in Rust, focusing on state management, data-flow, \
                layout and reactivity.",
            ),
        ],
    ],
    blog_title: "BLOG",
    blog_lines: &[
        &[Span::link_bold(
            "Creating a VGA Signal in Hubris",
            "https://lasernoises.com/blog/hubris-vga/",
        )],
        &[Span::link_bold(
            "A Shared Syntax for Custom Languages",
            "https://lasernoises.com/blog/universal-syntax/",
        )],
    ],
    education_title: "EDUCATION",
    education: Education {
        title: "Informatiker, Fachrichtung Applikationsentwicklung EFZ",
        date_range: "2015–2019",
        company: "Rafisa Informatik GmbH",
        location: "Dietikon, Switzerland",
    },
};
