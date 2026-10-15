use iced::Background;
use iced::Border;
use iced::Color;
use iced::Font;
use iced::Shadow;
use iced::Theme;
use iced::Vector;
use iced::color;
use iced::font::Weight;
use iced::theme::Palette;
use iced::widget::button;
use iced::widget::container;
use iced::widget::rule;
use iced::widget::scrollable;
use iced::widget::svg;
use iced::widget::text_editor;
use iced::widget::text_input;

pub const SURFACE: Color = Color::WHITE;
pub const SIDEBAR: Color = color!(0xf9f9f9);
pub const BORDER: Color = color!(0xe8e8e8);
pub const HOVER: Color = color!(0xf0f0f0);
pub const SELECTED: Color = color!(0xebebeb);
pub const TEXT: Color = color!(0x1f1f1f);
pub const TEXT_MUTED: Color = color!(0x6e6e6e);
pub const TEXT_FAINT: Color = color!(0xa0a0a0);
pub const CODE_BACKGROUND: Color = color!(0xf3f3f3);
pub const ACCENT: Color = color!(0x2f7cf6);
pub const ACCENT_HOVER: Color = color!(0x1f6ae0);
pub const DANGER: Color = color!(0xc4321f);
pub const DANGER_SURFACE: Color = color!(0xfdeeec);
pub const WARNING: Color = color!(0x9a6700);

const SMALL_RADIUS: f32 = 8.0;
const CARD_RADIUS: f32 = 12.0;
const BUBBLE_RADIUS: f32 = 18.0;
const COMPOSER_RADIUS: f32 = 20.0;

#[cfg(target_os = "windows")]
pub const FONT: Font = Font::with_name("Segoe UI");
#[cfg(not(target_os = "windows"))]
pub const FONT: Font = Font::DEFAULT;

pub const SEMIBOLD: Font = Font {
    weight: Weight::Semibold,
    ..FONT
};

pub fn thin_scrollbar() -> scrollable::Direction {
    scrollable::Direction::Vertical(
        scrollable::Scrollbar::new()
            .width(6)
            .scroller_width(6)
            .margin(2),
    )
}

pub fn theme() -> Theme {
    Theme::custom(
        "Codex Light",
        Palette {
            background: SURFACE,
            text: TEXT,
            primary: ACCENT,
            success: color!(0x1a7f37),
            warning: WARNING,
            danger: DANGER,
        },
    )
}

pub const SELECTION: Color = Color::from_rgba(0.18, 0.49, 0.96, 0.25);

pub fn selectable(color: Color) -> impl Fn(&Theme) -> iced_selection::text::Style {
    move |_theme| iced_selection::text::Style {
        color: Some(color),
        selection: SELECTION,
    }
}

pub fn sidebar(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SIDEBAR)),
        ..container::Style::default()
    }
}

pub fn surface(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SURFACE)),
        text_color: Some(TEXT),
        ..container::Style::default()
    }
}

pub fn divider(_theme: &Theme) -> rule::Style {
    rule::Style {
        color: BORDER,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    }
}

pub fn running_dot(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(ACCENT)),
        border: rounded(f32::MAX),
        ..container::Style::default()
    }
}

pub fn step_header(_theme: &Theme, status: button::Status) -> button::Style {
    let text_color = match status {
        button::Status::Hovered | button::Status::Pressed => TEXT,
        _ => TEXT_MUTED,
    };

    button::Style {
        background: None,
        text_color,
        border: rounded(SMALL_RADIUS),
        ..button::Style::default()
    }
}

pub fn diff_line(kind: DiffLine) -> impl Fn(&Theme) -> container::Style {
    move |_theme| {
        let background = match kind {
            DiffLine::Added => Some(color!(0xe6f6ea)),
            DiffLine::Removed => Some(color!(0xfdecec)),
            DiffLine::Context => None,
        };

        container::Style {
            background: background.map(Background::Color),
            text_color: Some(TEXT),
            ..container::Style::default()
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DiffLine {
    Added,
    Removed,
    Context,
}

pub fn user_bubble(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(CODE_BACKGROUND)),
        text_color: Some(TEXT),
        border: rounded(BUBBLE_RADIUS),
        ..container::Style::default()
    }
}

pub fn tool_card(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SIDEBAR)),
        text_color: Some(TEXT),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: CARD_RADIUS.into(),
        },
        ..container::Style::default()
    }
}

