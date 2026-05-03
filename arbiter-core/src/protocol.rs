use serde::{Deserialize, Serialize};

pub const PIPE_TELEMETRY: &str = r"\\.\pipe\arbiter_telemetry";

pub const PIPE_COMMAND: &str = r"\\.\pipe\arbiter_command";

#[derive(Debug, Serialize, Deserialize)]
pub enum ForgeCommand {
    SaveDecree(crate::ledger::DecreeDef),
    SaveWards(Vec<crate::decree::WardConfig>),
    SaveSignet(crate::signet::ArbiterConfig),
    SetPaused { paused: bool },
    RemoveDecree { decree_id: String },
    RenameDecree { decree_id: String, label: String },
    ReloadWards,
    ManualRun { summons_key: String, dry_run: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    #[serde(default)]
    pub time: String,
    pub tag: String,
    pub message: String,
    pub is_error: bool,
    pub decree_id: Option<String>,
}
