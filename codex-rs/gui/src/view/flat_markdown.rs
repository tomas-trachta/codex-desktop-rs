use iced::Element;
use iced::Font;
use iced::widget::markdown::Bullet;
use iced::widget::markdown::HeadingLevel;
use iced::widget::markdown::Item;
use iced::widget::markdown::Settings;
use iced::widget::markdown::Text;
use iced::widget::markdown::Uri;
use iced::widget::text::LineHeight;
use iced::widget::text::Span;
use iced_selection::rich_text as selectable_rich_text;

use super::style;
use crate::app::Message;

type OwnedSpan = Span<'static, Uri>;

const HEADING_GROWTH: f32 = 3.0;
const LIST_INDENT: &str = "    ";
const LINE_HEIGHT: f32 = 1.6;

pub struct Flattener {
    settings: Settings,
    spans: Vec<OwnedSpan>,
}

pub fn is_flowable(item: &Item) -> bool {
    !matches!(item, Item::Table { .. } | Item::Image { .. })
}

pub fn selectable_block<'a>(settings: Settings, items: &[&Item]) -> Element<'a, Message> {
    let mut flattener = Flattener {
        settings,
        spans: Vec::new(),
    };
    for item in items {
        flattener.push_item(item, 0);
    }

    selectable_rich_text(flattener.finish())
        .size(settings.text_size)
        .line_height(LineHeight::Relative(LINE_HEIGHT))
        .font(style::FONT)
        .on_link_click(Message::LinkClicked)
        .style(style::selectable(style::TEXT))
        .into()
}

impl Flattener {
    fn push_item(&mut self, item: &Item, depth: usize) {
        match item {
            Item::Heading(level, text) => self.push_heading(level, text),
            Item::Paragraph(text) => self.push_paragraph(text),
            Item::CodeBlock { code, .. } => self.push_code(code),
            Item::List { start, bullets } => self.push_list(*start, bullets, depth),
            Item::Quote(items) => self.push_quote(items, depth),
            Item::Rule => self.push_rule(),
            Item::Table { .. } | Item::Image { .. } => {}
        }
    }

    fn push_heading(&mut self, level: &HeadingLevel, text: &Text) {
        let size = heading_size(&self.settings, level);
        let spans = self
            .styled(text)
            .into_iter()
            .map(|span| span.size(size).font(style::SEMIBOLD));

        self.spans.extend(spans);
        self.end_block();
    }

    fn push_paragraph(&mut self, text: &Text) {
        let spans = self.styled(text);
        self.spans.extend(spans);
        self.end_block();
    }

    fn push_code(&mut self, code: &str) {
        for line in code.trim_end_matches('\n').lines() {
            self.spans.push(code_line(line, &self.settings));
            self.spans.push(Span::new("\n"));
        }
        self.gap();
    }

    fn push_list(&mut self, start: Option<u64>, bullets: &[Bullet], depth: usize) {
        for (index, bullet) in bullets.iter().enumerate() {
            let marker = list_marker(start, index, bullet);
            self.spans.push(Span::new(format!(
                "{}{marker}  ",
                LIST_INDENT.repeat(depth + 1)
            )));
            self.push_bullet_items(bullet_items(bullet), depth);
        }
        self.gap();
    }

    fn push_bullet_items(&mut self, items: &[Item], depth: usize) {
        for (index, item) in items.iter().enumerate() {
            match item {
                Item::Paragraph(text) if index == 0 => {
                    let spans = self.styled(text);
                    self.spans.extend(spans);
                    self.spans.push(Span::new("\n"));
                }
                Item::List { start, bullets } => self.push_list(*start, bullets, depth + 1),
                other => self.push_item(other, depth + 1),
            }
        }
    }

    fn push_quote(&mut self, items: &[Item], depth: usize) {
        let start = self.spans.len();
        for item in items {
            self.push_item(item, depth);
        }

        for span in &mut self.spans[start..] {
            span.color = Some(style::TEXT_MUTED);
        }
    }

    fn push_rule(&mut self) {
        self.spans
            .push(Span::new("────────────").color(style::BORDER));
        self.end_block();
    }

    fn styled(&self, text: &Text) -> Vec<OwnedSpan> {
        text.spans(self.settings.style)
            .iter()
            .cloned()
            .map(super::markdown::shrink_inline_code)
            .collect()
    }

    fn end_block(&mut self) {
        self.spans.push(Span::new("\n"));
        self.gap();
    }

    fn gap(&mut self) {
        self.spans.push(Span::new("\n"));
    }

    fn finish(mut self) -> Vec<OwnedSpan> {
        while self
            .spans
            .last()
            .is_some_and(|span| span.text.trim().is_empty())
        {
            self.spans.pop();
        }

        let line_height = LineHeight::Absolute((self.settings.text_size.0 * LINE_HEIGHT).into());
        self.spans
            .into_iter()
            .map(|span| span.line_height(line_height))
            .collect()
    }
}

fn heading_size(settings: &Settings, level: &HeadingLevel) -> f32 {
    let growth = match level {
        HeadingLevel::H1 | HeadingLevel::H2 => HEADING_GROWTH,
        HeadingLevel::H3 => HEADING_GROWTH / 2.0,
        HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6 => 0.0,
    };
    settings.text_size.0 + growth
}

fn code_line(line: &str, settings: &Settings) -> OwnedSpan {
    Span::new(format!(" {} ", line.replace('\t', LIST_INDENT)))
        .font(Font::MONOSPACE)
        .size(settings.code_size)
        .color(style::TEXT)
        .background(style::CODE_BACKGROUND)
}

fn list_marker(start: Option<u64>, index: usize, bullet: &Bullet) -> String {
    match (bullet, start) {
        (Bullet::Task { done: true, .. }, _) => "☑".to_string(),
        (Bullet::Task { done: false, .. }, _) => "☐".to_string(),
        (Bullet::Point { .. }, Some(first)) => format!("{}.", first + index as u64),
        (Bullet::Point { .. }, None) => "•".to_string(),
    }
}

fn bullet_items(bullet: &Bullet) -> &[Item] {
    match bullet {
        Bullet::Point { items } | Bullet::Task { items, .. } => items,
    }
}
