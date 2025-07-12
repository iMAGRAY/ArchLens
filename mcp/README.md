# 🏗️ ArchLens MCP Server v1.0.2

**Интеллектуальный анализ архитектуры кода для AI редакторов (Cursor, VSCode, Claude)**

## ✅ Статус: ГОТОВ К РАБОТЕ

Все 4 MCP инструмента успешно протестированы и работают:
- ✅ `export_ai_compact` - Полный AI анализ (~2800 токенов)
- ✅ `analyze_project` - Краткий анализ проекта
- ✅ `get_project_structure` - Структура проекта с проблемами
- ✅ `generate_diagram` - Mermaid диаграммы архитектуры

## 🚀 Быстрый старт

### 1. Сборка ArchLens
```bash
cargo build --release
```

### 2. Установка зависимостей MCP
```bash
cd mcp
npm install
```

### 3. Копирование бинарника
```bash
npm run update-binary
```

### 4. Тестирование
```bash
node test_all_tools.js
```

## 🛠️ Конфигурация в Cursor

Добавьте в `.cursor/mcp_settings.json`:

```json
{
  "mcpServers": {
    "archlens": {
      "command": "node",
      "args": ["./path/to/archlens_mcp_server.cjs"],
      "env": {
        "NODE_ENV": "production"
      }
    }
  }
}
```

## 📊 Возможности MCP инструментов

### 🤖 `export_ai_compact`
**Полный анализ архитектуры для AI (~2800 токенов)**
- Code Smells (20+ типов): длинные методы, магические числа, дублирование
- SOLID принципы и их нарушения
- Архитектурные антипаттерны: God Objects, tight coupling
- Циклические зависимости
- Метрики качества: цикломатическая сложность, техдолг
- Рекомендации по рефакторингу

**Пример использования:**
```typescript
const analysis = await callTool('export_ai_compact', {
  project_path: '/path/to/project',
  focus_critical_only: true  // только критические проблемы
});
```

### 📊 `analyze_project`
**Краткий анализ с оценкой архитектурного риска**
- Размер проекта и распределение файлов
- Оценка архитектурного риска (малый/средний/крупный)
- Рекомендации по углубленному анализу

**Пример использования:**
```typescript
const stats = await callTool('analyze_project', {
  project_path: '/path/to/project',
  verbose: true
});
```

### 📁 `get_project_structure`
**Структура с выявлением структурных проблем**
- Иерархическая структура проекта
- Неправильная организация слоев
- Файлы-кандидаты на рефакторинг
- Метрики по типам файлов

**Пример использования:**
```typescript
const structure = await callTool('get_project_structure', {
  project_path: '/path/to/project',
  show_metrics: true,
  max_files: 100
});
```

### 📈 `generate_diagram`
**Архитектурные диаграммы с проблемными связями**
- Mermaid диаграммы архитектуры
- Визуализация проблемных связей (красным)
- Метрики сложности компонентов
- Слои архитектуры

**Пример использования:**
```typescript
const diagram = await callTool('generate_diagram', {
  project_path: '/path/to/project',
  diagram_type: 'mermaid',
  include_metrics: true
});
```

## 🔧 Архитектура

### Основные компоненты:
- **`archlens_mcp_server.cjs`** - Главный MCP сервер
- **`package.json`** - Зависимости и скрипты
- **Binary Management** - Автоматический поиск бинарника ArchLens
- **Unified Error Handling** - Интеллектуальная обработка ошибок

### Ключевые улучшения v1.0.2:
- ✅ Автоматическое преобразование путей (относительные → абсолютные)
- ✅ Унифицированная обработка ошибок с решениями
- ✅ Исправлены конфликты имен переменных
- ✅ Правильная работа с файловым выводом (`generate_diagram`)
- ✅ Улучшенное логирование для отладки
- ✅ Стабильная работа с разными типами проектов

## 📋 Диагностика

### Частые проблемы:

**1. "ArchLens бинарник не найден"**
```bash
cargo build --release
npm run update-binary
```

**2. "Путь не существует"**
- Используйте абсолютные пути
- Проверьте права доступа к папке

**3. "Отказано в доступе"**
- Запустите от имени администратора
- Временно отключите антивирус

### Логи отладки:
MCP сервер выводит подробные логи в STDERR:
```
[MCP] Преобразование пути: "." → "C:\Full\Path"
[MCP] Запуск export: archlens.exe export C:\Full\Path ai_compact
[MCP] Export завершен с кодом: 0
[MCP] STDOUT длина: 1081 bytes
```

## 🧪 Тестирование

### Автоматические тесты:
```bash
# Полный тест всех инструментов
node test_all_tools.js

# Детальный тест с логами
node test_detailed.js

# Простой тест
node test_mcp.js
```

### Ожидаемые результаты:
- `export_ai_compact`: ~1000-3000 символов AI анализа
- `analyze_project`: ~1500-2000 символов статистики  
- `get_project_structure`: ~1000-1500 символов структуры
- `generate_diagram`: ~5000-10000 символов Mermaid диаграммы

## 📚 Интеграция

### В AI коде (например, в Cursor):
```typescript
// Получение полного анализа архитектуры
const analysis = await tools.export_ai_compact({ 
  project_path: workspace.rootPath,
  focus_critical_only: false 
});

// Анализ конкретных проблем
const criticalIssues = await tools.export_ai_compact({ 
  project_path: workspace.rootPath,
  focus_critical_only: true 
});

// Быстрая оценка проекта
const quickStats = await tools.analyze_project({ 
  project_path: workspace.rootPath 
});

// Диаграмма для понимания архитектуры
const mermaidDiagram = await tools.generate_diagram({ 
  project_path: workspace.rootPath,
  diagram_type: 'mermaid'
});
```

## 📈 Производительность

- **Малые проекты** (<50 файлов): <2 секунды
- **Средние проекты** (50-200 файлов): 2-5 секунд  
- **Крупные проекты** (>200 файлов): 5-15 секунд

Автоматическая оптимизация:
- Параллельная обработка файлов
- Умное кэширование анализа
- Ограничения по глубине сканирования

## 🔒 Безопасность

- Все операции выполняются локально
- Нет передачи данных во внешние сервисы
- Автоматическая очистка временных файлов
- Проверка прав доступа к файлам

---

**📧 Поддержка:** Создайте issue в репозитории для вопросов и предложений

**🎯 Статус:** Стабильная версия, готова к продуктивному использованию 