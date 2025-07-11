use std::path::Path;
use std::fs;
use std::collections::HashMap;
use crate::types::*;

pub fn generate_ai_compact(project_path: &str) -> std::result::Result<String, String> {
    if !Path::new(project_path).exists() {
        return Err("Путь не существует".to_string());
    }
    
    let mut output = String::new();
    
    // Заголовок
    output.push_str("# 🏗️ AI COMPACT ARCHITECTURE ANALYSIS\n\n");
    output.push_str(&format!("**Проект:** {}\n", project_path));
    output.push_str(&format!("**Дата анализа:** {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    output.push_str(&format!("**ID анализа:** {}\n\n", uuid::Uuid::new_v4()));
    
    // Быстрая статистика
    let stats = collect_basic_stats(project_path)?;
    output.push_str("## 📊 БЫСТРАЯ СТАТИСТИКА\n");
    output.push_str(&format!("- **Всего файлов:** {}\n", stats.total_files));
    output.push_str(&format!("- **Строк кода:** {}\n", stats.total_lines));
    output.push_str(&format!("- **Типов файлов:** {}\n", stats.file_types.len()));
    output.push_str(&format!("- **Компонентов:** {}\n", stats.components));
    output.push_str(&format!("- **Связей:** {}\n", stats.connections));
    output.push_str("\n");
    
    // Критические проблемы
    let issues = analyze_critical_issues(project_path)?;
    if !issues.is_empty() {
        output.push_str("## 🚨 КРИТИЧЕСКИЕ ПРОБЛЕМЫ\n");
        for issue in issues {
            output.push_str(&format!("- **{}:** {}\n", issue.severity, issue.description));
        }
        output.push_str("\n");
    }
    
    // Архитектурные паттерны
    let patterns = detect_architectural_patterns(project_path)?;
    if !patterns.is_empty() {
        output.push_str("## 🏛️ АРХИТЕКТУРНЫЕ ПАТТЕРНЫ\n");
        for pattern in patterns {
            output.push_str(&format!("- **{}:** {} (уверенность: {}%)\n", 
                                   pattern.name, pattern.description, pattern.confidence));
        }
        output.push_str("\n");
    }
    
    // Структура проекта
    let structure = analyze_project_structure(project_path)?;
    output.push_str("## 📁 СТРУКТУРА ПРОЕКТА\n");
    output.push_str(&format!("```\n{}\n```\n\n", structure));
    
    // Ключевые модули
    let modules = analyze_key_modules(project_path)?;
    if !modules.is_empty() {
        output.push_str("## 🔧 КЛЮЧЕВЫЕ МОДУЛИ\n");
        for module in modules {
            output.push_str(&format!("- **{}** ({}): {}\n", 
                                   module.name, module.category, module.description));
        }
        output.push_str("\n");
    }
    
    // Рекомендации
    let recommendations = generate_recommendations(project_path)?;
    if !recommendations.is_empty() {
        output.push_str("## 💡 РЕКОМЕНДАЦИИ\n");
        for rec in recommendations {
            output.push_str(&format!("- **{}:** {}\n", rec.priority, rec.description));
        }
        output.push_str("\n");
    }
    
    // Метрики качества
    let quality = calculate_quality_metrics(project_path)?;
    output.push_str("## 📈 МЕТРИКИ КАЧЕСТВА\n");
    output.push_str(&format!("- **Индекс сопровождаемости:** {}/100\n", quality.maintainability));
    output.push_str(&format!("- **Цикломатическая сложность:** {}\n", quality.complexity));
    output.push_str(&format!("- **Покрытие документацией:** {}%\n", quality.documentation_coverage));
    output.push_str(&format!("- **Техдолг:** {}\n", quality.tech_debt));
    output.push_str("\n");
    
    output.push_str("---\n");
    output.push_str("*Сгенерировано ArchLens AI Compact Export*\n");
    
    Ok(output)
}

#[derive(Debug)]
struct CompactStats {
    total_files: usize,
    total_lines: usize,
    file_types: HashMap<String, usize>,
    components: usize,
    connections: usize,
}

#[derive(Debug)]
struct CriticalIssue {
    severity: String,
    description: String,
}

#[derive(Debug)]
struct ArchitecturalPattern {
    name: String,
    description: String,
    confidence: u8,
}

#[derive(Debug)]
struct KeyModule {
    name: String,
    category: String,
    description: String,
}

#[derive(Debug)]
struct Recommendation {
    priority: String,
    description: String,
}

#[derive(Debug)]
struct QualityMetrics {
    maintainability: u8,
    complexity: u8,
    documentation_coverage: u8,
    tech_debt: String,
}

fn collect_basic_stats(project_path: &str) -> std::result::Result<CompactStats, String> {
    use super::stats;
    
    let project_stats = stats::get_project_stats(project_path)?;
    
    // Подсчет компонентов (упрощенно)
    let components = project_stats.file_types.values().sum::<usize>();
    let connections = (components * 2) / 3; // Приблизительная оценка
    
    Ok(CompactStats {
        total_files: project_stats.total_files,
        total_lines: project_stats.total_lines,
        file_types: project_stats.file_types,
        components,
        connections,
    })
}

fn analyze_critical_issues(project_path: &str) -> std::result::Result<Vec<CriticalIssue>, String> {
    let mut issues = Vec::new();
    
    // Проверка больших файлов
    let large_files = find_large_files(project_path)?;
    if !large_files.is_empty() {
        issues.push(CriticalIssue {
            severity: "HIGH".to_string(),
            description: format!("Найдено {} больших файлов (>500 строк)", large_files.len()),
        });
    }
    
    // Проверка дублирования
    let duplicates = find_potential_duplicates(project_path)?;
    if !duplicates.is_empty() {
        issues.push(CriticalIssue {
            severity: "MEDIUM".to_string(),
            description: format!("Найдено {} потенциальных дублей", duplicates.len()),
        });
    }
    
    Ok(issues)
}

fn detect_architectural_patterns(project_path: &str) -> std::result::Result<Vec<ArchitecturalPattern>, String> {
    let mut patterns = Vec::new();
    
    // Проверка на MVC
    if has_mvc_structure(project_path)? {
        patterns.push(ArchitecturalPattern {
            name: "MVC".to_string(),
            description: "Модель-Вид-Контроллер".to_string(),
            confidence: 85,
        });
    }
    
    // Проверка на модульность
    if has_modular_structure(project_path)? {
        patterns.push(ArchitecturalPattern {
            name: "Modular".to_string(),
            description: "Модульная архитектура".to_string(),
            confidence: 90,
        });
    }
    
    Ok(patterns)
}

fn analyze_project_structure(project_path: &str) -> std::result::Result<String, String> {
    let mut structure = String::new();
    
    let entries = fs::read_dir(project_path)
        .map_err(|e| format!("Ошибка чтения директории: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Ошибка обработки записи: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if !should_skip_directory(name) {
                    structure.push_str(&format!("📁 {}/\n", name));
                    if let Ok(sub_structure) = analyze_subdirectory(&path, 1) {
                        structure.push_str(&sub_structure);
                    }
                }
            }
        } else {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if is_important_file(name) {
                    structure.push_str(&format!("📄 {}\n", name));
                }
            }
        }
    }
    
    Ok(structure)
}

