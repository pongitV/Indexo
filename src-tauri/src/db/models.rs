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
pub struct OrganizationSessionSummary {
    pub session_id: String,
    pub root_path: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub status: String,
    pub files_scanned: i64,
    pub files_moved_count: usize,
    pub undone_count: usize,
    pub categories_assigned: Vec<String>,
    pub moves: Vec<MoveLogRecord>,
}