pub fn code_block(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(CODE_BACKGROUND)),
        text_color: Some(TEXT),
        border: rounded(SMALL_RADIUS),
        ..container::Style::default()
    }
}

pub fn notice(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(DANGER_SURFACE)),
        text_color: Some(DANGER),
        border: rounded(SMALL_RADIUS),
        ..container::Style::default()
    }
}

pub fn composer(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SURFACE)),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: COMPOSER_RADIUS.into(),
        },
        shadow: soft_shadow(),
        ..container::Style::default()
    }
}

pub fn dialog(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SURFACE)),
        text_color: Some(TEXT),
        border: Border {
            color: BORDER,
            width: 1.0,
            radius: CARD_RADIUS.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.18),
            offset: Vector::new(0.0, 12.0),
            blur_radius: 40.0,
        },
        ..container::Style::default()
    }
}

pub fn inline_input(_theme: &Theme, status: text_input::Status) -> text_input::Style {
    let border_color = match status {
        text_input::Status::Focused { .. } => ACCENT,
        _ => BORDER,
    };

    text_input::Style {
        background: Background::Color(SURFACE),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: SMALL_RADIUS.into(),
        },
        icon: TEXT_MUTED,
        placeholder: TEXT_FAINT,
        value: TEXT,
        selection: Color::from_rgba(0.18, 0.49, 0.96, 0.25),
    }
}

pub fn tooltip(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(TEXT)),
        text_color: Some(Color::WHITE),
        border: rounded(6.0),
        ..container::Style::default()
    }
}

pub fn composer_editor(_theme: &Theme, _status: text_editor::Status) -> text_editor::Style {
    text_editor::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border::default(),
        placeholder: TEXT_FAINT,
        value: TEXT,
        selection: Color::from_rgba(0.18, 0.49, 0.96, 0.25),
    }
}

pub fn row_button(is_selected: bool) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_theme, status| {
        let background = match (is_selected, status) {
            (true, _) => Some(SELECTED),
            (false, button::Status::Hovered | button::Status::Pressed) => Some(HOVER),
            (false, _) => None,
        };

        button::Style {
            background: background.map(Background::Color),
            text_color: TEXT,
            border: rounded(SMALL_RADIUS),
            ..button::Style::default()
        }
    }
}

pub fn icon_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => Some(Background::Color(HOVER)),
        _ => None,
    };

    button::Style {
        background,
        text_color: TEXT_MUTED,
        border: rounded(SMALL_RADIUS),
        ..button::Style::default()
    }
}

pub fn send_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => ACCENT_HOVER,
        button::Status::Disabled => color!(0xc9dcfb),
        button::Status::Active => ACCENT,
    };

    circle_button(background)
}

pub fn stop_button(_theme: &Theme, _status: button::Status) -> button::Style {
    circle_button(TEXT)
}

pub fn primary_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => ACCENT_HOVER,
        _ => ACCENT,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: Color::WHITE,
        border: rounded(SMALL_RADIUS),
        ..button::Style::default()
    }
}

pub fn secondary_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => SELECTED,
        _ => HOVER,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TEXT,
        border: rounded(SMALL_RADIUS),
        ..button::Style::default()
    }
}

pub fn tinted(color: Color) -> impl Fn(&Theme, svg::Status) -> svg::Style {
    move |_theme, _status| svg::Style { color: Some(color) }
}

fn circle_button(background: Color) -> button::Style {
    button::Style {
        background: Some(Background::Color(background)),
        text_color: Color::WHITE,
        border: rounded(f32::MAX),
        ..button::Style::default()
    }
}

fn rounded(radius: f32) -> Border {
    Border {
        radius: radius.into(),
        ..Border::default()
    }
}

fn soft_shadow() -> Shadow {
    Shadow {
        color: Color::from_rgba(0.0, 0.0, 0.0, 0.06),
        offset: Vector::new(0.0, 4.0),
        blur_radius: 16.0,
    }
}
