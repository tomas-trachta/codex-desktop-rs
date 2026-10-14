mod approval;
mod composer;
mod sidebar;
mod style;
mod transcript;

use iced::Alignment;
use iced::Element;
use iced::Length;
use iced::widget::button;
use iced::widget::center;
use iced::widget::column;
use iced::widget::container;
use iced::widget::row;
use iced::widget::rule;
use iced::widget::space;
use iced::widget::stack;
use iced::widget::text;

use crate::app::ActiveThread;
use crate::app::App;
use crate::app::Connection;
use crate::app::Message;

pub fn app(app: &App) -> Element<'_, Message> {
    let layout = row![sidebar::view(app), rule::vertical(1), main_panel(app)];

    match app.approvals.front() {
        Some(pending) => stack![layout, approval::view(app, pending)].into(),
        None => layout.into(),
    }
}

fn main_panel(app: &App) -> Element<'_, Message> {
    let body = match &app.connection {
        Connection::Starting => status_screen("Starting Codex…", None),
        Connection::Failed(message) => status_screen("Codex failed to start", Some(message)),
        Connection::Ready(_) => conversation(app),
    };

    container(body)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn conversation(app: &App) -> Element<'_, Message> {
    let mut panel = column![header(app)];

    if let Some(message) = &app.error {
        panel = panel.push(error_banner(message));
    }

    let content = match &app.active {
        Some(thread) => transcript::view(&thread.transcript, &app.theme),
        None => empty_state(app),
    };

    panel.push(content).push(composer::view(app)).into()
}

fn header(app: &App) -> Element<'_, Message> {
    let title = app
        .active
        .as_ref()
        .map_or("New conversation", |thread| thread.title.as_str());

    let status = app.active.as_ref().and_then(running_indicator);
    let mut bar = row![text(title).size(16)]
        .spacing(12)
        .align_y(Alignment::Center);

    if let Some(status) = status {
        bar = bar.push(status);
    }
    if app.is_read_only() {
        bar = bar.push(text("Read-only").size(13).style(text::warning));
    }

    container(bar.push(space::horizontal()).push(cwd_label(app)))
        .padding([12, 20])
        .width(Length::Fill)
        .style(style::header)
        .into()
}

fn running_indicator(thread: &ActiveThread) -> Option<Element<'_, Message>> {
    thread
        .running_turn
        .as_ref()
        .map(|_| text("Working…").size(13).style(text::secondary).into())
}

fn cwd_label(app: &App) -> Element<'_, Message> {
    text(app.launch.cwd.display().to_string())
        .size(12)
        .style(text::secondary)
        .into()
}

fn error_banner(message: &str) -> Element<'_, Message> {
    let dismiss = button(text("Dismiss").size(12))
        .style(button::text)
        .on_press(Message::DismissError);

    container(
        row![text(message).size(13), space::horizontal(), dismiss]
            .spacing(12)
            .align_y(Alignment::Center),
    )
    .padding([8, 20])
    .width(Length::Fill)
    .style(container::danger)
    .into()
}

fn empty_state(app: &App) -> Element<'_, Message> {
    let hint = format!("Start a new conversation in {}", app.launch.cwd.display());

    center(
        column![
            text("What should Codex work on?").size(22),
            text(hint).size(13).style(text::secondary),
        ]
        .spacing(8)
        .align_x(Alignment::Center),
    )
    .into()
}

fn status_screen<'a>(title: &'a str, detail: Option<&'a String>) -> Element<'a, Message> {
    let mut content = column![text(title).size(20)]
        .spacing(8)
        .align_x(Alignment::Center)
        .max_width(640);

    if let Some(detail) = detail {
        content = content.push(text(detail).size(13).style(text::secondary));
    }

    center(content).into()
}
