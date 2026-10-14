use iced::Element;
use iced::Font;
use iced::Length;
use iced::widget::button;
use iced::widget::center;
use iced::widget::column;
use iced::widget::container;
use iced::widget::opaque;
use iced::widget::row;
use iced::widget::space;
use iced::widget::text;

use super::style;
use super::transcript::changed_file;
use crate::app::App;
use crate::app::Message;
use crate::approval::ApprovalSubject;
use crate::approval::Decision;
use crate::approval::PendingApproval;

const DIALOG_WIDTH: f32 = 560.0;

pub fn view<'a>(app: &'a App, pending: &'a PendingApproval) -> Element<'a, Message> {
    let mut dialog = column![heading(app, pending)].spacing(14);

    if let Some(reason) = &pending.reason {
        dialog = dialog.push(text(reason).size(13).style(text::secondary));
    }

    let dialog = dialog.push(details(app, pending)).push(actions());

    let card = container(dialog)
        .padding(20)
        .max_width(DIALOG_WIDTH)
        .style(style::dialog);

    opaque(center(card).style(style::backdrop))
}

fn heading<'a>(app: &App, pending: &PendingApproval) -> Element<'a, Message> {
    let title = match pending.subject {
        ApprovalSubject::Command { .. } => "Allow Codex to run this command?",
        ApprovalSubject::FileChange { .. } => "Allow Codex to apply these changes?",
    };

    let mut heading = row![text(title).size(17), space::horizontal()];
    if app.approvals.len() > 1 {
        let queued = format!("{} pending", app.approvals.len());
        heading = heading.push(text(queued).size(12).style(text::secondary));
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
        .padding(12)
        .width(Length::Fill)
        .style(style::tool_card)
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
        body = body.push(text(format!("in {cwd}")).size(12).style(text::secondary));
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
                .style(text::secondary),
        );
    }

    body.into()
}

fn actions<'a>() -> Element<'a, Message> {
    row![
        space::horizontal(),
        decision_button("Decline", Decision::Decline, button::secondary),
        decision_button(
            "Allow for session",
            Decision::AcceptForSession,
            button::secondary
        ),
        decision_button("Allow once", Decision::Accept, button::primary),
    ]
    .spacing(8)
    .into()
}

fn decision_button<'a>(
    label: &'a str,
    decision: Decision,
    style: fn(&iced::Theme, button::Status) -> button::Style,
) -> Element<'a, Message> {
    button(text(label).size(13))
        .padding([8, 14])
        .style(style)
        .on_press(Message::Decide(decision))
        .into()
}
