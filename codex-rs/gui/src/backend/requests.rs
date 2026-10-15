use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicI64;
use std::sync::atomic::Ordering;

use codex_app_server_client::InProcessAppServerRequestHandle;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::ModelListParams;
use codex_app_server_protocol::ModelListResponse;
use codex_app_server_protocol::ProjectCreateParams;
use codex_app_server_protocol::ProjectCreateResponse;
use codex_app_server_protocol::ProjectListParams;
use codex_app_server_protocol::ProjectListResponse;
use codex_app_server_protocol::ProjectRoot;
use codex_app_server_protocol::ProjectSortKey;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::SortDirection;
use codex_app_server_protocol::Thread;
use codex_app_server_protocol::ThreadArchiveParams;
use codex_app_server_protocol::ThreadArchiveResponse;
use codex_app_server_protocol::ThreadListParams;
use codex_app_server_protocol::ThreadListResponse;
use codex_app_server_protocol::ThreadReadParams;
use codex_app_server_protocol::ThreadReadResponse;
use codex_app_server_protocol::ThreadResumeInitialTurnsPageParams;
use codex_app_server_protocol::ThreadResumeParams;
use codex_app_server_protocol::ThreadResumeResponse;
use codex_app_server_protocol::ThreadSetNameParams;
use codex_app_server_protocol::ThreadSetNameResponse;
use codex_app_server_protocol::ThreadSortKey;
use codex_app_server_protocol::ThreadSourceKind;
use codex_app_server_protocol::ThreadStartParams;
use codex_app_server_protocol::ThreadStartResponse;
use codex_app_server_protocol::ThreadTurnsListParams;
use codex_app_server_protocol::ThreadTurnsListResponse;
use codex_app_server_protocol::Turn;
use codex_app_server_protocol::TurnInterruptParams;
use codex_app_server_protocol::TurnInterruptResponse;
use codex_app_server_protocol::TurnItemsView;
use codex_app_server_protocol::TurnStartParams;
use codex_app_server_protocol::TurnStartResponse;
use codex_app_server_protocol::UserInput;
use codex_utils_absolute_path::AbsolutePathBuf;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use super::Access;
use super::Event;
use super::OpenedThread;
use super::Request;
use super::ThreadSettings;
use super::TurnOverrides;

const THREAD_PAGE_SIZE: u32 = 100;
const INITIAL_TURNS_PAGE_SIZE: u32 = 50;
const ACTIVE_WRITER_ERROR: &str = "already has an active writer";

#[derive(Clone)]
pub struct Requests {
    handle: InProcessAppServerRequestHandle,
    next_id: Arc<AtomicI64>,
    cwd: PathBuf,
}

impl Requests {
    pub fn new(handle: InProcessAppServerRequestHandle, cwd: PathBuf) -> Self {
        Self {
            handle,
            next_id: Arc::new(AtomicI64::new(1)),
            cwd,
        }
    }

    pub async fn execute(self, request: Request) -> Result<Option<Event>, String> {
        match request {
            Request::ListThreads => self.list_threads().await.map(Some),
            Request::ListModels => self.list_models().await.map(Some),
            Request::ListProjects => self.list_projects().await.map(Some),
            Request::CreateProject { path } => self.create_project(path).await.map(Some),
            Request::NewThread { cwd, project_id } => {
                self.start_thread(cwd, project_id).await.map(Some)
            }
            Request::OpenThread { thread_id } => self.open_thread(thread_id).await.map(Some),
            Request::SendMessage {
                thread_id,
                text,
                client_message_id,
                overrides,
            } => self
                .start_turn(thread_id, text, client_message_id, overrides)
                .await
                .map(Some),
            Request::Interrupt { thread_id, turn_id } => {
                self.interrupt(thread_id, turn_id).await.map(|()| None)
            }
            Request::Archive { thread_id } => self.archive(thread_id).await.map(Some),
            Request::Rename { thread_id, name } => self.rename(thread_id, name).await.map(Some),
        }
    }

    async fn list_threads(&self) -> Result<Event, String> {
        let params = ThreadListParams {
            cursor: None,
            limit: Some(THREAD_PAGE_SIZE),
            sort_key: Some(ThreadSortKey::UpdatedAt),
            sort_direction: None,
            model_providers: Some(Vec::new()),
            source_kinds: Some(interactive_source_kinds()),
            originators: None,
            archived: Some(false),
            section_id: None,
            project_id: None,
            parent_thread_id: None,
            ancestor_thread_id: None,
            cwd: None,
            use_state_db_only: false,
            search_term: None,
        };

        let response: ThreadListResponse = self
            .send(|request_id| ClientRequest::ThreadList { request_id, params })
            .await?;
        Ok(Event::ThreadsListed(response.data))
    }

