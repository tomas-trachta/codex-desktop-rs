use codex_app_server_protocol::ApprovalsReviewer;
use codex_app_server_protocol::AskForApproval;
use codex_gui::backend::ThreadSettings;
use codex_gui::backend::TurnOverrides;
use codex_utils_approval_presets::ApprovalPreset;
use codex_utils_approval_presets::builtin_approval_presets;

const ASK_FOR_APPROVAL_PRESET_ID: &str = "auto";

#[derive(Debug, Clone)]
pub struct ApprovalMode {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub approval_policy: AskForApproval,
    pub permissions: String,
}

pub fn modes() -> Vec<ApprovalMode> {
    builtin_approval_presets().into_iter().map(mode).collect()
}

pub fn find(id: &str) -> Option<ApprovalMode> {
    modes().into_iter().find(|mode| mode.id == id)
}

pub fn current(
    settings: Option<&ThreadSettings>,
    overrides: &TurnOverrides,
) -> Option<ApprovalMode> {
    let permissions = overrides
        .permissions
        .as_deref()
        .or_else(|| settings.and_then(|settings| settings.permission_profile.as_deref()))?;
    let approval_policy = overrides
        .approval_policy
        .as_ref()
        .or_else(|| settings.and_then(|settings| settings.approval_policy.as_ref()))?;

    modes()
        .into_iter()
        .find(|mode| mode.permissions == permissions && &mode.approval_policy == approval_policy)
}

pub fn fallback_label(settings: &ThreadSettings) -> Option<&'static str> {
    let label = match settings.approval_policy.as_ref()? {
        AskForApproval::OnRequest => match settings.approvals_reviewer {
            Some(ApprovalsReviewer::AutoReview) => "Approve for me",
            _ => "Ask for approval",
        },
        AskForApproval::UnlessTrusted => "Ask for untrusted commands",
        AskForApproval::Granular { .. } => "Custom approvals",
        AskForApproval::Never => "Full Access",
    };

    Some(label)
}

fn mode(preset: ApprovalPreset) -> ApprovalMode {
    let label = if preset.id == ASK_FOR_APPROVAL_PRESET_ID {
        "Ask for approval"
    } else {
        preset.label
    };

    ApprovalMode {
        id: preset.id,
        label,
        description: preset.description,
        approval_policy: preset.approval.into(),
        permissions: preset.active_permission_profile.id,
    }
}
