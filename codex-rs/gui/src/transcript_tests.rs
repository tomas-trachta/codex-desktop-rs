use codex_app_server_protocol::FileUpdateChange;
use codex_app_server_protocol::PatchApplyStatus;
use codex_app_server_protocol::PatchChangeKind;
use codex_app_server_protocol::ThreadItem;
use codex_app_server_protocol::UserInput;
use pretty_assertions::assert_eq;

use super::Entry;
use super::Transcript;

fn user_message(id: &str, client_id: Option<&str>, text: &str) -> ThreadItem {
    ThreadItem::UserMessage {
        id: id.to_string(),
        client_id: client_id.map(str::to_string),
        content: vec![UserInput::Text {
            text: text.to_string(),
            text_elements: Vec::new(),
        }],
    }
}

fn agent_message(id: &str, text: &str) -> ThreadItem {
    ThreadItem::AgentMessage {
        id: id.to_string(),
        text: text.to_string(),
        phase: None,
        memory_citation: None,
        delivery: None,
        questions: None,
    }
}

fn user_texts(transcript: &Transcript) -> Vec<&str> {
    transcript
        .rows()
        .iter()
        .filter_map(|row| match &row.entry {
            Entry::User(text) => Some(text.as_str()),
            _ => None,
        })
        .collect()
}

#[test]
fn server_echo_replaces_pending_user_message() {
    let mut transcript = Transcript::default();
    transcript.push_pending_user_message("client-1".to_string(), "draft".to_string());

    transcript.upsert(&user_message("item-1", Some("client-1"), "hello"));
    transcript.upsert(&user_message("item-1", Some("client-1"), "hello"));

    assert_eq!(user_texts(&transcript), vec!["hello"]);
}

#[test]
fn unrelated_user_messages_are_appended() {
    let mut transcript = Transcript::default();
    transcript.push_pending_user_message("client-1".to_string(), "first".to_string());

    transcript.upsert(&user_message("item-2", None, "second"));

    assert_eq!(user_texts(&transcript), vec!["first", "second"]);
}

#[test]
fn streamed_agent_message_stays_a_single_entry() {
    let mut transcript = Transcript::default();

    transcript.upsert(&agent_message("agent-1", ""));
    transcript.append_agent_text("agent-1", "Hel");
    transcript.append_agent_text("agent-1", "lo");
    transcript.upsert(&agent_message("agent-1", "Hello"));

    assert_eq!(transcript.rows().len(), 1);
    assert!(matches!(
        transcript.rows().first().map(|row| &row.entry),
        Some(Entry::Agent(_))
    ));
}

#[test]
fn file_changes_are_found_by_item_id() {
    let change = FileUpdateChange {
        path: "src/lib.rs".to_string(),
        kind: PatchChangeKind::Add,
        diff: "+fn main() {}".to_string(),
    };
    let mut transcript = Transcript::default();

    transcript.upsert(&ThreadItem::FileChange {
        id: "patch-1".to_string(),
        changes: vec![change.clone()],
        status: PatchApplyStatus::InProgress,
    });

    assert_eq!(transcript.file_changes("patch-1"), Some(&[change][..]));
    assert_eq!(transcript.file_changes("missing"), None);
}
