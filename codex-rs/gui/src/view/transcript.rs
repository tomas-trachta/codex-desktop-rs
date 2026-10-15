use codex_app_server_protocol::CommandAction;
use codex_app_server_protocol::CommandExecutionStatus;
use codex_app_server_protocol::FileUpdateChange;
use codex_app_server_protocol::PatchApplyStatus;
use codex_app_server_protocol::PatchChangeKind;
use iced::Alignment;
use iced::Color;
use iced::Element;
use iced::Font;
use iced::Length;
use iced::widget::Column;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::row;
use iced::widget::rule;
use iced::widget::scrollable;
use iced::widget::space;
use iced::widget::text;
use iced::widget::text::LineHeight;
use iced::widget::text::Wrapping;
use iced_selection::text as selectable_text;

use super::command_display::display_command;
use super::icon;
use super::icon::Icon;
use super::markdown;
use super::style;
use super::style::DiffLine;
use crate::app::ActiveThread;
use crate::app::Message;
use crate::transcript::AgentEntry;
use crate::transcript::CommandEntry;
use crate::transcript::Entry;
use crate::transcript::FileChangeEntry;
use crate::transcript::Row;

pub const CONTENT_WIDTH: f32 = 740.0;
const BUBBLE_WIDTH: f32 = 560.0;
const OUTPUT_TAIL_LINES: usize = 40;
const DIFF_LINE_LIMIT: usize = 400;
const STEP_LABEL_CHARACTERS: usize = 90;
const BODY_LINE_HEIGHT: f32 = 1.6;
const TAB_SPACES: &str = "    ";
const ADDED: Color = iced::color!(0x1a7f37);

pub fn view(thread: &ActiveThread) -> Element<'_, Message> {
    let mut rows = Column::with_children(
        thread
            .transcript
            .rows()
            .iter()
            .enumerate()
            .filter_map(|(index, row)| row_view(row, thread, ends_turn(thread, index))),
    )
    .spacing(14)
    .max_width(CONTENT_WIDTH);

    if thread.running_turn.is_some() {
        rows = rows.push(thinking());
    }

    scrollable(container(rows).center_x(Length::Fill).padding([28, 32]))
        .direction(style::thin_scrollbar())
        .anchor_bottom()
        .height(Length::Fill)
        .into()
}

fn row_view<'a>(
    row: &'a Row,
    thread: &'a ActiveThread,
    is_final_answer: bool,
) -> Option<Element<'a, Message>> {
    let expanded = thread.expanded_steps.contains(&row.key);

    match &row.entry {
        Entry::User(message) => Some(user_message(message)),
        Entry::Agent(agent) => Some(agent_message(agent, is_final_answer)),
        Entry::Reasoning(summary) => reasoning(summary),
        Entry::Command(command) => Some(command_step(&row.key, command, expanded)),
        Entry::FileChange(change) => Some(file_change_step(&row.key, change, expanded)),
        Entry::ToolCall(name) => Some(tool_step(name)),
        Entry::Notice(message) => Some(notice(message)),
        Entry::TurnSummary { duration_ms } => Some(turn_summary(*duration_ms)),
    }
}

fn user_message(message: &str) -> Element<'_, Message> {
    let bubble = container(
        selectable_text(message)
            .size(15)
            .line_height(LineHeight::Relative(BODY_LINE_HEIGHT))
            .style(style::selectable(style::TEXT)),
    )
    .padding([8, 16])
    .max_width(BUBBLE_WIDTH)
    .style(style::user_bubble);

    container(row![space::horizontal(), bubble])
        .padding(iced::Padding::ZERO.top(10))
        .into()
}

fn ends_turn(thread: &ActiveThread, index: usize) -> bool {
    match thread.transcript.rows().get(index + 1) {
        Some(next) => matches!(next.entry, Entry::TurnSummary { .. } | Entry::User(_)),
        None => thread.running_turn.is_none(),
    }
}

fn agent_message(agent: &AgentEntry, is_final_answer: bool) -> Element<'_, Message> {
    let message = markdown::view(&agent.content);
    if !is_final_answer {
        return message;
    }

    column![message, message_actions(&agent.text)]
        .spacing(8)
        .into()
}

fn message_actions(text: &str) -> Element<'_, Message> {
    let copy = button(icon::icon(Icon::Copy, 15.0, style::TEXT_MUTED))
        .padding(5)
        .style(style::icon_button)
        .on_press(Message::CopyText(text.to_string()));

    row![copy].into()
}

fn reasoning(summary: &str) -> Option<Element<'_, Message>> {
    let summary = summary.trim();
    if summary.is_empty() {
        return None;
    }

    let line = selectable_text(summary)
        .size(14)
        .line_height(LineHeight::Relative(BODY_LINE_HEIGHT))
        .style(style::selectable(style::TEXT_MUTED));

    Some(line.into())
}

