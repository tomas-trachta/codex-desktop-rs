use iced::Element;
use iced::Font;
use iced::Length;
use iced::Padding;
use iced::border;
use iced::widget::Column;
use iced::widget::column;
use iced::widget::container;
use iced::widget::markdown;
use iced::widget::markdown::HeadingLevel;
use iced::widget::markdown::Highlight;
use iced::widget::markdown::Settings;
use iced::widget::markdown::Text;
use iced::widget::scrollable;
use iced::widget::text::LineHeight;
use iced::widget::text::Span;
use iced_selection::rich_text as selectable_rich_text;

use super::flat_markdown;
use super::style;
use crate::app::Message;

const TEXT_SIZE: f32 = 15.0;
const CODE_SIZE: f32 = 13.0;
const INLINE_CODE_SIZE: f32 = 13.0;
const WORD_BOUNDARY: char = '\u{200B}';
const INLINE_CODE_PADDING: Padding = Padding {
    top: 1.0,
    right: 3.0,
    bottom: 1.0,
    left: 3.0,
};
const LINE_HEIGHT: f32 = 1.6;
const CODE_PADDING: u16 = 12;

struct Viewer;

pub fn view(content: &markdown::Content) -> Element<'_, Message> {
    let settings = settings();
    let mut blocks = Column::new().spacing(settings.spacing);
    let mut flowing: Vec<&markdown::Item> = Vec::new();

    for item in content.items() {
        if flat_markdown::is_flowable(item) {
            flowing.push(item);
            continue;
        }

        blocks = push_flowing(blocks, settings, &mut flowing);
        blocks = blocks.push(markdown::view_with(
            std::iter::once(item),
            settings,
            &Viewer,
        ));
    }

    push_flowing(blocks, settings, &mut flowing).into()
}

fn push_flowing<'a>(
    blocks: Column<'a, Message>,
    settings: Settings,
    flowing: &mut Vec<&markdown::Item>,
) -> Column<'a, Message> {
    if flowing.is_empty() {
        return blocks;
    }

    let block = flat_markdown::selectable_block(settings, flowing);
    flowing.clear();
    blocks.push(block)
}

fn settings() -> Settings {
    let mut settings = Settings::with_text_size(TEXT_SIZE, markdown_style());
    settings.code_size = CODE_SIZE.into();
    settings
}

fn markdown_style() -> markdown::Style {
    markdown::Style {
        font: style::FONT,
        inline_code_highlight: inline_code_highlight(),
        inline_code_padding: INLINE_CODE_PADDING,
        inline_code_color: style::TEXT,
        inline_code_font: Font::MONOSPACE,
        code_block_font: Font::MONOSPACE,
        link_color: style::ACCENT,
    }
}

impl<'a> markdown::Viewer<'a, Message> for Viewer {
    fn on_link_click(url: markdown::Uri) -> Message {
        Message::LinkClicked(url)
    }

    fn heading(
        &self,
        settings: Settings,
        level: &'a HeadingLevel,
        text: &'a Text,
        index: usize,
    ) -> Element<'a, Message> {
        let size = match level {
            HeadingLevel::H1 => settings.h1_size,
            HeadingLevel::H2 => settings.h2_size,
            HeadingLevel::H3 => settings.h3_size,
            HeadingLevel::H4 => settings.h4_size,
            HeadingLevel::H5 => settings.h5_size,
            HeadingLevel::H6 => settings.h6_size,
        };
        let top = if index > 0 {
            settings.text_size.0 / 2.0
        } else {
            0.0
        };

        container(
            selectable_rich_text(text.spans(settings.style))
                .size(size)
                .font(style::SEMIBOLD)
                .on_link_click(Message::LinkClicked)
                .style(style::selectable(style::TEXT)),
        )
        .padding(iced::Padding::ZERO.top(top))
        .into()
    }

    fn paragraph(&self, settings: Settings, text: &Text) -> Element<'a, Message> {
        let spans: Vec<Span<'static, markdown::Uri>> = text
            .spans(settings.style)
            .iter()
            .cloned()
            .map(shrink_inline_code)
            .collect();

        selectable_rich_text(spans)
            .size(settings.text_size)
            .line_height(LineHeight::Relative(LINE_HEIGHT))
            .font(settings.style.font)
            .on_link_click(Message::LinkClicked)
            .style(style::selectable(style::TEXT))
            .into()
    }

    fn code_block(
        &self,
        settings: Settings,
        _language: Option<&'a str>,
        _code: &'a str,
        lines: &'a [Text],
    ) -> Element<'a, Message> {
        let code = column(lines.iter().map(|line| code_line(settings, line)));

        container(scrollable(container(code).padding(CODE_PADDING)).direction(
            scrollable::Direction::Horizontal(
                scrollable::Scrollbar::default().width(4).scroller_width(4),
            ),
        ))
        .width(Length::Fill)
        .style(style::code_block)
        .into()
    }
}

pub fn shrink_inline_code(mut span: Span<'static, markdown::Uri>) -> Span<'static, markdown::Uri> {
    if span.link.as_deref().is_some_and(is_file_reference) {
        span.highlight = Some(inline_code_highlight());
        span.font = Some(Font::MONOSPACE);
        span.color = Some(style::TEXT);
        span.padding = INLINE_CODE_PADDING;
    }

    if span.highlight.is_some() {
        span.size = Some(INLINE_CODE_SIZE.into());
        span.text = format!("{WORD_BOUNDARY}{}{WORD_BOUNDARY}", span.text).into();
    }
    span
}

fn is_file_reference(uri: &str) -> bool {
    !uri.starts_with("http://") && !uri.starts_with("https://") && !uri.starts_with("mailto:")
}

fn inline_code_highlight() -> Highlight {
    Highlight {
        background: style::CODE_BACKGROUND.into(),
        border: border::rounded(4),
    }
}

fn code_line<'a>(settings: Settings, line: &Text) -> Element<'a, Message> {
    selectable_rich_text(line.spans(settings.style))
        .font(settings.style.code_block_font)
        .size(settings.code_size)
        .on_link_click(Message::LinkClicked)
        .style(style::selectable(style::TEXT))
        .into()
}
