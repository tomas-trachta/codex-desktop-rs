use std::collections::HashMap;

use codex_app_server_protocol::CommandAction;
use codex_app_server_protocol::CommandExecutionStatus;
use codex_app_server_protocol::FileUpdateChange;
use codex_app_server_protocol::PatchApplyStatus;
use codex_app_server_protocol::ThreadItem;
use codex_app_server_protocol::Turn;
use codex_app_server_protocol::UserInput;
use iced::widget::markdown;

#[derive(Default)]
pub struct Transcript {
    rows: Vec<Row>,
    positions: HashMap<String, usize>,
}

pub struct Row {
    pub key: String,
    pub entry: Entry,
}

pub enum Entry {
    User(String),
    Agent(AgentEntry),
    Reasoning(String),
    Command(CommandEntry),
    FileChange(FileChangeEntry),
    ToolCall(String),
    Notice(String),
    TurnSummary { duration_ms: i64 },
}

pub struct AgentEntry {
    pub content: markdown::Content,
    pub text: String,
}

pub struct CommandEntry {
    pub command: String,
    pub actions: Vec<CommandAction>,
    pub output: String,
    pub status: CommandExecutionStatus,
    pub exit_code: Option<i32>,
}

pub struct FileChangeEntry {
    pub changes: Vec<FileUpdateChange>,
    pub status: PatchApplyStatus,
}

impl Transcript {
    pub fn from_turns(turns: &[Turn]) -> Self {
        let mut transcript = Self::default();
        for turn in turns {
            transcript.apply_turn(turn);
        }
        transcript
    }

    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    pub fn file_changes(&self, item_id: &str) -> Option<&[FileUpdateChange]> {
        match self.entry(item_id) {
            Some(Entry::FileChange(entry)) => Some(&entry.changes),
            _ => None,
        }
    }

    pub fn push_pending_user_message(&mut self, client_message_id: String, text: String) {
        self.insert(client_message_id, Entry::User(text));
    }

    pub fn push_notice(&mut self, text: String) {
        let key = format!("notice-{}", self.rows.len());
        self.insert(key, Entry::Notice(text));
    }

    pub fn push_turn_summary(&mut self, turn_id: &str, duration_ms: Option<i64>) {
        if let Some(duration_ms) = duration_ms {
            self.insert(
                format!("turn-{turn_id}"),
                Entry::TurnSummary { duration_ms },
            );
        }
    }

    pub fn upsert(&mut self, item: &ThreadItem) {
        let Some(entry) = Entry::from_item(item) else {
            return;
        };

        self.adopt_client_message_id(item);
        self.insert(item.id().to_string(), entry);
    }

    pub fn append_agent_text(&mut self, item_id: &str, delta: &str) {
        if let Some(Entry::Agent(agent)) = self.entry_mut(item_id) {
            agent.push_str(delta);
            return;
        }

        self.insert(item_id.to_string(), Entry::Agent(AgentEntry::parse(delta)));
    }

    pub fn append_reasoning(&mut self, item_id: &str, delta: &str) {
        if let Some(Entry::Reasoning(summary)) = self.entry_mut(item_id) {
            summary.push_str(delta);
            return;
        }

        self.insert(item_id.to_string(), Entry::Reasoning(delta.to_string()));
    }

    pub fn append_command_output(&mut self, item_id: &str, delta: &str) {
        if let Some(Entry::Command(command)) = self.entry_mut(item_id) {
            command.output.push_str(delta);
        }
    }

    fn apply_turn(&mut self, turn: &Turn) {
        for item in &turn.items {
            self.upsert(item);
        }

        if let Some(error) = &turn.error {
            self.push_notice(error.message.clone());
        }

        self.push_turn_summary(&turn.id, turn.duration_ms);
    }

    fn adopt_client_message_id(&mut self, item: &ThreadItem) {
        let ThreadItem::UserMessage {
            id,
            client_id: Some(client_id),
            ..
        } = item
        else {
            return;
        };

        if let Some(position) = self.positions.remove(client_id) {
            self.positions.insert(id.clone(), position);
        }
    }

    fn insert(&mut self, key: String, entry: Entry) {
        if let Some(existing) = self.entry_mut(&key) {
            *existing = entry;
            return;
        }

        self.positions.insert(key.clone(), self.rows.len());
        self.rows.push(Row { key, entry });
    }

    fn entry(&self, key: &str) -> Option<&Entry> {
        let position = *self.positions.get(key)?;
        self.rows.get(position).map(|row| &row.entry)
    }

    fn entry_mut(&mut self, key: &str) -> Option<&mut Entry> {
        let position = *self.positions.get(key)?;
        self.rows.get_mut(position).map(|row| &mut row.entry)
    }
}

impl Entry {
    fn from_item(item: &ThreadItem) -> Option<Self> {
        match item {
            ThreadItem::UserMessage { content, .. } => Some(Self::User(user_text(content))),
            ThreadItem::AgentMessage { text, .. } => Some(Self::Agent(AgentEntry::parse(text))),
            ThreadItem::Reasoning { summary, .. } => Some(Self::Reasoning(summary.join("\n\n"))),
            ThreadItem::CommandExecution {
                command,
                command_actions,
                aggregated_output,
                status,
                exit_code,
                ..
            } => Some(Self::Command(CommandEntry {
                command: command.clone(),
                actions: command_actions.clone(),
                output: aggregated_output.clone().unwrap_or_default(),
                status: status.clone(),
                exit_code: *exit_code,
            })),
            ThreadItem::FileChange {
                changes, status, ..
            } => Some(Self::FileChange(FileChangeEntry {
                changes: changes.clone(),
                status: status.clone(),
            })),
            ThreadItem::McpToolCall { server, tool, .. } => {
                Some(Self::ToolCall(format!("{server} / {tool}")))
            }
            _ => None,
        }
    }
}

impl AgentEntry {
    fn parse(text: &str) -> Self {
        Self {
            content: markdown::Content::parse(text),
            text: text.to_string(),
        }
    }

    fn push_str(&mut self, delta: &str) {
        self.content.push_str(delta);
        self.text.push_str(delta);
    }
}

fn user_text(content: &[UserInput]) -> String {
    content
        .iter()
        .filter_map(user_input_text)
        .collect::<Vec<_>>()
        .join("\n")
}

fn user_input_text(input: &UserInput) -> Option<String> {
    match input {
        UserInput::Text { text, .. } => Some(text.clone()),
        UserInput::LocalImage { path, .. } => Some(format!("[image: {}]", path.display())),
        UserInput::Image { .. } => Some("[image]".to_string()),
        _ => None,
    }
}

#[cfg(test)]
#[path = "transcript_tests.rs"]
mod tests;
