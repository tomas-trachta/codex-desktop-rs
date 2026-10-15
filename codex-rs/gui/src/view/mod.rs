mod approval;
mod approval_menu;
mod command_display;
mod composer;
mod flat_markdown;
mod icon;
mod markdown;
mod model_menu;
mod sidebar;
pub mod style;
mod transcript;

use iced::Alignment;
use iced::Element;
use iced::Length;
use iced::widget::Id;
use iced::widget::button;
use iced::widget::center;
use iced::widget::column;
use iced::widget::container;
use iced::widget::row;
use iced::widget::rule;
use iced::widget::space;
use iced::widget::text;
use iced::widget::text_input;
use iced::widget::tooltip;

use crate::app::App;
use crate::app::Connection;
use crate::app::Message;
use crate::app::RENAME_ID;
use icon::Icon;

const HEADER_HEIGHT: f32 = 52.0;
const RENAME_WIDTH: f32 = 360.0;

pub fn app(app: &App) -> Element<'_, Message> {
    if !app.sidebar_visible {
        return main_panel(app);
    }

    row![
        sidebar::view(app),
        rule::vertical(1).style(style::divider),
        main_panel(app),
    ]
    .into()
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
        .style(style::surface)
        .into()
}

fn conversation(app: &App) -> Element<'_, Message> {
    let mut panel = column![header(app), rule::horizontal(1).style(style::divider)];

    if let Some(message) = &app.error {
        panel = panel.push(error_banner(message));
    }

    let content = match &app.active {
        Some(thread) => transcript::view(thread),
        None => empty_state(),
    };
    panel = panel.push(content);

    if let Some(pending) = app.approvals.front() {
        panel = panel.push(approval::view(app, pending));
    }
    if app.model_menu_open {
        panel = panel.push(model_menu::view(app));
    }
    if app.approval_menu_open {
        panel = panel.push(approval_menu::view(app));
    }

    panel.push(composer::view(app)).into()
}

fn header(app: &App) -> Element<'_, Message> {
    let mut bar = row![].spacing(10).align_y(Alignment::Center);

    if !app.sidebar_visible {
        bar = bar.push(header_action(
            Icon::Sidebar,
            "Show sidebar  Ctrl+B",
            Message::ToggleSidebar,
        ));
    }

    bar = bar
        .push(icon::icon(Icon::Folder, 16.0, style::TEXT_MUTED))
        .push(header_title(app));

    if app.is_turn_running() {
        bar = bar.push(text("Working…").size(13).color(style::TEXT_MUTED));
    }
    if app.is_read_only() {
        bar = bar.push(
            text("Read-only · open in another Codex window")
                .size(13)
                .color(style::WARNING),
        );
    }

    bar = bar.push(space::horizontal());
    if app.active.is_some() {
        bar = bar
            .push(header_action(
                Icon::Pencil,
                "Rename  Ctrl+Alt+R",
                Message::StartRename,
            ))
            .push(header_action(
                Icon::Archive,
                "Archive  Ctrl+Shift+A",
                Message::ArchiveThread,
            ));
    }

    container(bar)
        .height(HEADER_HEIGHT)
        .padding([0, 16])
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .into()
}

fn header_title(app: &App) -> Element<'_, Message> {
    if let Some(draft) = &app.renaming {
        return text_input("Thread name", draft)
            .id(Id::new(RENAME_ID))
            .on_input(Message::RenameEdited)
            .on_submit(Message::CommitRename)
            .size(14)
            .padding([4, 8])
            .width(RENAME_WIDTH)
            .style(style::inline_input)
            .into();
    }

    match (&app.active, &app.new_chat_target) {
        (Some(thread), _) => text(thread.title.as_str())
            .size(14)
            .font(style::SEMIBOLD)
            .into(),
        (None, Some(target)) => row![
            text("New chat").size(14).font(style::SEMIBOLD),
            text(format!("in {}", target.name))
                .size(14)
                .color(style::TEXT_MUTED),
        ]
        .spacing(6)
        .into(),
        (None, None) => text("New chat").size(14).font(style::SEMIBOLD).into(),
    }
}

pub fn header_action<'a>(kind: Icon, label: &'a str, message: Message) -> Element<'a, Message> {
    let action = button(icon::icon(kind, 16.0, style::TEXT_MUTED))
        .padding(6)
        .style(style::icon_button)
        .on_press(message);

    tooltip(
        action,
        container(text(label).size(12))
            .padding([4, 8])
            .style(style::tooltip),
        tooltip::Position::Bottom,
    )
    .into()
}

fn error_banner(message: &str) -> Element<'_, Message> {
    let dismiss = button(text("Dismiss").size(12).color(style::DANGER))
        .style(style::icon_button)
        .on_press(Message::DismissError);

    let banner = container(
        row![text(message).size(13), space::horizontal(), dismiss]
            .spacing(12)
            .align_y(Alignment::Center),
    )
    .padding([6, 12])
    .width(Length::Fill)
    .style(style::notice);

    container(banner).padding([8, 20]).into()
}

fn empty_state<'a>() -> Element<'a, Message> {
    center(
        column![
            text("What should Codex work on?")
                .size(24)
                .font(style::SEMIBOLD),
            text("Describe a task, ask a question, or paste an error.")
                .size(14)
                .color(style::TEXT_MUTED),
        ]
        .spacing(8)
        .align_x(Alignment::Center),
    )
    .into()
}

fn status_screen<'a>(title: &'a str, detail: Option<&'a String>) -> Element<'a, Message> {
    let mut content = column![text(title).size(20).font(style::SEMIBOLD)]
        .spacing(8)
        .align_x(Alignment::Center)
        .max_width(640);

    if let Some(detail) = detail {
        content = content.push(text(detail).size(13).color(style::TEXT_MUTED));
    }

    center(content).into()
}
