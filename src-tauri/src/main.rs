// main.rs - ponto de entrada do Indexo
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod engine;
mod fs_ops;

use db::Database;
use std::sync::Mutex;
use tauri::Manager;

/// Estado global compartilhado entre todos os comandos Tauri.
pub struct AppState {
    pub db: Mutex<Database>,
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Abre (ou cria) o banco em ./data/profile.db, sempre ao lado do
            // executavel -- isso e o que torna o perfil do usuario portatil.
            let db = Database::open_beside_executable()
                .expect("falha ao abrir/criar o banco de perfil local");
            app.manage(AppState {
                db: Mutex::new(db),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::scan::scan_folder,
            commands::scan::scan_specific_files,
            commands::classify::classify_scanned_files,
            commands::apply::apply_organization,
            commands::apply::undo_last_apply,
            commands::profile::list_categories,
            commands::profile::create_category,
            commands::profile::rename_category,
            commands::profile::merge_categories,
            commands::profile::delete_category,
            commands::profile::clean_unused_categories,
            commands::profile::purge_auto_categories,
            commands::profile::record_user_correction,
            commands::profile::export_profile,
            commands::profile::import_profile,
            commands::profile::get_setting,
            commands::profile::save_setting,
            commands::system::open_in_explorer,
            commands::system::open_with_default_app,
            commands::system::get_file_preview,
            commands::rename::suggest_semantic_names,
            commands::rename::apply_renames,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Indexo");
}