fn command_step<'a>(
    key: &'a str,
    command: &'a CommandEntry,
    expanded: bool,
) -> Element<'a, Message> {
    let header = step_header(
        key,
        Icon::Terminal,
        command_label(command),
        command_status(command),
    );

    if !expanded {
        return header;
    }

    column![header, command_details(command)].spacing(6).into()
}

fn command_details(command: &CommandEntry) -> Element<'_, Message> {
    let mut details = column![
        selectable_text(format!("$ {}", command.command))
            .font(Font::MONOSPACE)
            .size(12)
            .style(style::selectable(style::TEXT))
    ]
    .spacing(8);

    let output = output_tail(&command.output);
    if !output.is_empty() {
        details = details.push(
            selectable_text(output)
                .font(Font::MONOSPACE)
                .size(12)
                .style(style::selectable(style::TEXT_MUTED)),
        );
    }

    step_body(details)
}

fn file_change_step<'a>(
    key: &'a str,
    change: &'a FileChangeEntry,
    expanded: bool,
) -> Element<'a, Message> {
    let (added, removed) = change.changes.iter().map(diff_counts).fold(
        (0, 0),
        |(total_added, total_removed), (added, removed)| {
            (total_added + added, total_removed + removed)
        },
    );

    let header = button(
        row![
            icon::icon(Icon::File, 15.0, style::TEXT_MUTED),
            text(file_change_label(change)).size(14),
            text(format!("+{added}")).size(13).color(ADDED),
            text(format!("−{removed}")).size(13).color(style::DANGER),
            text(patch_status(&change.status))
                .size(13)
                .color(style::TEXT_FAINT),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([3, 4])
    .style(style::step_header)
    .on_press(Message::ToggleStep(key.to_string()));

    if !expanded {
        return header.into();
    }

    let diffs = column(change.changes.iter().map(file_diff)).spacing(10);
    column![header, step_body(diffs)].spacing(6).into()
}

fn file_diff(change: &FileUpdateChange) -> Element<'_, Message> {
    let header = row![
        text(change_kind_label(&change.kind))
            .font(Font::MONOSPACE)
            .size(12)
            .color(change_kind_color(&change.kind)),
        text(change.path.as_str()).font(Font::MONOSPACE).size(12),
    ]
    .spacing(10);

    let lines = column(
        change
            .diff
            .lines()
            .take(DIFF_LINE_LIMIT)
            .map(|line| diff_line(line, &change.kind)),
    );

    column![header, lines].spacing(6).into()
}

fn diff_line<'a>(line: &'a str, kind: &PatchChangeKind) -> Element<'a, Message> {
    let line_kind = classify_diff_line(line, kind);

    container(
        selectable_text(line.replace('\t', TAB_SPACES))
            .font(Font::MONOSPACE)
            .size(12)
            .wrapping(Wrapping::None)
            .style(style::selectable(style::TEXT)),
    )
    .width(Length::Fill)
    .padding([0, 6])
    .style(style::diff_line(line_kind))
    .into()
}

pub fn changed_file(change: &FileUpdateChange) -> Element<'_, Message> {
    row![
        text(change_kind_label(&change.kind))
            .font(Font::MONOSPACE)
            .size(12)
            .color(change_kind_color(&change.kind)),
        text(change.path.as_str()).font(Font::MONOSPACE).size(12),
    ]
    .spacing(10)
    .into()
}

fn tool_step(name: &str) -> Element<'_, Message> {
    row![
        icon::icon(Icon::Tool, 15.0, style::TEXT_MUTED),
        text(format!("Called {name}"))
            .size(14)
            .color(style::TEXT_MUTED),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding([3, 4])
    .into()
}

fn notice(message: &str) -> Element<'_, Message> {
    container(text(message).size(14))
        .padding([10, 14])
        .width(Length::Fill)
        .style(style::notice)
        .into()
}

fn turn_summary<'a>(duration_ms: i64) -> Element<'a, Message> {
    let label = text(format!("Worked for {}", format_duration(duration_ms)))
        .size(13)
        .color(style::TEXT_FAINT);

    row![
        rule::horizontal(1).style(style::divider),
        label,
        rule::horizontal(1).style(style::divider),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

fn thinking<'a>() -> Element<'a, Message> {
    text("Thinking…").size(14).color(style::TEXT_FAINT).into()
}

fn step_header<'a>(
    key: &str,
    kind: Icon,
    label: String,
    status: Option<&'static str>,
) -> Element<'a, Message> {
    let mut content = row![
        icon::icon(kind, 15.0, style::TEXT_MUTED),
        text(label).size(14).wrapping(Wrapping::None),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    if let Some(status) = status {
        content = content.push(text(status).size(13).color(style::TEXT_FAINT));
    }

    button(content)
        .padding([3, 4])
        .clip(true)
        .style(style::step_header)
        .on_press(Message::ToggleStep(key.to_string()))
        .into()
}

fn step_body<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding([10, 12])
        .width(Length::Fill)
        .style(style::tool_card)
        .into()
}

