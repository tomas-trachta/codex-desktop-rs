use std::collections::HashSet;
use std::collections::VecDeque;
use std::path::PathBuf;

use codex_app_server_protocol::Model;
use codex_app_server_protocol::Project;
use codex_app_server_protocol::ServerNotification;
use codex_app_server_protocol::Thread;
use codex_app_server_protocol::Turn;
use codex_app_server_protocol::TurnStatus;
use codex_protocol::openai_models::ReasoningEffort;
use iced::Size;
use iced::Subscription;
use iced::Task;
use iced::Theme;
use iced::event;
use iced::keyboard;
use iced::widget::Id;
use iced::widget::markdown;
use iced::widget::operation;
use iced::widget::text_editor;

use crate::approval_modes;
use crate::shortcuts;
use crate::transcript::Transcript;
use crate::view;
use crate::view::style;
use codex_gui::approval::Decision;
use codex_gui::approval::PendingApproval;
use codex_gui::backend;
use codex_gui::backend::Access;
use codex_gui::backend::Event;
use codex_gui::backend::Handle;
use codex_gui::backend::OpenedThread;
use codex_gui::backend::Request;
use codex_gui::backend::ThreadSettings;
use codex_gui::backend::TurnOverrides;
use codex_gui::launch::LaunchOptions;

const WINDOW_SIZE: Size = Size::new(1280.0, 820.0);
pub const COMPOSER_ID: &str = "composer";
pub const RENAME_ID: &str = "rename";
pub const SEARCH_ID: &str = "search";
const WORD_BOUNDARY: char = '\u{200B}';

pub fn run(launch: LaunchOptions) -> iced::Result {
    iced::application(move || App::new(launch.clone()), App::update, view::app)
        .title(App::title)
        .subscription(App::subscription)
        .theme(App::theme)
        .default_font(style::FONT)
        .window_size(WINDOW_SIZE)
        .run()
}

pub enum Connection {
    Starting,
    Ready(Handle),
    Failed(String),
}

