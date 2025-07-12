<div align="center">

![ArchLens Logo](icon.svg)

# ArchLens

**Intelligent Code Architecture Analyzer with AI-Powered MCP Server**  
**Интеллектуальный анализатор архитектуры кода с AI-сервером MCP**

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/Build-Passing-green.svg)]()
[![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-purple.svg)](https://modelcontextprotocol.io/)

[English](#english) • [Русский](#русский)

</div>

---

## English

### ▶ What is ArchLens?

ArchLens is a powerful Rust-based code architecture analyzer that provides comprehensive insights into your codebase structure, quality metrics, and architectural patterns. It comes with an integrated MCP (Model Context Protocol) server for seamless AI editor integration.

### ★ Key Features

| Feature | Description |
|---------|-------------|
| **▣ Architecture Analysis** | Deep analysis of code structure, dependencies, and patterns |
| **▦ Quality Metrics** | Cyclomatic complexity, technical debt, code smells detection |
| **◉ AI-Powered Export** | ~2800 token AI-ready analysis for LLM consumption |
| **◈ Visual Diagrams** | Mermaid-based architecture diagrams with problem highlighting |
| **⚙ MCP Integration** | Native support for Cursor, VSCode, Claude, and other AI editors |
| **⚡ Performance** | Fast Rust-based analysis with parallel processing |

### ▲ Quick Start

#### Prerequisites
- Rust 1.70+ 
- Node.js 18+ (for MCP server)
- Windows/macOS/Linux

#### Installation
```bash
# Clone repository
git clone https://github.com/yourusername/archlens
cd archlens

# Build CLI tool
cargo build --release

# Setup MCP server
cd mcp
npm install
```

#### Basic Usage
```bash
# Analyze project structure
./target/release/archlens analyze /path/to/project

# Export AI-ready analysis
./target/release/archlens export /path/to/project ai_compact

# Generate architecture diagram
./target/release/archlens diagram /path/to/project mermaid
```

### ⚙ MCP Server Integration

ArchLens includes a powerful MCP server for AI editor integration:

#### Available Tools
| Tool | Purpose | Output |
|------|---------|---------|
| `analyze_project` | Quick project overview | Statistics & risk assessment |
| `export_ai_compact` | Comprehensive AI analysis | ~2800 tokens of detailed insights |
| `get_project_structure` | Hierarchical structure | File organization & metrics |
| `generate_diagram` | Visual architecture | Mermaid diagrams with problems |

#### Cursor Configuration
Add to `.cursor/mcp_settings.json`:
```json
{
  "mcpServers": {
    "archlens": {
      "command": "node",
      "args": ["./path/to/archlens/mcp/archlens_mcp_server.cjs"],
      "env": {
        "ARCHLENS_DEBUG": "false",
        "ARCHLENS_AUTO_ELEVATE": "true"
      }
    }
  }
}
```

### ◆ Architecture

```
archlens/
├── src/                    # Core Rust analyzer
│   ├── cli/               # Command-line interface
│   ├── enrichment/        # Code quality analysis
│   └── ...
├── mcp/                   # MCP server implementation
│   ├── archlens_mcp_server.cjs
│   └── package.json
└── target/release/        # Compiled binaries
```

### ◉ Supported Languages

✅ **Fully Supported:** Rust, TypeScript, JavaScript, Python, Java, C#  
◐ **Partial Support:** C++, Go, PHP, Ruby  
◯ **Planned:** Swift, Kotlin, Dart

### ◈ Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

### ◐ License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Русский

### ▶ Что такое ArchLens?

ArchLens — это мощный анализатор архитектуры кода на Rust, который предоставляет комплексную информацию о структуре кодовой базы, метриках качества и архитектурных паттернах. Включает интегрированный MCP-сервер для бесшовной работы с AI-редакторами.

### ★ Ключевые возможности

| Функция | Описание |
|---------|----------|
| **▣ Анализ архитектуры** | Глубокий анализ структуры кода, зависимостей и паттернов |
| **▦ Метрики качества** | Цикломатическая сложность, технический долг, обнаружение запахов кода |
| **◉ AI-экспорт** | ~2800 токенов анализа для ИИ-потребления |
| **◈ Визуальные диаграммы** | Mermaid-диаграммы архитектуры с выделением проблем |
| **⚙ MCP-интеграция** | Нативная поддержка Cursor, VSCode, Claude и других AI-редакторов |
| **⚡ Производительность** | Быстрый анализ на Rust с параллельной обработкой |

### ▲ Быстрый старт

#### Требования
- Rust 1.70+
- Node.js 18+ (для MCP-сервера)
- Windows/macOS/Linux

#### Установка
```bash
# Клонировать репозиторий
git clone https://github.com/yourusername/archlens
cd archlens

# Собрать CLI-инструмент
cargo build --release

# Настроить MCP-сервер
cd mcp
npm install
```

#### Базовое использование
```bash
# Анализ структуры проекта
./target/release/archlens analyze /path/to/project

# Экспорт AI-готового анализа
./target/release/archlens export /path/to/project ai_compact

# Генерация диаграммы архитектуры
./target/release/archlens diagram /path/to/project mermaid
```

### ⚙ Интеграция с MCP-сервером

ArchLens включает мощный MCP-сервер для интеграции с AI-редакторами:

#### Доступные инструменты
| Инструмент | Назначение | Выход |
|------------|------------|--------|
| `analyze_project` | Быстрый обзор проекта | Статистика и оценка рисков |
| `export_ai_compact` | Комплексный AI-анализ | ~2800 токенов детального анализа |
| `get_project_structure` | Иерархическая структура | Организация файлов и метрики |
| `generate_diagram` | Визуальная архитектура | Mermaid-диаграммы с проблемами |

#### Конфигурация для Cursor
Добавить в `.cursor/mcp_settings.json`:
```json
{
  "mcpServers": {
    "archlens": {
      "command": "node",
      "args": ["./path/to/archlens/mcp/archlens_mcp_server.cjs"],
      "env": {
        "ARCHLENS_DEBUG": "false",
        "ARCHLENS_AUTO_ELEVATE": "true"
      }
    }
  }
}
```

### ◆ Архитектура

```
archlens/
├── src/                    # Основной анализатор на Rust
│   ├── cli/               # Интерфейс командной строки
│   ├── enrichment/        # Анализ качества кода
│   └── ...
├── mcp/                   # Реализация MCP-сервера
│   ├── archlens_mcp_server.cjs
│   └── package.json
└── target/release/        # Скомпилированные бинарники
```

### ◉ Поддерживаемые языки

✅ **Полная поддержка:** Rust, TypeScript, JavaScript, Python, Java, C#  
◐ **Частичная поддержка:** C++, Go, PHP, Ruby  
◯ **Планируется:** Swift, Kotlin, Dart

### ◈ Участие в разработке

1. Сделать форк репозитория
2. Создать ветку функции (`git checkout -b feature/amazing-feature`)
3. Зафиксировать изменения (`git commit -m 'Add amazing feature'`)
4. Отправить в ветку (`git push origin feature/amazing-feature`)
5. Открыть Pull Request

### ◐ Лицензия

Этот проект лицензирован под лицензией MIT - подробности в файле [LICENSE](LICENSE).

---

<div align="center">

**Made with ♥ by the ArchLens Team**

[★ Star us on GitHub](https://github.com/yourusername/archlens) • 
[◉ Report Issues](https://github.com/yourusername/archlens/issues) • 
[◈ Discussions](https://github.com/yourusername/archlens/discussions)

</div> 