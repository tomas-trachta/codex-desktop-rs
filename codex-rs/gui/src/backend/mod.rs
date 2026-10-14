mod bootstrap;
mod requests;
mod session;

use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::ServerNotification;
use codex_app_server_protocol::Thread;
use codex_app_server_protocol::Turn;
use codex_async_utils::THREAD_STACK_SIZE_BYTES;
use iced::futures::SinkExt;
use iced::futures::Stream;
use serde_json::Value;
use tokio::sync::mpsc;

use crate::approval::PendingApproval;
use crate::launch::LaunchOptions;

const EVENT_BUFFER: usize = 256;

#[derive(Debug, Clone)]
pub enum Command {
    Request(Request),
    ResolveServerRequest {
        request_id: RequestId,
        response: Value,
    },
}

#[derive(Debug, Clone)]
pub enum Request {
    ListThreads,
    NewThread,
    OpenThread {
        thread_id: String,
    },
    SendMessage {
        thread_id: String,
        text: String,
        client_message_id: String,
    },
    Interrupt {
        thread_id: String,
        turn_id: String,
    },
}

#[derive(Debug, Clone)]
pub struct OpenedThread {
    pub thread: Thread,
    pub turns: Vec<Turn>,
    pub access: Access,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    Writable,
    ReadOnly,
}

#[derive(Debug, Clone)]
pub enum Event {
    Connected(Handle),
    StartupFailed(String),
    Disconnected(String),
    ThreadsListed(Vec<Thread>),
    ThreadOpened(Box<OpenedThread>),
    TurnStarted { thread_id: String, turn_id: String },
    Notification(Box<ServerNotification>),
    ApprovalRequested(Box<PendingApproval>),
    RequestFailed(String),
}

#[derive(Debug, Clone)]
pub struct Handle {
    commands: mpsc::UnboundedSender<Command>,
}

impl Handle {
    pub fn request(&self, request: Request) {
        self.send(Command::Request(request));
    }

    pub fn resolve(&self, request_id: RequestId, response: Value) {
        self.send(Command::ResolveServerRequest {
            request_id,
            response,
        });
    }

    fn send(&self, command: Command) {
        if self.commands.send(command).is_err() {
            tracing::warn!("codex-gui backend is no longer running");
        }
    }
}

pub fn connect(launch: &LaunchOptions) -> impl Stream<Item = Event> + use<> {
    let launch = launch.clone();

    iced::stream::channel(EVENT_BUFFER, async move |mut output| {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        spawn_runtime_thread(launch, command_rx, event_tx);

        let handle = Handle {
            commands: command_tx,
        };
        if output.send(Event::Connected(handle)).await.is_err() {
            return;
        }

        while let Some(event) = event_rx.recv().await {
            if output.send(event).await.is_err() {
                break;
            }
        }
    })
}

fn spawn_runtime_thread(
    launch: LaunchOptions,
    commands: mpsc::UnboundedReceiver<Command>,
    events: mpsc::UnboundedSender<Event>,
) {
    let failure_events = events.clone();

    let spawned = std::thread::Builder::new()
        .name("codex-gui-backend".to_string())
        .stack_size(THREAD_STACK_SIZE_BYTES)
        .spawn(move || run_runtime(launch, commands, events));

    if let Err(err) = spawned {
        let _ = failure_events.send(Event::StartupFailed(format!(
            "failed to spawn backend thread: {err}"
        )));
    }
}

fn run_runtime(
    launch: LaunchOptions,
    commands: mpsc::UnboundedReceiver<Command>,
    events: mpsc::UnboundedSender<Event>,
) {
    match build_runtime() {
        Ok(runtime) => runtime.block_on(session::run(launch, commands, events)),
        Err(err) => {
            let _ = events.send(Event::StartupFailed(format!(
                "failed to build tokio runtime: {err}"
            )));
        }
    }
}

fn build_runtime() -> std::io::Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_stack_size(THREAD_STACK_SIZE_BYTES)
        .build()
}