fn analyze_subdirectory(dir_path: &Path, depth: usize) -> std::result::Result<String, String> {
    if depth > 2 {
        return Ok(String::new());
    }
    
    let mut structure = String::new();
    let indent = "  ".repeat(depth);
    
    let entries = fs::read_dir(dir_path)
        .map_err(|e| format!("Ошибка чтения поддиректории: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Ошибка обработки записи: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if !should_skip_directory(name) {
                    structure.push_str(&format!("{}📁 {}/\n", indent, name));
                }
            }
        } else {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if is_important_file(name) {
                    structure.push_str(&format!("{}📄 {}\n", indent, name));
                }
            }
        }
    }
    
    Ok(structure)
}

fn analyze_key_modules(project_path: &str) -> std::result::Result<Vec<KeyModule>, String> {
    let mut modules = Vec::new();
    
    // Анализ Rust проекта
    let cargo_toml = Path::new(project_path).join("Cargo.toml");
    if cargo_toml.exists() {
        modules.push(KeyModule {
            name: "Rust Project".to_string(),
            category: "Backend".to_string(),
            description: "Основной Rust проект".to_string(),
        });
    }
    
    // Анализ главного модуля
    let main_rs = Path::new(project_path).join("src/main.rs");
    if main_rs.exists() {
        modules.push(KeyModule {
            name: "Main Entry".to_string(),
            category: "Core".to_string(),
            description: "Точка входа приложения".to_string(),
        });
    }
    
    Ok(modules)
}

