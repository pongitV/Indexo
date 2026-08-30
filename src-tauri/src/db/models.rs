use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: String,
    pub session_id: String,
    pub original_path: String,
    pub filename: String,
    pub extension_declared: Option<String>,
    pub extension_detected: Option<String>,
    pub size_bytes: i64,
    pub content_hash: Option<String>,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
    pub last_scanned_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRecord {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub color: Option<String>,
    pub parent_id: Option<String>,
    pub created_by: String, // "auto" | "user"
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryWithCount {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub color: Option<String>,
    pub parent_id: Option<String>,
    pub created_by: String,
    pub file_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationRule {
    pub id: String,
    pub pattern_type: String, // extension | filename_regex | content_keyword | embedding_cluster
    pub pattern_value: String,
    pub category_id: String,
    pub confidence_weight: f32,
    pub created_from: String, // learned | user
    pub hit_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCorrection {
    pub id: String,
    pub file_id: String,
    pub old_category_id: Option<String>,
    pub new_category_id: String,
    pub corrected_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveLogRecord {
    pub id: String,
    pub session_id: String,
    pub file_id: String,
    pub from_path: String,
    pub to_path: String,
    pub moved_at: String,
    pub undone: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ScanSessionRecord {
    pub id: String,
    pub root_path: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub files_scanned: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryHistoryRecord {
    pub id: String,
    pub category_id: String,
    pub old_name: String,
    pub new_name: String,
    pub changed_by: String, // 'user' | 'ai_refinement' | 'merge' | 'auto'
    pub reason: Option<String>,
    pub changed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCategoryInfo {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub created_by: String, // "auto" | "user"
    pub file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFileInfo {
    pub file_id: String,
    pub filename: String,
    pub original_path: String,
    pub category_name: String,
    pub category_color: Option<String>,
    pub size_bytes: i64,
    pub is_already_organized: bool,
    pub proposed_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRenameInfo {
    pub file_id: String,
    pub original_name: String,
    pub proposed_name: String,
    pub final_name: Option<String>,
    pub from_path: String,
    pub to_path: String,
    pub applied: bool,
    pub undone: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationSessionSummary {
    pub session_id: String,
    pub root_path: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub status: String,
    pub files_scanned: i64,
    pub files_moved_count: usize,
    pub undone_count: usize,
    pub categories_assigned: Vec<SessionCategoryInfo>,
    pub files: Vec<SessionFileInfo>,
    pub moves: Vec<MoveLogRecord>,
    pub renames: Vec<SessionRenameInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRule {
    pub id: String,
    pub name: String,
    pub condition_field: String,    // 'extension' | 'filename_contains' | 'content_contains' | 'size_greater' | 'size_smaller'
    pub condition_operator: String, // 'equals' | 'contains' | 'starts_with' | 'ends_with' | 'greater_than' | 'less_than'
    pub condition_value: String,
    pub action_type: String,        // 'move_category' | 'rename_pattern' | 'apply_tag'
    pub action_value: String,
    pub is_enabled: bool,
    pub priority: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCustomRuleInput {
    pub name: String,
    pub condition_field: String,
    pub condition_operator: String,
    pub condition_value: String,
    pub action_type: String,
    pub action_value: String,
    pub priority: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedRuleInfo {
    pub id: String,
    pub pattern_type: String,
    pub pattern_value: String,
    pub category_id: String,
    pub category_name: String,
    pub category_color: Option<String>,
    pub confidence_weight: f64,
    pub created_from: String,
    pub hit_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageCategoryStat {
    pub category_name: String,
    pub category_color: Option<String>,
    pub total_files: i64,
    pub total_bytes: i64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageAnalytics {
    pub total_organized_files: i64,
    pub total_organized_bytes: i64,
    pub total_sessions_count: i64,
    pub categories_breakdown: Vec<StorageCategoryStat>,
    pub recent_activity_dates: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateItem {
    pub path: String,
    pub filename: String,
    pub size_bytes: u64,
    pub modified_at: Option<String>,
    pub resolution: Option<String>, // ex: "1920x1080" se imagem/vídeo
    pub is_recommended_to_keep: bool,
    pub is_selected_to_keep: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub group_id: String,
    pub hash: String,
    pub size_bytes: u64,
    pub items: Vec<DuplicateItem>,
    pub potential_savings_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateResolveAction {
    pub keep_path: String,
    pub delete_or_move_paths: Vec<String>,
    pub action_type: String, // 'trash' | 'delete' | 'archive_folder'
    pub archive_folder_path: Option<String>,
}