fn command_label(command: &CommandEntry) -> String {
    let labels: Vec<String> = command.actions.iter().filter_map(action_label).collect();
    let label = if labels.is_empty() {
        format!("Ran {}", display_command(&command.command))
    } else {
        labels.join(", ")
    };

    truncate(&label, STEP_LABEL_CHARACTERS)
}

fn action_label(action: &CommandAction) -> Option<String> {
    match action {
        CommandAction::Read { name, .. } => Some(format!("Read {name}")),
        CommandAction::ListFiles {
            path: Some(path), ..
        } => Some(format!("Listed files in {path}")),
        CommandAction::ListFiles { path: None, .. } => Some("Listed files".to_string()),
        CommandAction::Search {
            query: Some(query), ..
        } => Some(format!("Searched for {query}")),
        CommandAction::Search { query: None, .. } => Some("Searched files".to_string()),
        CommandAction::Unknown { .. } => None,
    }
}

fn command_status(command: &CommandEntry) -> Option<&'static str> {
    match (&command.status, command.exit_code) {
        (CommandExecutionStatus::InProgress, _) => Some("Running…"),
        (CommandExecutionStatus::Completed, Some(0) | None) => None,
        (CommandExecutionStatus::Completed, Some(_)) => Some("Exited with error"),
        (CommandExecutionStatus::Failed, _) => Some("Failed"),
        (CommandExecutionStatus::Declined, _) => Some("Declined"),
    }
}

fn file_change_label(change: &FileChangeEntry) -> String {
    match change.changes.as_slice() {
        [single] => format!("Edited {}", file_name(&single.path)),
        changes => format!("Edited {} files", changes.len()),
    }
}

fn patch_status(status: &PatchApplyStatus) -> &'static str {
    match status {
        PatchApplyStatus::InProgress => "Applying…",
        PatchApplyStatus::Completed => "",
        PatchApplyStatus::Failed => "Failed",
        PatchApplyStatus::Declined => "Declined",
    }
}

fn diff_counts(change: &FileUpdateChange) -> (usize, usize) {
    change
        .diff
        .lines()
        .map(|line| classify_diff_line(line, &change.kind))
        .fold((0, 0), |(added, removed), kind| match kind {
            DiffLine::Added => (added + 1, removed),
            DiffLine::Removed => (added, removed + 1),
            DiffLine::Context => (added, removed),
        })
}

fn classify_diff_line(line: &str, kind: &PatchChangeKind) -> DiffLine {
    match kind {
        PatchChangeKind::Add => DiffLine::Added,
        PatchChangeKind::Delete => DiffLine::Removed,
        PatchChangeKind::Update { .. } if line.starts_with("+++") || line.starts_with("---") => {
            DiffLine::Context
        }
        PatchChangeKind::Update { .. } if line.starts_with('+') => DiffLine::Added,
        PatchChangeKind::Update { .. } if line.starts_with('-') => DiffLine::Removed,
        PatchChangeKind::Update { .. } => DiffLine::Context,
    }
}

fn change_kind_label(kind: &PatchChangeKind) -> &'static str {
    match kind {
        PatchChangeKind::Add => "A",
        PatchChangeKind::Delete => "D",
        PatchChangeKind::Update { .. } => "M",
    }
}

fn change_kind_color(kind: &PatchChangeKind) -> Color {
    match kind {
        PatchChangeKind::Add => ADDED,
        PatchChangeKind::Delete => style::DANGER,
        PatchChangeKind::Update { .. } => style::WARNING,
    }
}

fn format_duration(duration_ms: i64) -> String {
    let seconds = (duration_ms / 1000).max(1);

    match (seconds / 60, seconds % 60) {
        (0, seconds) => format!("{seconds}s"),
        (minutes, 0) => format!("{minutes}m"),
        (minutes, seconds) => format!("{minutes}m {seconds}s"),
    }
}

fn output_tail(output: &str) -> String {
    let lines: Vec<&str> = output.trim_end().lines().collect();
    let start = lines.len().saturating_sub(OUTPUT_TAIL_LINES);

    lines[start..].join("\n").replace('\t', TAB_SPACES)
}

fn file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

fn truncate(label: &str, limit: usize) -> String {
    if label.chars().count() <= limit {
        return label.to_string();
    }

    let shortened: String = label.chars().take(limit - 1).collect();
    format!("{}…", shortened.trim_end())
}
