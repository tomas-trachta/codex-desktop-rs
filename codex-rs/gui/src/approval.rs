use std::path::PathBuf;

use codex_app_server_protocol::CommandExecutionApprovalDecision;
use codex_app_server_protocol::CommandExecutionRequestApprovalParams;
use codex_app_server_protocol::CommandExecutionRequestApprovalResponse;
use codex_app_server_protocol::FileChangeApprovalDecision;
use codex_app_server_protocol::FileChangeRequestApprovalParams;
use codex_app_server_protocol::FileChangeRequestApprovalResponse;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::ServerRequest;
use codex_utils_path_uri::LegacyAppPathString;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct PendingApproval {
    pub request_id: RequestId,
    pub thread_id: String,
    pub item_id: String,
    pub reason: Option<String>,
    pub subject: ApprovalSubject,
}

#[derive(Debug, Clone)]
pub enum ApprovalSubject {
    Command {
        command: Option<String>,
        cwd: Option<String>,
    },
    FileChange {
        grant_root: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum Decision {
    Accept,
    AcceptForSession,
    Decline,
}

impl PendingApproval {
    pub fn from_request(request: &ServerRequest) -> Option<Self> {
        match request {
            ServerRequest::CommandExecutionRequestApproval { request_id, params } => {
                Some(Self::for_command(request_id, params))
            }
            ServerRequest::FileChangeRequestApproval { request_id, params } => {
                Some(Self::for_file_change(request_id, params))
            }
            _ => None,
        }
    }

    pub fn response(&self, decision: Decision) -> Value {
        match self.subject {
            ApprovalSubject::Command { .. } => to_json(CommandExecutionRequestApprovalResponse {
                decision: command_decision(decision),
            }),
            ApprovalSubject::FileChange { .. } => to_json(FileChangeRequestApprovalResponse {
                decision: file_change_decision(decision),
            }),
        }
    }

    fn for_command(request_id: &RequestId, params: &CommandExecutionRequestApprovalParams) -> Self {
        Self {
            request_id: request_id.clone(),
            thread_id: params.thread_id.clone(),
            item_id: params.item_id.clone(),
            reason: params.reason.clone(),
            subject: ApprovalSubject::Command {
                command: params.command.clone(),
                cwd: params.cwd.as_ref().map(LegacyAppPathString::render_for_ui),
            },
        }
    }

    fn for_file_change(request_id: &RequestId, params: &FileChangeRequestApprovalParams) -> Self {
        Self {
            request_id: request_id.clone(),
            thread_id: params.thread_id.clone(),
            item_id: params.item_id.clone(),
            reason: params.reason.clone(),
            subject: ApprovalSubject::FileChange {
                grant_root: params.grant_root.clone(),
            },
        }
    }
}

fn command_decision(decision: Decision) -> CommandExecutionApprovalDecision {
    match decision {
        Decision::Accept => CommandExecutionApprovalDecision::Accept,
        Decision::AcceptForSession => CommandExecutionApprovalDecision::AcceptForSession,
        Decision::Decline => CommandExecutionApprovalDecision::Decline,
    }
}

fn file_change_decision(decision: Decision) -> FileChangeApprovalDecision {
    match decision {
        Decision::Accept => FileChangeApprovalDecision::Accept,
        Decision::AcceptForSession => FileChangeApprovalDecision::AcceptForSession,
        Decision::Decline => FileChangeApprovalDecision::Decline,
    }
}

fn to_json(response: impl Serialize) -> Value {
    serde_json::to_value(response).unwrap_or_else(|err| {
        tracing::error!(%err, "failed to encode approval response");
        Value::Null
    })
}
