use crate::db::models::{CreateCustomRuleInput, CustomRule, LearnedRuleInfo, StorageAnalytics};
use tauri::State;

#[tauri::command]
pub async fn create_custom_rule(
    state: State<'_, crate::AppState>,
    input: CreateCustomRuleInput,
) -> Result<CustomRule, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.create_custom_rule(input).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_custom_rules(state: State<'_, crate::AppState>) -> Result<Vec<CustomRule>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.list_custom_rules().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_custom_rule(
    state: State<'_, crate::AppState>,
    rule: CustomRule,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.update_custom_rule(rule).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_custom_rule(
    state: State<'_, crate::AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_custom_rule(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_custom_rule(
    state: State<'_, crate::AppState>,
    id: String,
    is_enabled: bool,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.toggle_custom_rule(&id, is_enabled).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_all_learned_rules(state: State<'_, crate::AppState>) -> Result<Vec<LearnedRuleInfo>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.list_all_learned_rules().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_learned_rule(
    state: State<'_, crate::AppState>,
    id: String,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_learned_rule(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_storage_analytics(state: State<'_, crate::AppState>) -> Result<StorageAnalytics, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_storage_analytics().map_err(|e| e.to_string())
}