fn generate_recommendations(project_path: &str) -> std::result::Result<Vec<Recommendation>, String> {
    let mut recommendations = Vec::new();
    
    let large_files = find_large_files(project_path)?;
    if !large_files.is_empty() {
        recommendations.push(Recommendation {
            priority: "HIGH".to_string(),
            description: "Разделить большие файлы на меньшие модули".to_string(),
        });
    }
    
    let test_coverage = estimate_test_coverage(project_path)?;
    if test_coverage < 50 {
        recommendations.push(Recommendation {
            priority: "MEDIUM".to_string(),
            description: "Увеличить покрытие тестами".to_string(),
        });
    }
    
    Ok(recommendations)
}

fn calculate_quality_metrics(project_path: &str) -> std::result::Result<QualityMetrics, String> {
    let maintainability = estimate_maintainability(project_path)?;
    let complexity = estimate_complexity(project_path)?;
    let documentation = estimate_documentation_coverage(project_path)?;
    let tech_debt = estimate_tech_debt(project_path)?;
    
    Ok(QualityMetrics {
        maintainability,
        complexity,
        documentation_coverage: documentation,
        tech_debt,
    })
}

// Вспомогательные функции
fn find_large_files(project_path: &str) -> std::result::Result<Vec<String>, String> {
    let mut large_files = Vec::new();
    scan_for_large_files(Path::new(project_path), &mut large_files)?;
    Ok(large_files)
}

fn scan_for_large_files(dir: &Path, large_files: &mut Vec<String>) -> std::result::Result<(), String> {
    if !dir.is_dir() {
        return Ok(());
    }
    
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Ошибка чтения директории: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Ошибка обработки записи: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if !should_skip_directory(name) {
                    scan_for_large_files(&path, large_files)?;
                }
            }
        } else {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if is_code_file(ext) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if content.lines().count() > 500 {
                            large_files.push(path.display().to_string());
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

fn find_potential_duplicates(project_path: &str) -> std::result::Result<Vec<String>, String> {
    let mut duplicates = Vec::new();
    
    // Простая проверка на дублирование имен файлов
    let entries = fs::read_dir(project_path)
        .map_err(|e| format!("Ошибка чтения директории: {}", e))?;
    
    let mut file_names: HashMap<String, usize> = HashMap::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("Ошибка обработки записи: {}", e))?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.contains("backup") || name.contains("copy") || name.contains("_old") {
                    duplicates.push(name.to_string());
                }
            }
        }
    }
    
    Ok(duplicates)
}

fn has_mvc_structure(project_path: &str) -> std::result::Result<bool, String> {
    let has_models = Path::new(project_path).join("models").exists() || 
                     Path::new(project_path).join("src/models").exists();
    let has_views = Path::new(project_path).join("views").exists() || 
                    Path::new(project_path).join("src/views").exists();
    let has_controllers = Path::new(project_path).join("controllers").exists() || 
                          Path::new(project_path).join("src/controllers").exists();
    
    Ok(has_models && has_views && has_controllers)
}

fn has_modular_structure(project_path: &str) -> std::result::Result<bool, String> {
    let src_dir = Path::new(project_path).join("src");
    if !src_dir.exists() {
        return Ok(false);
    }
    
    let entries = fs::read_dir(src_dir)
        .map_err(|e| format!("Ошибка чтения src директории: {}", e))?;
    
    let mut module_count = 0;
    for entry in entries {
        let entry = entry.map_err(|e| format!("Ошибка обработки записи: {}", e))?;
        if entry.path().is_dir() {
            module_count += 1;
        }
    }
    
    Ok(module_count >= 3)
}

