#!/usr/bin/env node

// 🛠️ Простая установка ArchLens MCP Server
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

console.log('🛠️ ArchLens MCP Server - Простая установка\n');

// Проверка зависимостей
console.log('📋 Проверка зависимостей');
console.log('─'.repeat(50));

try {
  const nodeVersion = execSync('node --version', { encoding: 'utf8' }).trim();
  console.log(`✅ Node.js: ${nodeVersion}`);
  
  const npmVersion = execSync('npm --version', { encoding: 'utf8' }).trim();
  console.log(`✅ npm: ${npmVersion}`);
} catch (error) {
  console.error('❌ Node.js или npm не установлены!');
  process.exit(1);
}

// Генерация конфигурации
console.log('\n⚙️ Генерация конфигурации');
console.log('─'.repeat(50));

const currentPath = __dirname;
const serverPath = path.join(currentPath, 'archlens_mcp_server.cjs');

const config = {
  "mcp.servers": {
    "archlens": {
      "command": "node",
      "args": [serverPath],
      "cwd": currentPath
    }
  }
};

// Сохраняем конфигурацию
const configPath = path.join(__dirname, 'cursor_config.json');
fs.writeFileSync(configPath, JSON.stringify(config, null, 2));

console.log(`✅ Конфигурация создана: ${configPath}`);

// Проверка бинарника
console.log('\n🔍 Проверка бинарника ArchLens');
console.log('─'.repeat(50));

const binaryPaths = [
  path.join(__dirname, '..', 'target', 'release', 'archlens.exe'),
  path.join(__dirname, '..', 'target', 'debug', 'archlens.exe')
];

let binaryFound = false;
for (const binPath of binaryPaths) {
  if (fs.existsSync(binPath)) {
    console.log(`✅ Бинарник найден: ${binPath}`);
    binaryFound = true;
    break;
  }
}

if (!binaryFound) {
  console.log('⚠️ Бинарник ArchLens не найден');
  console.log('   Выполните: cargo build --release');
}

// Инструкции
console.log('\n🎯 Настройка Cursor');
console.log('─'.repeat(50));

console.log('1. Откройте настройки Cursor (Ctrl+,)');
console.log('2. Нажмите иконку {} для settings.json');
console.log('3. Добавьте конфигурацию:');
console.log('');
console.log(JSON.stringify(config, null, 2));
console.log('');
console.log('4. Перезапустите Cursor');
console.log('5. Тестируйте: "Какие инструменты анализа архитектуры у тебя есть?"');

// Сохранение инструкций
const instructions = `# ArchLens MCP Server - Настройка

## Конфигурация для Cursor:
${JSON.stringify(config, null, 2)}

## Тестовые команды:
- "Какие инструменты анализа архитектуры у тебя есть?"
- "Покажи структуру проекта в папке ${path.join(__dirname, '..')}"
- "Создай Mermaid диаграмму архитектуры"
- "Экспортируй архитектуру в AI Compact формат"

## Файлы:
- Сервер: ${serverPath}
- Конфигурация: ${configPath}
- Документация: ${path.join(__dirname, 'README.md')}
`;

fs.writeFileSync(path.join(__dirname, 'setup_complete.md'), instructions);

console.log('\n🎉 УСТАНОВКА ЗАВЕРШЕНА!');
console.log('─'.repeat(50));
console.log('✅ MCP сервер готов к работе');
console.log('✅ Конфигурация создана');
console.log('✅ Инструкции сохранены в setup_complete.md');
console.log('');
console.log('🚀 Наслаждайтесь анализом архитектуры в Cursor!'); 