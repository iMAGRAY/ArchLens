// Парсер командной строки

use std::env;

/// Команды CLI
#[derive(Debug, Clone)]
pub enum CliCommand {
    Analyze {
        project_path: String,
        verbose: bool,
        include_tests: bool,
        deep: bool,
    },
    Export {
        project_path: String,
        format: ExportFormat,
        output: Option<String>,
        options: ExportOptions,
    },
    Structure {
        project_path: String,
        max_depth: Option<usize>,
        show_metrics: bool,
    },
    Diagram {
        project_path: String,
        diagram_type: DiagramType,
        output: Option<String>,
        include_metrics: bool,
    },
    Version,
    Help,
}

/// Форматы экспорта
#[derive(Debug, Clone)]
pub enum ExportFormat {
    AiCompact,
    Json,
    Markdown,
    Html,
}

/// Типы диаграмм
#[derive(Debug, Clone)]
pub enum DiagramType {
    Mermaid,
    Dot,
    Svg,
}

/// Опции экспорта
#[derive(Debug, Clone, Default)]
pub struct ExportOptions {
    pub focus_critical_only: bool,
    pub include_diff_analysis: bool,
    pub include_metrics: bool,
}

/// Парсинг аргументов командной строки
pub fn parse_args() -> Result<CliCommand, String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Ok(CliCommand::Help);
    }

    let mut parser = ArgParser::new(args);
    parser.parse()
}

/// Парсер аргументов
struct ArgParser {
    args: Vec<String>,
    pos: usize,
}

impl ArgParser {
    fn new(args: Vec<String>) -> Self {
        Self { args, pos: 1 } // Пропускаем имя программы
    }

    fn parse(&mut self) -> Result<CliCommand, String> {
        let command = self
            .current()
            .ok_or_else(|| "Не указана команда".to_string())?;

        match command.as_str() {
            "analyze" => self.parse_analyze(),
            "export" => self.parse_export(),
            "structure" => self.parse_structure(),
            "diagram" => self.parse_diagram(),
            "version" | "--version" | "-V" => Ok(CliCommand::Version),
            "help" | "--help" | "-h" => Ok(CliCommand::Help),
            _ => Err(format!("Неизвестная команда: {}", command)),
        }
    }

    fn parse_analyze(&mut self) -> Result<CliCommand, String> {
        let project_path = self.current().map(|s| s.clone());
        self.advance();

        let mut verbose = false;
        let mut include_tests = false;
        let mut deep = false;

        // Парсим флаги
        while let Some(arg) = self.current() {
            match arg.as_str() {
                "--verbose" | "-v" => verbose = true,
                "--include-tests" => include_tests = true,
                "--deep" => deep = true,
                _ => break,
            }
            self.advance();
        }

        Ok(CliCommand::Analyze {
            project_path: project_path.unwrap_or_else(|| {
                crate::get_default_project_path()
                    .to_string_lossy()
                    .to_string()
            }),
            verbose,
            include_tests,
            deep,
        })
    }

    fn parse_export(&mut self) -> Result<CliCommand, String> {
        let project_path = self.current().map(|s| s.clone());
        self.advance();

        let format_str = self
            .current()
            .ok_or_else(|| "Не указан формат экспорта".to_string())?;

        let format = match format_str.as_str() {
            "ai_compact" | "ai-compact" => ExportFormat::AiCompact,
            "json" => ExportFormat::Json,
            "markdown" | "md" => ExportFormat::Markdown,
            "html" => ExportFormat::Html,
            _ => return Err(format!("Неподдерживаемый формат: {}", format_str)),
        };

        self.advance();

        let mut output = None;
        let mut options = ExportOptions::default();

        // Парсим оставшиеся аргументы
        while let Some(arg) = self.current() {
            match arg.as_str() {
                "--output" | "-o" => {
                    self.advance();
                    output = self.current().cloned();
                    if output.is_some() {
                        self.advance();
                    }
                }
                "--critical-only" => {
                    options.focus_critical_only = true;
                    self.advance();
                }
                "--include-diff" => {
                    options.include_diff_analysis = true;
                    self.advance();
                }
                "--include-metrics" => {
                    options.include_metrics = true;
                    self.advance();
                }
                _ => {
                    // Если не флаг, считаем это выходным файлом
                    if output.is_none() && !arg.starts_with("-") {
                        output = Some(arg.clone());
                    }
                    self.advance();
                }
            }
        }

        Ok(CliCommand::Export {
            project_path: project_path.unwrap_or_else(|| {
                crate::get_default_project_path()
                    .to_string_lossy()
                    .to_string()
            }),
            format,
            output,
            options,
        })
    }

    fn parse_structure(&mut self) -> Result<CliCommand, String> {
        let project_path = self.current().map(|s| s.clone());
        self.advance();

        let mut max_depth = None;
        let mut show_metrics = false;

        while let Some(arg) = self.current() {
            match arg.as_str() {
                "--max-depth" => {
                    self.advance();
                    if let Some(depth_str) = self.current() {
                        max_depth = Some(
                            depth_str
                                .parse()
                                .map_err(|_| "Неверное значение для --max-depth")?,
                        );
                        self.advance();
                    }
                }
                "--show-metrics" => {
                    show_metrics = true;
                    self.advance();
                }
                _ => break,
            }
        }

        Ok(CliCommand::Structure {
            project_path: project_path.unwrap_or_else(|| {
                crate::get_default_project_path()
                    .to_string_lossy()
                    .to_string()
            }),
            max_depth,
            show_metrics,
        })
    }

    fn parse_diagram(&mut self) -> Result<CliCommand, String> {
        let project_path = self.current().map(|s| s.clone());
        self.advance();

        let diagram_type_str = self
            .current()
            .ok_or_else(|| "Не указан тип диаграммы".to_string())?;

        let diagram_type = match diagram_type_str.as_str() {
            "mermaid" => DiagramType::Mermaid,
            "dot" => DiagramType::Dot,
            "svg" => DiagramType::Svg,
            _ => {
                return Err(format!(
                    "Неподдерживаемый тип диаграммы: {}",
                    diagram_type_str
                ))
            }
        };

        self.advance();

        let mut output = None;
        let mut include_metrics = false;

        while let Some(arg) = self.current() {
            match arg.as_str() {
                "--output" | "-o" => {
                    self.advance();
                    output = self.current().cloned();
                    if output.is_some() {
                        self.advance();
                    }
                }
                "--include-metrics" => {
                    include_metrics = true;
                    self.advance();
                }
                _ => {
                    if output.is_none() && !arg.starts_with("-") {
                        output = Some(arg.clone());
                    }
                    self.advance();
                }
            }
        }

        Ok(CliCommand::Diagram {
            project_path: project_path.unwrap_or_else(|| {
                crate::get_default_project_path()
                    .to_string_lossy()
                    .to_string()
            }),
            diagram_type,
            output,
            include_metrics,
        })
    }

    fn current(&self) -> Option<&String> {
        self.args.get(self.pos)
    }

    fn advance(&mut self) {
        self.pos += 1;
    }
}
