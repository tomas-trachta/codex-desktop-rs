use std::collections::VecDeque;

use codex_app_server_protocol::ServerNotification;
use codex_app_server_protocol::Thread;
use codex_app_server_protocol::TurnStatus;
use iced::Size;
use iced::Subscription;
use iced::Task;
use iced::Theme;
use iced::widget::markdown;
use iced::widget::text_editor;

use crate::approval::Decision;
use crate::approval::PendingApproval;
use crate::backend;
use crate::backend::Access;
use crate::backend::Event;
use crate::backend::Handle;
use crate::backend::OpenedThread;
use crate::backend::Request;
use crate::launch::LaunchOptions;
use crate::transcript::Transcript;
use crate::view;

const WINDOW_SIZE: Size = Size::new(1280.0, 820.0);

pub fn run(launch: LaunchOptions) -> iced::Result {
    iced::application(move || App::new(launch.clone()), App::update, view::app)
        .title(App::title)
        .subscription(App::subscription)
        .theme(App::theme)
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
    queued_message: Option<String>,
    sent_messages: u64,
}

#[derive(Debug, Clone)]
pub enum Message {
    Backend(Event),
    NewThread,
    OpenThread(String),
    RefreshThreads,
    ComposerEdited(text_editor::Action),
    Submit,
    Interrupt,
    Decide(Decision),
    DismissError,
    LinkClicked(markdown::Uri),
}

impl App {
    fn new(launch: LaunchOptions) -> Self {
        Self {
            launch,
            theme: Theme::Dark,
            connection: Connection::Starting,
            threads: Vec::new(),
            active: None,
            composer: text_editor::Content::new(),
            approvals: VecDeque::new(),
            error: None,
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
        Subscription::run_with(self.launch.clone(), backend::connect).map(Message::Backend)
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
        match message {
            Message::Backend(event) => self.handle_backend_event(event),
            Message::NewThread => self.request(Request::NewThread),
            Message::OpenThread(thread_id) => self.request(Request::OpenThread { thread_id }),
            Message::RefreshThreads => self.request(Request::ListThreads),
            Message::ComposerEdited(action) => self.composer.perform(action),
            Message::Submit => self.submit(),
            Message::Interrupt => self.interrupt(),
            Message::Decide(decision) => self.decide(decision),
            Message::DismissError => self.error = None,
            Message::LinkClicked(uri) => open_link(&uri),
        }

        Task::none()
    }

    fn handle_backend_event(&mut self, event: Event) {
        match event {
            Event::Connected(handle) => self.on_connected(handle),
            Event::StartupFailed(message) | Event::Disconnected(message) => {
                self.connection = Connection::Failed(message);
            }
            Event::ThreadsListed(threads) => self.threads = threads,
            Event::ThreadOpened(opened) => self.on_thread_opened(*opened),
            Event::TurnStarted { thread_id, turn_id } => self.on_turn_started(&thread_id, turn_id),
            Event::Notification(notification) => self.apply_notification(*notification),
            Event::ApprovalRequested(approval) => self.approvals.push_back(*approval),
            Event::RequestFailed(message) => self.error = Some(message),
        }
    }

    fn on_connected(&mut self, handle: Handle) {
        self.connection = Connection::Ready(handle);
        self.request(Request::ListThreads);
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
        });

        self.request(Request::ListThreads);
        self.send_queued_message();
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
                self.on_turn_completed(
                    &event.thread_id,
                    event.turn.error.map(|error| error.message),
                );
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

    fn on_turn_completed(&mut self, thread_id: &str, error: Option<String>) {
        if let Some(thread) = self.thread_mut(thread_id) {
            thread.running_turn = None;
            if let Some(message) = error {
                thread.transcript.push_notice(message);
            }
        }

        self.request(Request::ListThreads);
    }

    fn submit(&mut self) {
        if !self.can_submit() {
            return;
        }

        let text = self.composer_text();
        self.composer = text_editor::Content::new();

        match self.active.as_ref().map(|thread| thread.id.clone()) {
            Some(thread_id) => self.send_message(thread_id, text),
            None => self.start_thread_with(text),
        }
    }

    fn start_thread_with(&mut self, text: String) {
        self.queued_message = Some(text);
        self.request(Request::NewThread);
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

        self.request(Request::SendMessage {
            thread_id,
            text,
            client_message_id,
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

fn open_link(uri: &str) {
    if let Err(err) = webbrowser::open(uri) {
        tracing::warn!(%err, uri, "failed to open link");
    }
}
