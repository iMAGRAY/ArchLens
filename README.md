# ArchLens 🔍

**Интеллектуальный анализатор архитектуры кода** - современное desktop-приложение для анализа и визуализации архитектуры программных проектов.

![Tauri](https://img.shields.io/badge/Tauri-1.7-blue?logo=tauri)
![Rust](https://img.shields.io/badge/Rust-1.70+-orange?logo=rust)
![React](https://img.shields.io/badge/React-18+-blue?logo=react)
![TypeScript](https://img.shields.io/badge/TypeScript-5+-blue?logo=typescript)
![License](https://img.shields.io/badge/License-MIT-green)

## ✨ Возможности

- 🔍 **Автоматический анализ** кода проектов (Rust, TypeScript, JavaScript, Python)
- 🏗️ **Построение архитектурных капсул** - структурных единиц кода
- 📊 **Метрики сложности** - цикломатическая сложность, связность, сцепление
- 🎯 **Архитектурные слои** - автоматическое определение слоев (Core, API, UI, Utils, Business)
- 📈 **Визуализация графа зависимостей** - интерактивные диаграммы
- 📄 **Экспорт в разные форматы** - JSON, YAML, Mermaid диаграммы
- 🔗 **Анализ связей** между компонентами
- ⚠️ **Детекция проблем** - высокая сложность, отсутствие документации
- 💡 **Рекомендации** по улучшению архитектуры

## 🚀 Быстрый старт

### Требования

- **Rust** 1.70+ ([установить](https://rustup.rs/))
- **Node.js** 18+ ([установить](https://nodejs.org/))
- **npm** или **yarn**

### Установка

```bash
# Клонируйте репозиторий
git clone https://github.com/username/ArchLens.git
cd ArchLens

# Установите зависимости
npm install

# Запустите в режиме разработки
npm run tauri dev
```

### Сборка

```bash
# Создание production билда
npm run tauri build
```

Собранное приложение будет находиться в `src-tauri/target/release/`.

## 🎯 Использование

1. **Запустите приложение**
2. **Выберите папку проекта** через диалог выбора
3. **Нажмите "Анализировать"** для запуска анализа
4. **Изучите результаты**:
   - Метрики проекта (количество капсул, связей, индексы качества)
   - Архитектурные слои
   - Рекомендации по улучшению
5. **Экспортируйте результаты** в нужном формате

## 🏗️ Архитектура

### Основные компоненты

- **FileScanner** - сканирование и фильтрация файлов проекта
- **ParserAST** - парсинг кода и извлечение AST элементов  
- **CapsuleConstructor** - создание архитектурных капсул
- **CapsuleGraphBuilder** - построение графа зависимостей
- **CapsuleEnricher** - обогащение метаданными
- **ValidatorOptimizer** - валидация и оптимизация
- **Exporter** - экспорт в различные форматы

### Технический стек

**Backend (Rust)**
- [Tauri](https://tauri.app/) - кроссплатформенный фреймворк
- [Serde](https://serde.rs/) - сериализация данных
- [Regex](https://docs.rs/regex/) - обработка паттернов
- [Walkdir](https://docs.rs/walkdir/) - обход файловой системы
- [Chrono](https://docs.rs/chrono/) - работа со временем

**Frontend (React + TypeScript)**
- [React](https://reactjs.org/) - UI фреймворк
- [Material-UI](https://mui.com/) - компоненты интерфейса
- [Vite](https://vitejs.dev/) - сборщик и dev сервер
- [TypeScript](https://www.typescriptlang.org/) - типизация

## 📋 Поддерживаемые языки

- ✅ **Rust** (.rs)
- ✅ **TypeScript** (.ts, .tsx)  
- ✅ **JavaScript** (.js, .jsx)
- ✅ **Python** (.py)
- 🔄 **Java** (планируется)
- 🔄 **Go** (планируется)
- 🔄 **C/C++** (планируется)

## 🤝 Участие в разработке

Мы приветствуем участие в разработке проекта!

### Локальная разработка

```bash
# Установите зависимости
npm install

# Запустите dev сервер
npm run tauri dev

# Запустите тесты
cargo test
npm test

# Проверьте код
cargo clippy
npm run lint
```

### Структура проекта

```
ArchLens/
├── src/                    # Rust backend
│   ├── commands.rs         # Tauri команды
│   ├── core.rs            # Основные типы данных  
│   ├── file_scanner.rs    # Сканирование файлов
│   ├── parser_ast.rs      # AST парсер
│   ├── capsule_constructor.rs # Создание капсул
│   └── ...
├── src/                   # React frontend  
│   ├── App.tsx           # Главный компонент
│   ├── main.tsx          # Точка входа
│   └── ...
├── public/               # Статические файлы
├── icons/               # Иконки приложения
└── tauri.conf.json      # Конфигурация Tauri
```

## 📄 Лицензия

Этот проект распространяется под лицензией MIT. Подробности в файле [LICENSE](LICENSE).

## 🔗 Полезные ссылки

- [Документация Tauri](https://tauri.app/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [React документация](https://reactjs.org/docs/)
- [Material-UI документация](https://mui.com/)

## 📞 Контакты

Если у вас есть вопросы или предложения, создайте [Issue](https://github.com/username/ArchLens/issues) или отправьте Pull Request.

---

*ArchLens - делаем архитектуру кода понятной и измеримой* 🚀 