fn estimate_test_coverage(project_path: &str) -> std::result::Result<u8, String> {
    let mut total_files = 0;
    let mut test_files = 0;
    
    scan_for_test_files(Path::new(project_path), &mut total_files, &mut test_files)?;
    
    if total_files == 0 {
        return Ok(0);
    }
    
    Ok(((test_files * 100) / total_files) as u8)
}

fn scan_for_test_files(dir: &Path, total_files: &mut usize, test_files: &mut usize) -> std::result::Result<(), String> {
    if !dir.is_dir() {
        return Ok(());
    }
    
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("Ошибка чтения директории: {}", e))?;
    
    for entry in entries {
        let entry = entry.map_err(|e| format!("Ошибка обработки записи: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if !should_skip_directory(name) {
                    scan_for_test_files(&path, total_files, test_files)?;
                }
            }
        } else {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if is_code_file(&name.to_lowercase()) {
                    *total_files += 1;
                    if name.contains("test") || name.contains("spec") {
                        *test_files += 1;
                    }
                }
            }
        }
    }
    
    Ok(())
}

fn estimate_maintainability(project_path: &str) -> std::result::Result<u8, String> {
    let large_files = find_large_files(project_path)?;
    let duplicates = find_potential_duplicates(project_path)?;
    
    let mut score = 100;
    
    // Штрафы за проблемы
    if !large_files.is_empty() {
        score -= (large_files.len() * 10).min(30) as u8;
    }
    
    if !duplicates.is_empty() {
        score -= (duplicates.len() * 5).min(20) as u8;
    }
    
    Ok(score.max(0))
}

fn estimate_complexity(project_path: &str) -> std::result::Result<u8, String> {
    let stats = collect_basic_stats(project_path)?;
    
    // Упрощенная оценка сложности
    let avg_lines_per_file = if stats.total_files > 0 {
        stats.total_lines / stats.total_files
    } else {
        0
    };
    
    let complexity = (avg_lines_per_file / 10).min(10) as u8;
    Ok(complexity)
}

fn estimate_documentation_coverage(project_path: &str) -> std::result::Result<u8, String> {
    let readme_exists = Path::new(project_path).join("README.md").exists() ||
                        Path::new(project_path).join("readme.md").exists() ||
                        Path::new(project_path).join("README.txt").exists();
    
    let docs_dir_exists = Path::new(project_path).join("docs").exists() ||
                          Path::new(project_path).join("doc").exists();
    
    let mut score = 0;
    if readme_exists { score += 50; }
    if docs_dir_exists { score += 30; }
    
    Ok(score.min(100))
}

fn estimate_tech_debt(project_path: &str) -> std::result::Result<String, String> {
    let large_files = find_large_files(project_path)?;
    let duplicates = find_potential_duplicates(project_path)?;
    
    let issues = large_files.len() + duplicates.len();
    
    match issues {
        0 => Ok("Низкий".to_string()),
        1..=5 => Ok("Средний".to_string()),
        _ => Ok("Высокий".to_string()),
    }
}

fn should_skip_directory(dir_name: &str) -> bool {
    matches!(dir_name, "node_modules" | "target" | ".git" | ".svn" | "dist" | "build" | 
                      ".next" | ".nuxt" | "coverage" | "__pycache__" | "backup")
}

fn is_important_file(filename: &str) -> bool {
    matches!(filename, "Cargo.toml" | "package.json" | "README.md" | "main.rs" | 
                      "lib.rs" | "index.js" | "index.ts" | "app.js" | "app.py" | 
                      "main.py" | "setup.py" | "requirements.txt" | "Dockerfile" | 
                      "docker-compose.yml" | ".gitignore")
}

fn is_code_file(ext: &str) -> bool {
    matches!(ext, "rs" | "js" | "ts" | "jsx" | "tsx" | "py" | "java" | "cpp" | "c" | "h" | 
                 "hpp" | "cs" | "php" | "rb" | "go" | "swift" | "kt" | "scala" | "clj" | 
                 "hs" | "ml" | "fs" | "dart" | "lua" | "r" | "m" | "mm" | "vb" | "pas" | 
                 "pl" | "pm" | "sh" | "bash" | "zsh" | "fish" | "ps1" | "psm1" | "psd1")
} 