use codex_app_server_client::InProcessAppServerClient;
use codex_app_server_client::InProcessServerEvent;
use codex_app_server_protocol::JSONRPCErrorError;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::ServerRequest;
use serde_json::Value;
use tokio::sync::mpsc;

use super::Command;
use super::Event;
use super::Request;
use super::bootstrap;
use super::requests::Requests;
use crate::approval::PendingApproval;
use crate::launch::LaunchOptions;

const UNSUPPORTED_REQUEST_CODE: i64 = -32601;

pub async fn run(
    launch: LaunchOptions,
    commands: mpsc::UnboundedReceiver<Command>,
    events: mpsc::UnboundedSender<Event>,
) {
    let client = match bootstrap::start_client(&launch).await {
        Ok(client) => client,
        Err(err) => {
            let _ = events.send(Event::StartupFailed(format!("{err:#}")));
            return;
        }
    };

    let requests = Requests::new(client.request_handle(), launch.cwd);
    let session = Session {
        client,
        requests,
        events,
    };
    session.serve(commands).await;
}

struct Session {
    client: InProcessAppServerClient,
    requests: Requests,
    events: mpsc::UnboundedSender<Event>,
}

impl Session {
    async fn serve(mut self, mut commands: mpsc::UnboundedReceiver<Command>) {
        loop {
            tokio::select! {
                command = commands.recv() => {
                    let Some(command) = command else {
                        break;
                    };
                    self.dispatch(command).await;
                }
                event = self.client.next_event() => {
                    let Some(event) = event else {
                        self.emit(Event::Disconnected("the embedded app-server stopped".to_string()));
                        break;
                    };
                    self.forward(event).await;
                }
            }
        }

        self.shutdown().await;
    }

    async fn dispatch(&self, command: Command) {
        match command {
            Command::Request(request) => self.spawn_request(request),
            Command::ResolveServerRequest {
                request_id,
                response,
            } => self.resolve(request_id, response).await,
        }
    }

    fn spawn_request(&self, request: Request) {
        let requests = self.requests.clone();
        let events = self.events.clone();

        tokio::spawn(async move {
            let event = match requests.execute(request).await {
                Ok(event) => event,
                Err(message) => Some(Event::RequestFailed(message)),
            };

            if let Some(event) = event {
                let _ = events.send(event);
            }
        });
    }

    async fn resolve(&self, request_id: RequestId, response: Value) {
        if let Err(err) = self
            .client
            .resolve_server_request(request_id, response)
            .await
        {
            self.emit(Event::RequestFailed(format!(
                "failed to answer the app-server: {err}"
            )));
        }
    }

    async fn forward(&self, event: InProcessServerEvent) {
        match event {
            InProcessServerEvent::ServerNotification(notification) => {
                self.emit(Event::Notification(notification));
            }
            InProcessServerEvent::ServerRequest(request) => {
                self.route_server_request(*request).await;
            }
            InProcessServerEvent::Lagged { skipped } => {
                tracing::warn!(skipped, "codex-gui fell behind the app-server event stream");
            }
        }
    }

    async fn route_server_request(&self, request: ServerRequest) {
        match PendingApproval::from_request(&request) {
            Some(approval) => self.emit(Event::ApprovalRequested(Box::new(approval))),
            None => self.reject_unsupported(request).await,
        }
    }

    async fn reject_unsupported(&self, request: ServerRequest) {
        let error = JSONRPCErrorError {
            code: UNSUPPORTED_REQUEST_CODE,
            message: format!(
                "{} is not supported by codex-gui yet",
                method_name(&request)
            ),
            data: None,
        };

        if let Err(err) = self
            .client
            .reject_server_request(request.id().clone(), error)
            .await
        {
            tracing::warn!(%err, "failed to reject unsupported server request");
        }
    }

    fn emit(&self, event: Event) {
        let _ = self.events.send(event);
    }

    async fn shutdown(self) {
        if let Err(err) = self.client.shutdown().await {
            tracing::warn!(%err, "embedded app-server did not shut down cleanly");
        }
    }
}

fn method_name(request: &ServerRequest) -> String {
    serde_json::to_value(request)
        .ok()
        .and_then(|value| value.get("method")?.as_str().map(str::to_owned))
        .unwrap_or_else(|| "this server request".to_string())
}
