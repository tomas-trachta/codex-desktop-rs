use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use codex_app_server_protocol::Thread;
use iced::Alignment;
use iced::Element;
use iced::Length;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::space;
use iced::widget::text;

use super::style;
use crate::app::App;
use crate::app::Message;
use crate::app::thread_title;

const SIDEBAR_WIDTH: f32 = 280.0;
const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;

pub fn view(app: &App) -> Element<'_, Message> {
    let active_id = app.active.as_ref().map(|thread| thread.id.as_str());
    let now = unix_now();

    let threads = column(
        app.threads
            .iter()
            .map(|thread| thread_entry(thread, active_id == Some(thread.id.as_str()), now)),
    )
    .spacing(2);

    let content = column![
        toolbar(app),
        text("Threads").size(12).style(text::secondary),
        scrollable(threads).height(Length::Fill),
    ]
    .spacing(12)
    .padding(12);

    container(content)
        .width(SIDEBAR_WIDTH)
        .height(Length::Fill)
        .style(style::sidebar)
        .into()
}

fn toolbar(app: &App) -> Element<'_, Message> {
    let ready = app.is_ready();

    let new_thread = button(text("New thread").size(13))
        .style(button::primary)
        .on_press_maybe(ready.then_some(Message::NewThread));
    let refresh = button(text("Refresh").size(13))
        .style(button::text)
        .on_press_maybe(ready.then_some(Message::RefreshThreads));

    row![
        text("Codex").size(20),
        space::horizontal(),
        refresh,
        new_thread
    ]
    .spacing(6)
    .align_y(Alignment::Center)
    .into()
}

fn thread_entry(thread: &Thread, is_active: bool, now: i64) -> Element<'_, Message> {
    let details = format!(
        "{} · {}",
        folder_name(thread),
        relative_time(now - thread.updated_at)
    );

    let label = column![
        text(thread_title(thread)).size(14),
        text(details).size(11).style(text::secondary),
    ]
    .spacing(2);

    let style = if is_active {
        button::secondary
    } else {
        button::text
    };

    button(label)
        .width(Length::Fill)
        .padding([8, 10])
        .style(style)
        .on_press(Message::OpenThread(thread.id.clone()))
        .into()
}

fn folder_name(thread: &Thread) -> String {
    thread
        .cwd
        .as_path()
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| thread.cwd.as_path().display().to_string())
}

fn relative_time(elapsed_seconds: i64) -> String {
    match elapsed_seconds.max(0) {
        seconds if seconds < MINUTE => "just now".to_string(),
        seconds if seconds < HOUR => format!("{}m ago", seconds / MINUTE),
        seconds if seconds < DAY => format!("{}h ago", seconds / HOUR),
        seconds => format!("{}d ago", seconds / DAY),
    }
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or_default()
}
