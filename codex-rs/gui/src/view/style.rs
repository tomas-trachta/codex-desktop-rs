use iced::Background;
use iced::Border;
use iced::Color;
use iced::Theme;
use iced::widget::container;

const CORNER_RADIUS: f32 = 10.0;
const BACKDROP_ALPHA: f32 = 0.55;

pub fn sidebar(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(Background::Color(palette.background.weakest.color)),
        ..container::Style::default()
    }
}

pub fn header(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(Background::Color(palette.background.base.color)),
        border: Border {
            color: palette.background.weak.color,
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn user_bubble(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(Background::Color(palette.background.weak.color)),
        text_color: Some(palette.background.weak.text),
        border: Border {
            radius: CORNER_RADIUS.into(),
            ..Border::default()
        },
        ..container::Style::default()
    }
}

pub fn tool_card(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(Background::Color(palette.background.weakest.color)),
        border: Border {
            color: palette.background.weak.color,
            width: 1.0,
            radius: CORNER_RADIUS.into(),
        },
        ..container::Style::default()
    }
}

pub fn dialog(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();

    container::Style {
        background: Some(Background::Color(palette.background.base.color)),
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: CORNER_RADIUS.into(),
        },
        ..container::Style::default()
    }
}

pub fn backdrop(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color {
            a: BACKDROP_ALPHA,
            ..Color::BLACK
        })),
        ..container::Style::default()
    }
}
