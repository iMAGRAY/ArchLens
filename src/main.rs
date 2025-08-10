// Обеспечиваем работу как CLI, так и GUI
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use archlens::cli;
#[cfg(feature = "gui")]
use archlens::commands::*;
use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    // Если есть аргументы командной строки, запускаем CLI режим
    if args.len() > 1 && args[1] != "--tauri" {
        match cli::run().await {
            Ok(_) => std::process::exit(0),
            Err(e) => {
                eprintln!("❌ Ошибка CLI: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Иначе запускаем GUI режим (только когда собрано с фичей `gui`)
    #[cfg(feature = "gui")]
    {
        tauri::Builder::default()
            .manage(AppState::default())
            .invoke_handler(tauri::generate_handler![
                analyze_project,
                get_analysis_status,
                export_analysis,
                generate_architecture_diagram,
                generate_svg_architecture_diagram,
                get_project_structure,
                validate_project_path
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }

    #[cfg(not(feature = "gui"))]
    {
        eprintln!("GUI not built. Re-run with --features gui to enable Tauri.");
    }
}
