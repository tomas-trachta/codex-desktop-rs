use std::path::Path;

use codex_app_server_protocol::Project;
use codex_app_server_protocol::Thread;
use codex_app_server_protocol::ThreadStatus;
use iced::Alignment;
use iced::Element;
use iced::Length;
use iced::widget::Column;
use iced::widget::Id;
use iced::widget::button;
use iced::widget::column;
use iced::widget::container;
use iced::widget::row;
use iced::widget::scrollable;
use iced::widget::space;
use iced::widget::text;
use iced::widget::text::Wrapping;
use iced::widget::text_input;

use super::header_action;
use super::icon;
use super::icon::Icon;
use super::style;
use crate::app::App;
use crate::app::Message;
use crate::app::NewChatTarget;
use crate::app::SEARCH_ID;
use crate::app::thread_title;

const SIDEBAR_WIDTH: f32 = 290.0;
const THREADS_PER_PROJECT: usize = 5;
const RECENT_THREADS: usize = 12;
const TITLE_CHARACTERS: usize = 34;
const THREAD_INDENT: f32 = 26.0;
const RUNNING_DOT_SIZE: f32 = 7.0;

struct ProjectGroup<'a> {
    name: String,
    cwd: &'a Path,
    project_id: Option<&'a str>,
    threads: Vec<&'a Thread>,
}

pub fn view(app: &App) -> Element<'_, Message> {
    let active_id = app.active.as_ref().map(|thread| thread.id.as_str());
    let threads = app.visible_threads();

    let mut content = column![header(app)].spacing(4).padding([16, 12]);
    if let Some(query) = &app.search {
        content = content.push(search_box(query));
    }

    let content = content
        .push(new_chat(app))
        .push(projects_header())
        .push(projects(app, &threads, active_id))
        .push(section_label("Recents"))
        .push(recents(&threads));

    container(
        scrollable(content)
            .direction(style::thin_scrollbar())
            .height(Length::Fill),
    )
    .width(SIDEBAR_WIDTH)
    .height(Length::Fill)
    .style(style::sidebar)
    .into()
}

fn header(app: &App) -> Element<'_, Message> {
    let refresh = button(icon::icon(Icon::Refresh, 16.0, style::TEXT_MUTED))
        .padding(6)
        .style(style::icon_button)
        .on_press_maybe(app.is_ready().then_some(Message::RefreshThreads));

    let title = text("Codex").size(19).font(style::SEMIBOLD);

    let search = button(icon::icon(Icon::Search, 16.0, style::TEXT_MUTED))
        .padding(6)
        .style(style::icon_button)
        .on_press(Message::ToggleSearch);

    container(
        row![title, space::horizontal(), search, refresh]
            .spacing(2)
            .align_y(Alignment::Center),
    )
    .padding(iced::Padding::ZERO.left(8))
    .into()
}

fn search_box(query: &str) -> Element<'_, Message> {
    container(
        text_input("Search chats", query)
            .id(Id::new(SEARCH_ID))
            .on_input(Message::SearchEdited)
            .size(14)
            .padding([6, 10])
            .style(style::inline_input),
    )
    .padding(iced::Padding::ZERO.top(8))
    .into()
}

fn new_chat(app: &App) -> Element<'_, Message> {
    let label = row![
        icon::icon(Icon::NewChat, 16.0, style::TEXT),
        text("New chat").size(14),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    container(
        button(label)
            .width(Length::Fill)
            .padding([7, 8])
            .style(style::row_button(false))
            .on_press_maybe(app.is_ready().then_some(Message::NewChat)),
    )
    .padding([10, 0])
    .into()
}

fn section_label<'a>(label: &'a str) -> Element<'a, Message> {
    container(text(label).size(13).color(style::TEXT_MUTED))
        .padding([10, 8])
        .into()
}

fn projects_header<'a>() -> Element<'a, Message> {
    row![
        section_label("Projects"),
        space::horizontal(),
        header_action(Icon::Plus, "Add project  Ctrl+O", Message::AddProject),
    ]
    .align_y(Alignment::Center)
    .into()
}

