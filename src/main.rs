// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use archinalysator::commands::*;

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            analyze_project,
            get_analysis_status,
            export_analysis,
            generate_architecture_diagram,
            get_project_structure,
            validate_project_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