    async fn list_models(&self) -> Result<Event, String> {
        let params = ModelListParams {
            cursor: None,
            limit: None,
            include_hidden: Some(false),
        };

        let response: ModelListResponse = self
            .send(|request_id| ClientRequest::ModelList { request_id, params })
            .await?;
        Ok(Event::ModelsListed(response.data))
    }

    async fn list_projects(&self) -> Result<Event, String> {
        let params = ProjectListParams {
            cursor: None,
            limit: None,
            sort_key: Some(ProjectSortKey::RecencyAt),
            sort_direction: None,
        };

        let response: ProjectListResponse = self
            .send(|request_id| ClientRequest::ProjectList { request_id, params })
            .await?;
        Ok(Event::ProjectsListed(response.data))
    }

    async fn create_project(&self, path: PathBuf) -> Result<Event, String> {
        let root = AbsolutePathBuf::from_absolute_path(&path).map_err(|err| err.to_string())?;
        let params = ProjectCreateParams {
            name: folder_name(&path),
            roots: vec![ProjectRoot { path: root }],
            metadata: None,
            idempotency_key: Uuid::new_v4().to_string(),
        };

        let response: ProjectCreateResponse = self
            .send(|request_id| ClientRequest::ProjectCreate { request_id, params })
            .await?;
        Ok(Event::ProjectCreated(Box::new(response.project)))
    }

    async fn start_thread(
        &self,
        cwd: Option<PathBuf>,
        project_id: Option<String>,
    ) -> Result<Event, String> {
        let cwd = cwd.unwrap_or_else(|| self.cwd.clone());
        let params = ThreadStartParams {
            cwd: Some(cwd.to_string_lossy().into_owned()),
            project_id,
            ..ThreadStartParams::default()
        };

        let response: ThreadStartResponse = self
            .send(|request_id| ClientRequest::ThreadStart { request_id, params })
            .await?;
        let settings = ThreadSettings {
            model: Some(response.model),
            reasoning_effort: response.reasoning_effort,
            approval_policy: Some(response.approval_policy),
            approvals_reviewer: Some(response.approvals_reviewer),
            permission_profile: response.active_permission_profile.map(|profile| profile.id),
        };

        Ok(opened(
            response.thread,
            Vec::new(),
            Access::Writable,
            settings,
        ))
    }

    async fn open_thread(&self, thread_id: String) -> Result<Event, String> {
        match self.resume_thread(thread_id.clone()).await {
            Err(message) if message.contains(ACTIVE_WRITER_ERROR) => {
                self.view_thread(thread_id).await
            }
            result => result,
        }
    }

    async fn view_thread(&self, thread_id: String) -> Result<Event, String> {
        let turns = self.recent_turns(&thread_id).await?;
        let thread = self.read_thread(thread_id).await?;

        let settings = ThreadSettings {
            model: thread.model.clone(),
            reasoning_effort: thread.reasoning_effort.clone(),
            ..ThreadSettings::default()
        };

        Ok(opened(thread, turns, Access::ReadOnly, settings))
    }

    async fn read_thread(&self, thread_id: String) -> Result<Thread, String> {
        let params = ThreadReadParams {
            thread_id,
            include_turns: false,
        };

        let response: ThreadReadResponse = self
            .send(|request_id| ClientRequest::ThreadRead { request_id, params })
            .await?;
        Ok(response.thread)
    }

    async fn recent_turns(&self, thread_id: &str) -> Result<Vec<Turn>, String> {
        match self.turns_page(thread_id, TurnItemsView::Full).await {
            Ok(turns) => Ok(turns),
            Err(_) => self.turns_page(thread_id, TurnItemsView::Summary).await,
        }
    }

    async fn turns_page(
        &self,
        thread_id: &str,
        items_view: TurnItemsView,
    ) -> Result<Vec<Turn>, String> {
        let params = ThreadTurnsListParams {
            thread_id: thread_id.to_string(),
            cursor: None,
            limit: Some(INITIAL_TURNS_PAGE_SIZE),
            sort_direction: Some(SortDirection::Desc),
            items_view: Some(items_view),
        };

        let response: ThreadTurnsListResponse = self
            .send(|request_id| ClientRequest::ThreadTurnsList { request_id, params })
            .await?;
        Ok(response.data.into_iter().rev().collect())
    }

