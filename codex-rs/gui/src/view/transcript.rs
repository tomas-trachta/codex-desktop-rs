use codex_app_server_protocol::CommandExecutionStatus;
use codex_app_server_protocol::FileUpdateChange;
use codex_app_server_protocol::PatchApplyStatus;
use codex_app_server_protocol::PatchChangeKind;
use iced::Element;
use iced::Font;
use iced::Length;
use iced::Theme;
use iced::widget::column;
use iced::widget::container;
use iced::widget::markdown;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::space;
use iced::widget::text;

use super::style;
use crate::app::Message;
use crate::transcript::CommandEntry;
use crate::transcript::Entry;
use crate::transcript::FileChangeEntry;
use crate::transcript::Transcript;

const CONTENT_WIDTH: f32 = 860.0;
const BUBBLE_WIDTH: f32 = 620.0;
const OUTPUT_TAIL_LINES: usize = 14;

pub fn view<'a>(transcript: &'a Transcript, theme: &'a Theme) -> Element<'a, Message> {
    let entries = column(
        transcript
            .entries()
            .iter()
            .filter_map(|entry| entry_view(entry, theme)),
    )
    .spacing(16)
    .max_width(CONTENT_WIDTH);

    scrollable(container(entries).center_x(Length::Fill).padding([24, 32]))
        .anchor_bottom()
        .height(Length::Fill)
        .into()
}

fn entry_view<'a>(entry: &'a Entry, theme: &'a Theme) -> Option<Element<'a, Message>> {
    match entry {
        Entry::User(message) => Some(user_message(message)),
        Entry::Agent(content) => Some(agent_message(content, theme)),
        Entry::Reasoning(summary) => reasoning(summary),
        Entry::Command(command) => Some(command_card(command)),
        Entry::FileChange(change) => Some(file_change_card(change)),
        Entry::ToolCall(name) => Some(tool_call(name)),
        Entry::Notice(message) => Some(notice(message)),
    }
}

fn user_message(message: &str) -> Element<'_, Message> {
    let bubble = container(text(message).size(14))
        .padding([10, 14])
        .max_width(BUBBLE_WIDTH)
        .style(style::user_bubble);

    row![space::horizontal(), bubble].into()
}

fn agent_message<'a>(content: &'a markdown::Content, theme: &'a Theme) -> Element<'a, Message> {
    markdown::view(content.items(), theme).map(Message::LinkClicked)
}

fn reasoning(summary: &str) -> Option<Element<'_, Message>> {
    let summary = summary.trim();
    if summary.is_empty() {
        return None;
    }

    Some(text(summary).size(13).style(text::secondary).into())
}

fn command_card(command: &CommandEntry) -> Element<'_, Message> {
    let heading = row![
        text(format!("$ {}", command.command))
            .font(Font::MONOSPACE)
            .size(13)
            .width(Length::Fill),
        text(command_status(command))
            .size(12)
            .style(text::secondary),
    ]
    .spacing(12);

    let mut card = column![heading].spacing(8);
    let output = output_tail(&command.output);
    if !output.is_empty() {
        card = card.push(
            text(output)
                .font(Font::MONOSPACE)
                .size(12)
                .style(text::secondary),
        );
    }

    container(card)
        .padding(12)
        .width(Length::Fill)
        .style(style::tool_card)
        .into()
}

fn file_change_card(change: &FileChangeEntry) -> Element<'_, Message> {
    let heading = row![
        text("File changes").size(13).width(Length::Fill),
        text(patch_status(&change.status))
            .size(12)
            .style(text::secondary),
    ];

    let files = column(change.changes.iter().map(changed_file)).spacing(4);

    container(column![heading, files].spacing(8))
        .padding(12)
        .width(Length::Fill)
        .style(style::tool_card)
        .into()
}

pub fn changed_file(change: &FileUpdateChange) -> Element<'_, Message> {
    text(format!(
        "{}  {}",
        change_kind_label(&change.kind),
        change.path
    ))
    .font(Font::MONOSPACE)
    .size(12)
    .into()
}

fn tool_call(name: &str) -> Element<'_, Message> {
    container(text(format!("Tool call: {name}")).size(13))
        .padding(12)
        .width(Length::Fill)
        .style(style::tool_card)
        .into()
}

fn notice(message: &str) -> Element<'_, Message> {
    container(text(message).size(13))
        .padding([8, 12])
        .width(Length::Fill)
        .style(container::danger)
        .into()
}

fn command_status(command: &CommandEntry) -> String {
    match (&command.status, command.exit_code) {
        (CommandExecutionStatus::InProgress, _) => "running".to_string(),
        (CommandExecutionStatus::Completed, Some(code)) => format!("exit {code}"),
        (CommandExecutionStatus::Completed, None) => "done".to_string(),
        (CommandExecutionStatus::Failed, _) => "failed".to_string(),
        (CommandExecutionStatus::Declined, _) => "declined".to_string(),
    }
}

fn patch_status(status: &PatchApplyStatus) -> &'static str {
    match status {
        PatchApplyStatus::InProgress => "applying",
        PatchApplyStatus::Completed => "applied",
        PatchApplyStatus::Failed => "failed",
        PatchApplyStatus::Declined => "declined",
    }
}

fn change_kind_label(kind: &PatchChangeKind) -> &'static str {
    match kind {
        PatchChangeKind::Add => "A",
        PatchChangeKind::Delete => "D",
        PatchChangeKind::Update { .. } => "M",
    }
}

fn output_tail(output: &str) -> String {
    let lines: Vec<&str> = output.trim_end().lines().collect();
    let start = lines.len().saturating_sub(OUTPUT_TAIL_LINES);

    lines[start..].join("\n")
}
