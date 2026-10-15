use iced::Alignment;
use iced::Element;
use iced::Font;
use iced::Length;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::row;
use iced::widget::space;
use iced::widget::text;

use super::icon;
use super::icon::Icon;
use super::style;
use super::transcript::CONTENT_WIDTH;
use super::transcript::changed_file;
use crate::app::App;
use crate::app::Message;
use codex_gui::approval::ApprovalSubject;
use codex_gui::approval::Decision;
use codex_gui::approval::PendingApproval;

pub fn view<'a>(app: &'a App, pending: &'a PendingApproval) -> Element<'a, Message> {
    let mut card = column![heading(app, pending)].spacing(12);

    if let Some(reason) = &pending.reason {
        card = card.push(text(reason).size(14).color(style::TEXT_MUTED));
    }

    let card = container(card.push(details(app, pending)).push(actions()))
        .padding([16, 18])
        .width(Length::Fill)
        .style(style::dialog);

    container(container(card).max_width(CONTENT_WIDTH))
        .center_x(Length::Fill)
        .padding([0, 32])
        .into()
}

fn heading<'a>(app: &App, pending: &PendingApproval) -> Element<'a, Message> {
    let title = match pending.subject {
        ApprovalSubject::Command { .. } => "Run this command?",
        ApprovalSubject::FileChange { .. } => "Apply these changes?",
    };

    let mut heading = row![
        icon::icon(Icon::Shield, 16.0, style::ACCENT),
        text(title).size(15).font(style::SEMIBOLD),
        space::horizontal(),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    if app.approvals.len() > 1 {
        let queued = format!("{} pending", app.approvals.len());
        heading = heading.push(text(queued).size(12).color(style::TEXT_MUTED));
    }

    heading.into()
}

fn details<'a>(app: &'a App, pending: &'a PendingApproval) -> Element<'a, Message> {
    let body = match &pending.subject {
        ApprovalSubject::Command { command, cwd } => {
            command_details(command.as_deref(), cwd.as_deref())
        }
        ApprovalSubject::FileChange { grant_root } => {
            let root = grant_root.as_ref().map(|root| root.display().to_string());
            file_change_details(app, pending, root)
        }
    };

    container(body)
        .padding([10, 12])
        .width(Length::Fill)
        .style(style::code_block)
        .into()
}

fn command_details<'a>(command: Option<&'a str>, cwd: Option<&'a str>) -> Element<'a, Message> {
    let mut body = column![
        text(command.unwrap_or("(command not provided)"))
            .font(Font::MONOSPACE)
            .size(13)
    ]
    .spacing(6);

    if let Some(cwd) = cwd {
        body = body.push(text(format!("in {cwd}")).size(12).color(style::TEXT_MUTED));
    }

    body.into()
}

fn file_change_details<'a>(
    app: &'a App,
    pending: &'a PendingApproval,
    grant_root: Option<String>,
) -> Element<'a, Message> {
    let changes = app
        .active
        .as_ref()
        .filter(|thread| thread.id == pending.thread_id)
        .and_then(|thread| thread.transcript.file_changes(&pending.item_id))
        .unwrap_or_default();

    let mut body = column(changes.iter().map(changed_file)).spacing(4);

    if changes.is_empty() {
        body = body.push(text("The changed files are not loaded in this view.").size(12));
    }
    if let Some(root) = grant_root {
        body = body.push(
            text(format!("Grants write access to {root}"))
                .size(12)
                .color(style::TEXT_MUTED),
        );
    }

    body.into()
}

fn actions<'a>() -> Element<'a, Message> {
    row![
        space::horizontal(),
        decision_button(
            "Always allow",
            None,
            Decision::AcceptForSession,
            style::secondary_button
        ),
        decision_button(
            "Decline",
            Some("Esc"),
            Decision::Decline,
            style::secondary_button
        ),
        decision_button(
            "Approve",
            Some("⏎"),
            Decision::Accept,
            style::primary_button
        ),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn decision_button<'a>(
    label: &'a str,
    hint: Option<&'a str>,
    decision: Decision,
    style: fn(&iced::Theme, button::Status) -> button::Style,
) -> Element<'a, Message> {
    let mut content = row![text(label).size(13)]
        .spacing(8)
        .align_y(Alignment::Center);
    if let Some(hint) = hint {
        content = content.push(text(hint).size(12).color(style::TEXT_FAINT));
    }

    button(content)
        .padding([7, 14])
        .style(style)
        .on_press(Message::Decide(decision))
        .into()
}