pub struct ActiveThread {
    pub id: String,
    pub title: String,
    pub transcript: Transcript,
    pub running_turn: Option<String>,
    pub access: Access,
    pub settings: ThreadSettings,
    pub expanded_steps: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct NewChatTarget {
    pub name: String,
    pub cwd: PathBuf,
    pub project_id: Option<String>,
}

pub struct App {
    pub launch: LaunchOptions,
    pub theme: Theme,
    pub connection: Connection,
    pub threads: Vec<Thread>,
    pub active: Option<ActiveThread>,
    pub composer: text_editor::Content,
    pub approvals: VecDeque<PendingApproval>,
    pub error: Option<String>,
    pub sidebar_visible: bool,
    pub renaming: Option<String>,
    pub search: Option<String>,
    pub models: Vec<Model>,
    pub projects: Vec<Project>,
    pub overrides: TurnOverrides,
    pub model_menu_open: bool,
    pub approval_menu_open: bool,
    pub new_chat_target: Option<NewChatTarget>,
    last_prompt: Option<String>,
    queued_message: Option<String>,
    sent_messages: u64,
}

#[derive(Debug, Clone)]
pub enum Message {
    Backend(Event),
    NewChat,
    OpenThread(String),
    OpenRecent(usize),
    CycleThread(isize),
    ToggleSidebar,
    ToggleStep(String),
    RestorePrompt,
    Keyboard(keyboard::Event),
    Escape,
    CleanClipboard,
    ToggleModelMenu,
    SelectModel(String),
    SelectEffort(ReasoningEffort),
    ToggleApprovalMenu,
    SelectApprovalMode(&'static str),
    AddProject,
    ProjectFolderPicked(Option<PathBuf>),
    NewChatIn(NewChatTarget),
    ArchiveThread,
    StartRename,
    RenameEdited(String),
    CommitRename,
    ToggleSearch,
    SearchEdited(String),
    RefreshThreads,
    ComposerEdited(text_editor::Action),
    Submit,
    Interrupt,
    Decide(Decision),
    DismissError,
    LinkClicked(markdown::Uri),
    CopyText(String),
}

impl Message {
    fn name(&self) -> &'static str {
        match self {
            Self::Backend(event) => event.name(),
            Self::NewChat => "new_chat",
            Self::OpenThread(_) => "open_thread",
            Self::OpenRecent(_) => "open_recent",
            Self::RefreshThreads => "refresh_threads",
            Self::CycleThread(_) => "cycle_thread",
            Self::ToggleSidebar => "toggle_sidebar",
            Self::ToggleStep(_) => "toggle_step",
            Self::RestorePrompt => "restore_prompt",
            Self::Keyboard(_) => "keyboard",
            Self::Escape => "escape",
            Self::CleanClipboard => "clean_clipboard",
            Self::ToggleModelMenu => "toggle_model_menu",
            Self::SelectModel(_) => "select_model",
            Self::SelectEffort(_) => "select_effort",
            Self::ToggleApprovalMenu => "toggle_approval_menu",
            Self::SelectApprovalMode(_) => "select_approval_mode",
            Self::AddProject => "add_project",
            Self::ProjectFolderPicked(_) => "project_folder_picked",
            Self::NewChatIn(_) => "new_chat_in",
            Self::ArchiveThread => "archive_thread",
            Self::StartRename => "start_rename",
            Self::RenameEdited(_) => "rename_edited",
            Self::CommitRename => "commit_rename",
            Self::ToggleSearch => "toggle_search",
            Self::SearchEdited(_) => "search_edited",
            Self::ComposerEdited(_) => "composer_edited",
            Self::Submit => "submit",
            Self::Interrupt => "interrupt",
            Self::Decide(_) => "decide",
            Self::DismissError => "dismiss_error",
            Self::LinkClicked(_) => "link_clicked",
            Self::CopyText(_) => "copy_text",
        }
    }
}

impl App {
    fn new(launch: LaunchOptions) -> Self {
        Self {
            launch,
            theme: style::theme(),
            connection: Connection::Starting,
            threads: Vec::new(),
            active: None,
            composer: text_editor::Content::new(),
            approvals: VecDeque::new(),
            error: None,
            sidebar_visible: true,
            renaming: None,
            search: None,
            models: Vec::new(),
            projects: Vec::new(),
            overrides: TurnOverrides::default(),
            model_menu_open: false,
            approval_menu_open: false,
            new_chat_target: None,
            last_prompt: None,
            queued_message: None,
            sent_messages: 0,
        }
    }