    async fn resume_thread(&self, thread_id: String) -> Result<Event, String> {
        let params = ThreadResumeParams {
            thread_id,
            exclude_turns: true,
            initial_turns_page: Some(ThreadResumeInitialTurnsPageParams {
                limit: Some(INITIAL_TURNS_PAGE_SIZE),
                sort_direction: Some(SortDirection::Desc),
                items_view: Some(TurnItemsView::Full),
            }),
            ..ThreadResumeParams::default()
        };

        let response: ThreadResumeResponse = self
            .send(|request_id| ClientRequest::ThreadResume { request_id, params })
            .await?;
        Ok(opened_from_resume(response))
    }

    async fn start_turn(
        &self,
        thread_id: String,
        text: String,
        client_message_id: String,
        overrides: TurnOverrides,
    ) -> Result<Event, String> {
        let params = TurnStartParams {
            thread_id: thread_id.clone(),
            model: overrides.model,
            effort: overrides.effort,
            approval_policy: overrides.approval_policy,
            permissions: overrides.permissions,
            client_user_message_id: Some(client_message_id),
            input: vec![UserInput::Text {
                text,
                text_elements: Vec::new(),
            }],
            ..TurnStartParams::default()
        };

        let response: TurnStartResponse = self
            .send(|request_id| ClientRequest::TurnStart { request_id, params })
            .await?;
        Ok(Event::TurnStarted {
            thread_id,
            turn_id: response.turn.id,
        })
    }

    async fn interrupt(&self, thread_id: String, turn_id: String) -> Result<(), String> {
        let params = TurnInterruptParams { thread_id, turn_id };

        let _: TurnInterruptResponse = self
            .send(|request_id| ClientRequest::TurnInterrupt { request_id, params })
            .await?;
        Ok(())
    }

    async fn archive(&self, thread_id: String) -> Result<Event, String> {
        let params = ThreadArchiveParams {
            thread_id: thread_id.clone(),
        };

        let _: ThreadArchiveResponse = self
            .send(|request_id| ClientRequest::ThreadArchive { request_id, params })
            .await?;
        Ok(Event::ThreadArchived { thread_id })
    }

    async fn rename(&self, thread_id: String, name: String) -> Result<Event, String> {
        let params = ThreadSetNameParams {
            thread_id: thread_id.clone(),
            name: name.clone(),
        };

        let _: ThreadSetNameResponse = self
            .send(|request_id| ClientRequest::ThreadSetName { request_id, params })
            .await?;
        Ok(Event::ThreadRenamed { thread_id, name })
    }

    async fn send<T>(&self, build: impl FnOnce(RequestId) -> ClientRequest) -> Result<T, String>
    where
        T: DeserializeOwned,
    {
        let request = build(self.next_request_id());

        self.handle
            .request_typed(request)
            .await
            .map_err(|err| err.to_string())
    }

    fn next_request_id(&self) -> RequestId {
        RequestId::Integer(self.next_id.fetch_add(1, Ordering::Relaxed))
    }
}

fn interactive_source_kinds() -> Vec<ThreadSourceKind> {
    vec![
        ThreadSourceKind::Cli,
        ThreadSourceKind::VsCode,
        ThreadSourceKind::AppServer,
    ]
}

fn opened_from_resume(response: ThreadResumeResponse) -> Event {
    let turns = match response.initial_turns_page {
        Some(page) => page.data.into_iter().rev().collect(),
        None => response.thread.turns.clone(),
    };

    let settings = ThreadSettings {
        model: Some(response.model),
        reasoning_effort: response.reasoning_effort,
        approval_policy: Some(response.approval_policy),
        approvals_reviewer: Some(response.approvals_reviewer),
        permission_profile: response.active_permission_profile.map(|profile| profile.id),
    };

    opened(response.thread, turns, Access::Writable, settings)
}

fn opened(thread: Thread, turns: Vec<Turn>, access: Access, settings: ThreadSettings) -> Event {
    Event::ThreadOpened(Box::new(OpenedThread {
        thread,
        turns,
        access,
        settings,
    }))
}

fn folder_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}