fn projects<'a>(
    app: &'a App,
    threads: &[&'a Thread],
    active_id: Option<&str>,
) -> Element<'a, Message> {
    let mut list = Column::new().spacing(2);
    let selected_cwd = app
        .active
        .is_none()
        .then_some(app.new_chat_target.as_ref())
        .flatten()
        .map(|target| target.cwd.as_path());

    for project in project_groups(&app.projects, threads) {
        let is_selected = selected_cwd == Some(project.cwd);
        list = list.push(project_row(&project, is_selected));
        for thread in project.threads {
            let is_active = active_id == Some(thread.id.as_str());
            list = list.push(
                container(thread_row(thread, is_active))
                    .padding(iced::Padding::ZERO.left(THREAD_INDENT)),
            );
        }
    }

    list.into()
}

fn recents<'a>(threads: &[&'a Thread]) -> Element<'a, Message> {
    let rows = threads
        .iter()
        .take(RECENT_THREADS)
        .map(|thread| thread_row(thread, false));

    Column::with_children(rows).spacing(2).into()
}

fn project_row<'a>(project: &ProjectGroup<'_>, is_selected: bool) -> Element<'a, Message> {
    let label = row![
        icon::icon(Icon::Folder, 16.0, style::TEXT_MUTED),
        text(project.name.clone()).size(14),
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    let target = NewChatTarget {
        name: project.name.clone(),
        cwd: project.cwd.to_path_buf(),
        project_id: project.project_id.map(str::to_string),
    };

    button(label)
        .width(Length::Fill)
        .padding([7, 8])
        .style(style::row_button(is_selected))
        .on_press(Message::NewChatIn(target))
        .into()
}

fn thread_row(thread: &Thread, is_active: bool) -> Element<'_, Message> {
    let title = text(truncate(&thread_title(thread)))
        .size(14)
        .wrapping(Wrapping::None)
        .width(Length::Fill);

    let mut label = row![title].spacing(8).align_y(Alignment::Center);
    if matches!(thread.status, ThreadStatus::Active { .. }) {
        label = label.push(running_dot());
    }

    button(label)
        .width(Length::Fill)
        .padding([7, 8])
        .clip(true)
        .style(style::row_button(is_active))
        .on_press(Message::OpenThread(thread.id.clone()))
        .into()
}

fn running_dot<'a>() -> Element<'a, Message> {
    container(space())
        .width(RUNNING_DOT_SIZE)
        .height(RUNNING_DOT_SIZE)
        .style(style::running_dot)
        .into()
}

fn project_groups<'a>(projects: &'a [Project], threads: &[&'a Thread]) -> Vec<ProjectGroup<'a>> {
    let mut groups: Vec<ProjectGroup<'a>> = projects.iter().filter_map(saved_project).collect();

    for &thread in threads {
        match groups.iter_mut().find(|group| group.contains(thread)) {
            Some(group) if group.threads.len() < THREADS_PER_PROJECT => group.threads.push(thread),
            Some(_) => {}
            None => groups.push(folder_project(thread)),
        }
    }

    groups
}

fn saved_project(project: &Project) -> Option<ProjectGroup<'_>> {
    let root = project.roots.first()?;

    Some(ProjectGroup {
        name: project.name.clone(),
        cwd: root.path.as_path(),
        project_id: Some(project.id.as_str()),
        threads: Vec::new(),
    })
}

fn folder_project(thread: &Thread) -> ProjectGroup<'_> {
    let cwd = thread.cwd.as_path();

    ProjectGroup {
        name: folder_name(cwd),
        cwd,
        project_id: None,
        threads: vec![thread],
    }
}

impl ProjectGroup<'_> {
    fn contains(&self, thread: &Thread) -> bool {
        let same_project = self
            .project_id
            .is_some_and(|id| thread.project_id.as_deref() == Some(id));

        same_project || thread.cwd.as_path().starts_with(self.cwd)
    }
}

fn folder_name(cwd: &Path) -> String {
    cwd.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| cwd.display().to_string())
}

fn truncate(title: &str) -> String {
    if title.chars().count() <= TITLE_CHARACTERS {
        return title.to_string();
    }

    let shortened: String = title.chars().take(TITLE_CHARACTERS - 1).collect();
    format!("{}…", shortened.trim_end())
}