    fn title(&self) -> String {
        match &self.active {
            Some(thread) => format!("{} — Codex", thread.title),
            None => "Codex".to_string(),
        }
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            Subscription::run_with(self.launch.clone(), backend::connect).map(Message::Backend),
            keyboard::listen().map(Message::Keyboard),
            event::listen_with(shortcuts::escape),
            event::listen_with(shortcuts::copy),
        ])
    }

    pub fn has_pending_approval(&self) -> bool {
        !self.approvals.is_empty()
    }

    pub fn is_ready(&self) -> bool {
        matches!(self.connection, Connection::Ready(_))
    }

    pub fn is_turn_running(&self) -> bool {
        self.active
            .as_ref()
            .is_some_and(|thread| thread.running_turn.is_some())
    }

    pub fn is_read_only(&self) -> bool {
        self.active
            .as_ref()
            .is_some_and(|thread| thread.access == Access::ReadOnly)
    }

    pub fn can_submit(&self) -> bool {
        self.is_ready()
            && !self.is_read_only()
            && !self.is_turn_running()
            && !self.composer_text().is_empty()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        tracing::debug!(message = message.name(), "update");

        match message {
            Message::Backend(event) => self.handle_backend_event(event),
            Message::NewChat => return self.new_chat(),
            Message::OpenThread(thread_id) => self.request(Request::OpenThread { thread_id }),
            Message::OpenRecent(slot) => self.open_recent(slot),
            Message::CycleThread(step) => self.cycle_thread(step),
            Message::ToggleSidebar => self.sidebar_visible = !self.sidebar_visible,
            Message::ToggleStep(key) => self.toggle_step(key),
            Message::RestorePrompt => self.restore_prompt(),
            Message::Keyboard(event) => return self.handle_shortcut(event),
            Message::Escape => self.handle_escape(),
            Message::CleanClipboard => return clean_clipboard(),
            Message::ToggleModelMenu => self.toggle_model_menu(),
            Message::SelectModel(model) => self.select_model(model),
            Message::SelectEffort(effort) => self.overrides.effort = Some(effort),
            Message::ToggleApprovalMenu => self.toggle_approval_menu(),
            Message::SelectApprovalMode(id) => self.select_approval_mode(id),
            Message::AddProject => return pick_project_folder(),
            Message::ProjectFolderPicked(path) => self.create_project(path),
            Message::NewChatIn(target) => return self.new_chat_in(target),
            Message::ArchiveThread => self.archive_active(),
            Message::StartRename => return self.start_rename(),
            Message::RenameEdited(name) => self.renaming = Some(name),
            Message::CommitRename => self.commit_rename(),
            Message::ToggleSearch => return self.toggle_search(),
            Message::SearchEdited(query) => self.search = Some(query),
            Message::RefreshThreads => self.request(Request::ListThreads),
            Message::ComposerEdited(action) => self.composer.perform(action),
            Message::Submit => self.submit(),
            Message::Interrupt => self.interrupt(),
            Message::Decide(decision) => self.decide(decision),
            Message::DismissError => self.error = None,
            Message::LinkClicked(uri) => open_link(&uri),
            Message::CopyText(text) => return iced::clipboard::write(text),
        }

        Task::none()
    }

    fn handle_shortcut(&mut self, event: keyboard::Event) -> Task<Message> {
        let context = shortcuts::Context {
            has_pending_approval: self.has_pending_approval(),
        };

        match shortcuts::global(event, &context) {
            Some(message) => self.update(message),
            None => Task::none(),
        }
    }

    fn handle_escape(&mut self) {
        if self.renaming.take().is_some() {
            return;
        }
        if self.model_menu_open || self.approval_menu_open {
            self.close_menus();
            return;
        }
        if self.has_pending_approval() {
            self.decide(Decision::Decline);
            return;
        }
        self.search = None;
    }

    pub fn effective_model(&self) -> Option<&Model> {
        let slug = self.overrides.model.as_deref().or_else(|| {
            self.active
                .as_ref()
                .and_then(|thread| thread.settings.model.as_deref())
        });

        match slug {
            Some(slug) => self.models.iter().find(|model| model.model == slug),
            None => self.models.iter().find(|model| model.is_default),
        }
    }

    pub fn effective_model_label(&self) -> Option<String> {
        let slug = self.overrides.model.clone().or_else(|| {
            self.active
                .as_ref()
                .and_then(|thread| thread.settings.model.clone())
        });

        match (self.effective_model(), slug) {
            (Some(model), _) => Some(model.display_name.clone()),
            (None, slug) => slug,
        }
    }

    pub fn effective_effort(&self) -> Option<ReasoningEffort> {
        self.overrides
            .effort
            .clone()
            .or_else(|| {
                self.active
                    .as_ref()
                    .and_then(|thread| thread.settings.reasoning_effort.clone())
            })
            .or_else(|| {
                self.effective_model()
                    .map(|model| model.default_reasoning_effort.clone())
            })
    }

    fn toggle_model_menu(&mut self) {
        let open = !self.model_menu_open;
        self.close_menus();
        self.model_menu_open = open;
    }

    fn toggle_approval_menu(&mut self) {
        let open = !self.approval_menu_open;
        self.close_menus();
        self.approval_menu_open = open;
    }

    fn close_menus(&mut self) {
        self.model_menu_open = false;
        self.approval_menu_open = false;
    }

    fn select_approval_mode(&mut self, id: &str) {
        if let Some(mode) = approval_modes::find(id) {
            self.overrides.approval_policy = Some(mode.approval_policy);
            self.overrides.permissions = Some(mode.permissions);
        }
        self.close_menus();
    }

    fn create_project(&self, path: Option<PathBuf>) {
        if let Some(path) = path {
            self.request(Request::CreateProject { path });
        }
    }

    fn new_chat_in(&mut self, target: NewChatTarget) -> Task<Message> {
        self.active = None;
        self.new_chat_target = Some(target);
        focus_composer()
    }

    fn select_model(&mut self, slug: String) {
        let supported = self
            .models
            .iter()
            .find(|model| model.model == slug)
            .map(supported_efforts)
            .unwrap_or_default();

        let keeps_effort = self
            .effective_effort()
            .is_some_and(|effort| supported.contains(&effort));
        if !keeps_effort {
            self.overrides.effort = None;
        }

        self.overrides.model = Some(slug);
    }

    fn archive_active(&self) {
        if let Some(thread) = &self.active {
            self.request(Request::Archive {
                thread_id: thread.id.clone(),
            });
        }
    }

    fn start_rename(&mut self) -> Task<Message> {
        let Some(thread) = &self.active else {
            return Task::none();
        };

        self.renaming = Some(thread.title.clone());
        operation::focus(Id::new(RENAME_ID))
    }

    fn commit_rename(&mut self) {
        let Some(name) = self.renaming.take() else {
            return;
        };
        let name = name.trim().to_string();

        if let Some(thread) = self.active.as_ref().filter(|_| !name.is_empty()) {
            self.request(Request::Rename {
                thread_id: thread.id.clone(),
                name,
            });
        }
    }

    fn toggle_search(&mut self) -> Task<Message> {
        if self.search.take().is_some() {
            return Task::none();
        }

        self.search = Some(String::new());
        operation::focus(Id::new(SEARCH_ID))
    }

    pub fn visible_threads(&self) -> Vec<&Thread> {
        let query = self
            .search
            .as_deref()
            .map(str::trim)
            .filter(|query| !query.is_empty())
            .map(str::to_lowercase);

        self.threads
            .iter()
            .filter(|thread| {
                query
                    .as_deref()
                    .is_none_or(|query| thread_title(thread).to_lowercase().contains(query))
            })
            .collect()
    }

    fn new_chat(&mut self) -> Task<Message> {
        self.active = None;
        self.new_chat_target = None;
        focus_composer()
    }

    fn open_recent(&self, slot: usize) {
        if let Some(thread) = self.threads.get(slot) {
            self.request(Request::OpenThread {
                thread_id: thread.id.clone(),
            });
        }
    }

    fn cycle_thread(&self, step: isize) {
        let current = self.active.as_ref().and_then(|active| {
            self.threads
                .iter()
                .position(|thread| thread.id == active.id)
        });
        let count = self.threads.len() as isize;
        if count == 0 {
            return;
        }

        let next = match current {
            Some(position) => (position as isize + step).rem_euclid(count),
            None => 0,
        };
        self.open_recent(next as usize);
    }

    fn toggle_step(&mut self, key: String) {
        let Some(thread) = self.active.as_mut() else {
            return;
        };

        if !thread.expanded_steps.remove(&key) {
            thread.expanded_steps.insert(key);
        }
    }

    fn restore_prompt(&mut self) {
        if let Some(prompt) = &self.last_prompt
            && self.composer.text().trim().is_empty()
        {
            self.composer = text_editor::Content::with_text(prompt);
        }
    }

    fn handle_backend_event(&mut self, event: Event) {
        match event {
            Event::Connected(handle) => self.on_connected(handle),
            Event::StartupFailed(message) | Event::Disconnected(message) => {
                self.connection = Connection::Failed(message);
            }
            Event::ThreadsListed(threads) => self.threads = threads,
            Event::ModelsListed(models) => self.models = models,
            Event::ProjectsListed(projects) => self.projects = projects,
            Event::ProjectCreated(project) => self.on_project_created(*project),
            Event::ThreadOpened(opened) => self.on_thread_opened(*opened),
            Event::TurnStarted { thread_id, turn_id } => self.on_turn_started(&thread_id, turn_id),
            Event::ThreadArchived { thread_id } => self.on_thread_archived(&thread_id),
            Event::ThreadRenamed { thread_id, name } => self.on_thread_renamed(&thread_id, name),
            Event::Notification(notification) => self.apply_notification(*notification),
            Event::ApprovalRequested(approval) => self.approvals.push_back(*approval),
            Event::RequestFailed(message) => self.error = Some(message),
        }
    }

    fn on_connected(&mut self, handle: Handle) {
        self.connection = Connection::Ready(handle);
        self.request(Request::ListThreads);
        self.request(Request::ListModels);
        self.request(Request::ListProjects);
    }

    fn on_thread_opened(&mut self, opened: OpenedThread) {
        let running_turn = opened
            .turns
            .iter()
            .find(|turn| turn.status == TurnStatus::InProgress)
            .map(|turn| turn.id.clone());

        self.active = Some(ActiveThread {
            id: opened.thread.id.clone(),
            title: thread_title(&opened.thread),
            transcript: Transcript::from_turns(&opened.turns),
            running_turn,
            access: opened.access,
            settings: opened.settings,
            expanded_steps: HashSet::new(),
        });

        self.new_chat_target = None;
        self.close_menus();
        self.request(Request::ListThreads);
        self.send_queued_message();
    }

    fn on_project_created(&mut self, project: Project) {
        if let Some(root) = project.roots.first() {
            self.active = None;
            self.new_chat_target = Some(NewChatTarget {
                name: project.name.clone(),
                cwd: root.path.to_path_buf(),
                project_id: Some(project.id.clone()),
            });
        }

        self.request(Request::ListProjects);
    }

    fn on_thread_archived(&mut self, thread_id: &str) {
        if self.thread_mut(thread_id).is_some() {
            self.active = None;
        }
        self.request(Request::ListThreads);
    }

    fn on_thread_renamed(&mut self, thread_id: &str, name: String) {
        if let Some(thread) = self.thread_mut(thread_id) {
            thread.title = name;
        }
        self.request(Request::ListThreads);
    }

    fn on_turn_started(&mut self, thread_id: &str, turn_id: String) {
        if let Some(thread) = self.thread_mut(thread_id) {
            thread.running_turn = Some(turn_id);
        }
    }

    fn apply_notification(&mut self, notification: ServerNotification) {
        match notification {
            ServerNotification::ItemStarted(event) => {
                self.with_transcript(&event.thread_id, |transcript| {
                    transcript.upsert(&event.item);
                });
            }
            ServerNotification::ItemCompleted(event) => {
                self.with_transcript(&event.thread_id, |transcript| {
                    transcript.upsert(&event.item);
                });
            }
            ServerNotification::AgentMessageDelta(event) => {
                self.with_transcript(&event.thread_id, |transcript| {
                    transcript.append_agent_text(&event.item_id, &event.delta);
                });
            }
            ServerNotification::ReasoningSummaryTextDelta(event) => {
                self.with_transcript(&event.thread_id, |transcript| {
                    transcript.append_reasoning(&event.item_id, &event.delta);
                });
            }
            ServerNotification::CommandExecutionOutputDelta(event) => {
                self.with_transcript(&event.thread_id, |transcript| {
                    transcript.append_command_output(&event.item_id, &event.delta);
                });
            }
            ServerNotification::TurnStarted(event) => {
                self.on_turn_started(&event.thread_id, event.turn.id);
            }
            ServerNotification::TurnCompleted(event) => {
                self.on_turn_completed(&event.thread_id, &event.turn);
            }
            ServerNotification::Error(event) if !event.will_retry => {
                self.with_transcript(&event.thread_id, |transcript| {
                    transcript.push_notice(event.error.message);
                });
            }
            ServerNotification::ServerRequestResolved(event) => {
                self.approvals
                    .retain(|approval| approval.request_id != event.request_id);
            }
            ServerNotification::ThreadStarted(_) | ServerNotification::ThreadNameUpdated(_) => {
                self.request(Request::ListThreads);
            }
            _ => {}
        }
    }

    fn on_turn_completed(&mut self, thread_id: &str, turn: &Turn) {
        if let Some(thread) = self.thread_mut(thread_id) {
            thread.running_turn = None;
            if let Some(error) = &turn.error {
                thread.transcript.push_notice(error.message.clone());
            }
            thread
                .transcript
                .push_turn_summary(&turn.id, turn.duration_ms);
        }

        self.request(Request::ListThreads);
    }

    fn submit(&mut self) {
        if !self.can_submit() {
            return;
        }

        let text = self.composer_text();
        self.composer = text_editor::Content::new();
        self.last_prompt = Some(text.clone());

        match self.active.as_ref().map(|thread| thread.id.clone()) {
            Some(thread_id) => self.send_message(thread_id, text),
            None => self.start_thread_with(text),
        }
    }

    fn start_thread_with(&mut self, text: String) {
        self.queued_message = Some(text);

        let target = self.new_chat_target.take();
        self.request(Request::NewThread {
            cwd: target.as_ref().map(|target| target.cwd.clone()),
            project_id: target.and_then(|target| target.project_id),
        });
    }

    fn send_queued_message(&mut self) {
        let Some(text) = self.queued_message.take() else {
            return;
        };

        if let Some(thread_id) = self.active.as_ref().map(|thread| thread.id.clone()) {
            self.send_message(thread_id, text);
        }
    }

    fn send_message(&mut self, thread_id: String, text: String) {
        let client_message_id = self.next_client_message_id();

        if let Some(thread) = self.thread_mut(&thread_id) {
            thread
                .transcript
                .push_pending_user_message(client_message_id.clone(), text.clone());
        }

        self.close_menus();
        self.request(Request::SendMessage {
            thread_id,
            text,
            client_message_id,
            overrides: self.overrides.clone(),
        });
    }

    fn interrupt(&mut self) {
        let Some(thread) = &self.active else {
            return;
        };
        let Some(turn_id) = thread.running_turn.clone() else {
            return;
        };

        let thread_id = thread.id.clone();
        self.request(Request::Interrupt { thread_id, turn_id });
    }

    fn decide(&mut self, decision: Decision) {
        let Some(approval) = self.approvals.pop_front() else {
            return;
        };

        if let Connection::Ready(handle) = &self.connection {
            handle.resolve(approval.request_id.clone(), approval.response(decision));
        }
    }

    fn request(&self, request: Request) {
        if let Connection::Ready(handle) = &self.connection {
            handle.request(request);
        }
    }

    fn with_transcript(&mut self, thread_id: &str, apply: impl FnOnce(&mut Transcript)) {
        if let Some(thread) = self.thread_mut(thread_id) {
            apply(&mut thread.transcript);
        }
    }

    fn thread_mut(&mut self, thread_id: &str) -> Option<&mut ActiveThread> {
        self.active.as_mut().filter(|thread| thread.id == thread_id)
    }

    fn composer_text(&self) -> String {
        self.composer.text().trim().to_string()
    }

    fn next_client_message_id(&mut self) -> String {
        self.sent_messages += 1;
        format!("codex-gui-{}-{}", std::process::id(), self.sent_messages)
    }
}

