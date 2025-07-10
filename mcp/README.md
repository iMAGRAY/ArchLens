# 🏗️ ArchLens MCP Server

**Интеллектуальный анализ архитектуры кода для AI редакторов**

MCP (Model Context Protocol) сервер для интеграции ArchLens с AI редакторами типа Cursor, VSCode с Claude, и другими поддерживающими MCP инструментами.

## ✨ Возможности

- 🔍 **Полный анализ архитектуры** - Анализ структуры кода, метрик сложности, зависимостей
- 🤖 **AI Compact экспорт** - Сжатый анализ (~2800 токенов) оптимизированный для AI моделей  
- 📊 **Структура проекта** - Быстрый обзор файлов, типов и архитектурных слоев
- 📈 **Генерация диаграмм** - SVG, Mermaid, DOT диаграммы компонентов и связей
- 🎯 **Автономность** - Включен готовый бинарник, не требует Rust/Cargo

## 📦 Что включено

Минимальный набор файлов для работы:
- `archlens_mcp_server.cjs` - MCP сервер (18KB)
- `archlens.exe` - бинарник ArchLens (7.1MB) 
- `install_simple.js` - автоматический установщик
- `cursor_config.json` - готовая конфигурация MCP
- `README.md` - полная документация
- `package.json` - NPM метаданные

## 🚀 Быстрый старт

### 1. Установка зависимостей

```bash
cd mcp
npm install
```

### 2. Проверка установки

Пакет включает готовый бинарник, проверьте его работу:
```bash
npm run check
```

**Примечание:** Бинарник уже включен в пакет. Сборка из исходников не требуется.

### 3. Настройка Cursor

**Автоматическая настройка (рекомендуется):**
```bash
node install_simple.js
```

**Ручная настройка** - добавьте в настройки Cursor (`settings.json`):
```json
{
  "mcp.servers": {
    "archlens": {
      "command": "node",
      "args": ["/path/to/node_modules/archlens-mcp-server/archlens_mcp_server.cjs"],
      "cwd": "/path/to/node_modules/archlens-mcp-server"
    }
  }
}
```

**Пример настроек для Windows:**
```json
{
  "mcp.servers": {
    "archlens": {
      "command": "C:\\Program Files\\nodejs\\node.exe",
      "args": ["C:\\path\\to\\archlens-mcp-server\\archlens_mcp_server.cjs"],
      "cwd": "C:\\path\\to\\archlens-mcp-server"
    }
  }
}
```

### 4. Перезапустите Cursor

После добавления настроек перезапустите Cursor для активации MCP сервера.

## 📋 Доступные инструменты

### 🔍 `analyze_project`
Полный анализ архитектуры проекта

**Параметры:**
- `project_path` (обязательный) - путь к корню проекта
- `include_patterns` - паттерны файлов для включения
- `exclude_patterns` - паттерны файлов для исключения  
- `max_depth` - максимальная глубина сканирования
- `analyze_dependencies` - анализировать зависимости
- `extract_comments` - извлекать комментарии
- `generate_summaries` - генерировать описания компонентов

**Пример использования в Cursor:**
```
Проанализируй архитектуру проекта в папке /home/user/myproject
```

### 🤖 `export_ai_compact`
Экспорт в AI Compact формат для анализа AI моделями

**Параметры:**
- `project_path` (обязательный) - путь к проекту
- `output_file` - файл для сохранения (опционально)
- `include_diff_analysis` - включить diff анализ
- `focus_critical_only` - только критические проблемы

**Пример использования:**
```
Экспортируй архитектуру проекта в AI Compact формат для анализа
```

### 📊 `get_project_structure` 
Быстрый обзор структуры проекта

**Параметры:**
- `project_path` (обязательный) - путь к проекту
- `show_metrics` - включить базовые метрики
- `max_files` - максимальное количество файлов

**Пример использования:**
```
Покажи структуру проекта в текущей папке
```

### 📈 `generate_diagram`
Генерация архитектурных диаграмм

**Параметры:**
- `project_path` (обязательный) - путь к проекту
- `diagram_type` - тип диаграммы (svg, mermaid, dot)
- `output_file` - файл для сохранения
- `include_metrics` - включить метрики

**Пример использования:**
```
Создай Mermaid диаграмму архитектуры проекта
```

## 🔧 Настройка для других редакторов

### VSCode с Claude

Установите расширение MCP и добавьте конфигурацию в `settings.json`:

```json
{
  "mcp.servers": {
    "archlens": {
      "command": "node",
      "args": ["/absolute/path/to/archlens_mcp_server.cjs"],
      "env": {}
    }
  }
}
```

### Standalone использование

Можно запустить сервер напрямую для тестирования:

```bash
node archlens_mcp_server.cjs
```

## 🚀 Примеры использования

### Анализ текущего проекта
```
AI: Проанализируй архитектуру текущего проекта и найди потенциальные проблемы
```

### Сравнение версий
```  
AI: Экспортируй AI Compact анализ проекта и покажи изменения по сравнению с предыдущей версией
```

### Генерация документации
```
AI: Создай Mermaid диаграмму архитектуры и опиши основные компоненты системы
```

## 🐛 Устранение неполадок

### Бинарник ArchLens не найден
Проверьте что пакет установлен корректно:
```bash
npm run check
```

### Ошибки прав доступа (Linux/macOS)
Убедитесь что файлы исполняемые:
```bash
chmod +x archlens_mcp_server.cjs
chmod +x archlens  # для Linux/macOS
```

### Проблемы с путями в Windows
Используйте абсолютные пути с двойными слешами:
```json
"command": "C:\\Program Files\\nodejs\\node.exe",
"args": ["C:\\path\\to\\archlens-mcp-server\\archlens_mcp_server.cjs"]
```

### MCP сервер не подключается
1. Перезапустите Cursor после изменения настроек
2. Проверьте логи: View → Output → Model Context Protocol
3. Убедитесь что Node.js >= 18.0.0

## 🔄 Обновление

Для обновления MCP сервера:
```bash
npm update archlens-mcp-server
```

## 📝 Лицензия

MIT License - см. основной проект ArchLens.

## 🤝 Поддержка

- **GitHub Issues**: Создайте issue для сообщения о проблемах
- **Документация**: см. этот README.md
- **Установка**: запустите `node install_simple.js` для автоматической настройки

---

## 📦 Автономная публикация

MCP сервер спроектирован как **полностью автономный** пакет для независимой публикации:

### Команды управления
```bash
# Обновление бинарника из основного проекта
npm run update-binary

# Проверка работоспособности
npm run check

# Создание автономного пакета для публикации
npm run standalone

# Полная сборка + упаковка
npm run dist

# Создание NPM пакета
npm run package
```

### Автономные файлы
- ✅ `archlens.exe` - включенный бинарник (6.8 MB)
- ✅ `archlens_mcp_server.cjs` - MCP сервер
- ✅ `install_simple.js` - автоматический установщик
- ✅ `update-binary.js` - обновление бинарника
- ✅ `create-standalone.js` - упаковка для публикации

### Публикация
```bash
# Создание дистрибутива
npm run dist

# NPM публикация 
npm publish

# GitHub релиз (из папки dist/)
cd dist && tar -czf ../archlens-mcp-standalone.tar.gz *
```

📋 **Размер пакета**: ~15-20 MB  
🎯 **Автономность**: не требует Rust/Cargo  
📖 **Документация**: см. `PUBLISH_GUIDE.md`

---

**🎯 Готово к работе!** ArchLens MCP Server предоставляет AI мощные инструменты для анализа архитектуры кода прямо в редакторе. 