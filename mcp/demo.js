#!/usr/bin/env node

// 🎯 Демонстрация ArchLens MCP Server
// Показывает все возможности без полноценного анализа

console.log('🏗️ ArchLens MCP Server - Демонстрация возможностей\n');

// Демонстрация структуры проекта
console.log('📊 СТРУКТУРА ПРОЕКТА:');
console.log('─'.repeat(50));
console.log(`{
  "status": "success",
  "structure": {
    "total_files": 45,
    "file_types": {
      ".rs": 25,
      ".ts": 12,
      ".js": 5,
      ".json": 3
    },
    "layers": ["src", "components", "utils", "api"],
    "files": [
      {
        "path": "src/main.rs",
        "extension": ".rs",
        "size": 2048
      },
      {
        "path": "src/lib.rs", 
        "extension": ".rs",
        "size": 1024
      }
    ]
  }
}\n`);

// Демонстрация AI Compact экспорта
console.log('🤖 AI COMPACT ЭКСПОРТ:');
console.log('─'.repeat(50));
console.log(`{
  "status": "success",
  "ai_compact_analysis": "🏗️ АРХИТЕКТУРНЫЙ АНАЛИЗ (AI COMPACT)\\n\\n📊 КРИТИЧЕСКИЕ ПРОБЛЕМЫ (TOP-10):\\n🔥 main.rs (complexity: 45, coupling: 8) [GOD-OBJECT]\\n🕸️ parser.rs (deps: 12, warnings: 3)\\n\\n🎯 АРХИТЕКТУРНЫЕ ПАТТЕРНЫ:\\n[CYCLE] Core → Utils → Core\\n[ORPHAN] legacy_module.rs\\n[NO-TESTS] critical_component.rs\\n\\n📈 МЕТРИКИ:\\nКомплексность: 🔥 высокая (avg: 23.4)\\nСвязность: ⚠️ проблемы (15 циклов)\\nПокрытие: 📊 67% тестами",
  "token_count": 2847,
  "compressed_ratio": "98.7%"
}\n`);

// Демонстрация генерации диаграммы
console.log('📈 ГЕНЕРАЦИЯ ДИАГРАММЫ:');
console.log('─'.repeat(50));
console.log(`{
  "status": "success",
  "diagram": "graph TD\\n    A[Core] --> B[Parser]\\n    B --> C[Analyzer]\\n    C --> D[Exporter]\\n    D --> E[UI]\\n    E --> A",
  "diagram_type": "mermaid",
  "size": 156
}\n`);

// Демонстрация полного анализа
console.log('🔍 ПОЛНЫЙ АНАЛИЗ:');
console.log('─'.repeat(50));
console.log(`{
  "status": "success",
  "analysis": {
    "project_metrics": {
      "total_components": 127,
      "total_relations": 89,
      "average_complexity": 12.3,
      "coupling_index": 0.67,
      "test_coverage": 78.5
    },
    "architectural_layers": {
      "Core": 15,
      "API": 8,
      "UI": 23,
      "Utils": 12,
      "Business": 19
    },
    "critical_issues": [
      {
        "type": "HIGH_COMPLEXITY",
        "component": "main.rs",
        "value": 45,
        "recommendation": "Разбить на меньшие функции"
      },
      {
        "type": "CIRCULAR_DEPENDENCY", 
        "components": ["parser.rs", "validator.rs"],
        "recommendation": "Создать общий интерфейс"
      }
    ]
  }
}\n`);

console.log('✨ ДОСТУПНЫЕ ИНСТРУМЕНТЫ:');
console.log('─'.repeat(50));
console.log('🔍 analyze_project - Полный анализ архитектуры');
console.log('🤖 export_ai_compact - Сжатый AI анализ (~2800 токенов)');
console.log('📊 get_project_structure - Быстрый обзор структуры');
console.log('📈 generate_diagram - Создание диаграмм (SVG, Mermaid)');
console.log('');

console.log('🎯 ПРИМЕРЫ КОМАНД ДЛЯ AI:');
console.log('─'.repeat(50));
console.log('• "Проанализируй архитектуру проекта в папке D:\\MyProject"');
console.log('• "Экспортируй архитектуру в AI Compact формат"');
console.log('• "Покажи структуру проекта"');
console.log('• "Создай Mermaid диаграмму архитектуры"');
console.log('• "Найди циклические зависимости"');
console.log('• "Покажи метрики сложности кода"');
console.log('');

console.log('🚀 ГОТОВО! MCP сервер предоставляет мощные инструменты анализа архитектуры для AI!'); 