pub fn supported_efforts(model: &Model) -> Vec<ReasoningEffort> {
    model
        .supported_reasoning_efforts
        .iter()
        .map(|option| option.reasoning_effort.clone())
        .collect()
}

pub fn thread_title(thread: &Thread) -> String {
    let name = thread
        .name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty());
    let preview = thread
        .preview
        .lines()
        .next()
        .map(str::trim)
        .filter(|line| !line.is_empty());

    name.or(preview).unwrap_or("Untitled thread").to_string()
}

fn clean_clipboard() -> Task<Message> {
    iced::clipboard::read().then(|contents| match contents {
        Some(text) if text.contains(WORD_BOUNDARY) => {
            iced::clipboard::write(text.replace(WORD_BOUNDARY, ""))
        }
        _ => Task::none(),
    })
}

fn pick_project_folder() -> Task<Message> {
    Task::perform(
        async {
            rfd::AsyncFileDialog::new()
                .set_title("Add project")
                .pick_folder()
                .await
                .map(|folder| folder.path().to_path_buf())
        },
        Message::ProjectFolderPicked,
    )
}

fn focus_composer() -> Task<Message> {
    operation::focus(Id::new(COMPOSER_ID))
}

fn open_link(uri: &str) {
    if let Err(err) = webbrowser::open(uri) {
        tracing::warn!(%err, uri, "failed to open link");
    }
}
