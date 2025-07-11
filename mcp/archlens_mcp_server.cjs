#!/usr/bin/env node

// 🏗️ ARCHLENS MCP СЕРВЕР v1.0.0
// Интеллектуальный анализ архитектуры кода для AI редакторов (Cursor, VSCode, Claude)
const { Server } = require("@modelcontextprotocol/sdk/server/index.js");
const { StdioServerTransport } = require("@modelcontextprotocol/sdk/server/stdio.js");
const { CallToolRequestSchema, ListToolsRequestSchema } = require("@modelcontextprotocol/sdk/types.js");
const { spawn, exec } = require('child_process');
const fs = require('fs');
const path = require('path');
const os = require('os');

const server = new Server({
  name: "archlens-mcp-server",
  version: "1.0.0"
}, {
  capabilities: { tools: {} }
});

// 🔍 Определение пути к бинарнику ArchLens
function getArchLensBinary() {
  const platform = os.platform();
  const extension = platform === 'win32' ? '.exe' : '';
  
  // Приоритет: локальный бинарник в папке MCP
  const possiblePaths = [
    path.join(__dirname, `archlens${extension}`),  // В папке mcp (приоритет)
    path.join(__dirname, '..', 'target', 'release', `archlens${extension}`),
    path.join(__dirname, '..', 'target', 'debug', `archlens${extension}`),
    `archlens${extension}`,
    'archlens'
  ];
  
  for (const binPath of possiblePaths) {
    if (fs.existsSync(binPath)) {
      return binPath;
    }
  }
  
  throw new Error('❌ ArchLens бинарник не найден.\n' + 
    '📋 Убедитесь что:\n' + 
    '  1. Проект собран: cargo build --release\n' + 
    '  2. Бинарник скопирован: npm run update-binary\n' + 
    '  3. Или запустите: node update-binary.js');
}

// 🚀 Универсальная функция запуска ArchLens команд
async function runArchlensCommand(args, commandType = 'generic') {
  return new Promise((resolve, reject) => {
    const binary = getArchLensBinary();
    console.error(`[MCP] Запуск команды: ${binary} ${args.join(' ')}`);
    
    const child = spawn(binary, args, {
      stdio: ['pipe', 'pipe', 'pipe'],
      cwd: __dirname
    });
    
    let stdout = '';
    let stderr = '';
    
    child.stdout.on('data', (data) => {
      stdout += data.toString();
    });
    
    child.stderr.on('data', (data) => {
      stderr += data.toString();
    });
    
    child.on('close', (code) => {
      console.error(`[MCP] Команда завершена с кодом: ${code}`);
      
      if (code === 0) {
        try {
          // Пытаемся распарсить JSON
          const result = JSON.parse(stdout);
          resolve(result);
        } catch (e) {
          // Если не JSON, возвращаем текст
          resolve({
            status: "success",
            message: "Команда выполнена успешно",
            output: stdout,
            command_type: commandType
          });
        }
      } else {
        // Детальная диагностика ошибок
        let errorMessage = `Команда завершилась с ошибкой (код ${code})`;
        
        if (stderr.includes('os error 5') || stderr.includes('Access is denied')) {
          errorMessage += '\n🔒 Ошибка доступа к файлам - попробуйте:';
          errorMessage += '\n  • Запустить от имени администратора';
          errorMessage += '\n  • Проверить права доступа к папке';
          errorMessage += '\n  • Временно отключить антивирус';
          errorMessage += '\n  • Убедиться что файлы не используются другими процессами';
        } else if (stderr.includes('No such file or directory')) {
          errorMessage += '\n📁 Путь не найден - проверьте правильность пути к проекту';
        } else if (stderr.includes('Permission denied')) {
          errorMessage += '\n🚫 Нет прав доступа - запустите с правами администратора';
        }
        
        errorMessage += `\n📋 Детали ошибки: ${stderr}`;
        
        reject(new Error(errorMessage));
      }
    });
    
    child.on('error', (error) => {
      console.error(`[MCP] Ошибка запуска процесса: ${error.message}`);
      reject(new Error(`Не удалось запустить ArchLens: ${error.message}`));
    });
  });
}

// 📊 Анализ архитектуры проекта
async function handleAnalyzeProject(args) {
  const { 
    project_path,
    include_patterns = ["**/*.rs", "**/*.ts", "**/*.js", "**/*.py"],
    exclude_patterns = ["**/target/**", "**/node_modules/**", "**/.git/**"],
    max_depth = 10,
    analyze_dependencies = true,
    extract_comments = true,
    generate_summaries = true
  } = args;
  
  try {
    if (!project_path) {
      throw new Error("project_path обязателен");
    }
    
    if (!fs.existsSync(project_path)) {
      throw new Error(`Путь не существует: ${project_path}`);
    }
    
        // Запускаем анализ через бинарник в CLI режиме
    const result = await new Promise((resolve, reject) => {
      const binary = getArchLensBinary();
      const child = spawn(binary, ['analyze', project_path], {
        stdio: ['pipe', 'pipe', 'pipe']
      });
      
      let stdout = '';
      let stderr = '';
      
      child.stdout.on('data', (data) => {
        stdout += data.toString();
      });
      
      child.stderr.on('data', (data) => {
        stderr += data.toString();
      });
      
      child.on('close', (code) => {
        if (code === 0) {
          try {
            const analysisResult = JSON.parse(stdout);
            resolve(analysisResult);
          } catch (e) {
            resolve({
              status: "success",
              message: "Анализ завершен",
              output: stdout,
              lines_analyzed: (stdout.match(/\n/g) || []).length
            });
          }
        } else {
          reject(new Error(`Анализ завершился с ошибкой (код ${code}): ${stderr}`));
        }
      });
      
      child.on('error', (error) => {
        reject(error);
      });
    });
    
    return {
      content: [{
        type: "text",
        text: JSON.stringify({
          status: "success",
          analysis: result,
          project_path,
          analyzed_at: new Date().toISOString()
        }, null, 2)
      }]
    };
    
  } catch (error) {
    return {
      content: [{
        type: "text",
        text: JSON.stringify({
          status: "error",
          error: error.message,
          project_path
        }, null, 2)
      }],
      isError: true
    };
  }
}

// 🤖 Экспорт в AI Compact формат
async function handleExportAICompact(args) {
  const { 
    project_path,
    output_file,
    include_diff_analysis = true,
    focus_critical_only = true
  } = args;
  
  try {
    if (!project_path) {
      throw new Error("project_path обязателен");
    }
    
    // Напрямую экспортируем в AI Compact формат
    const result = await new Promise((resolve, reject) => {
      const binary = getArchLensBinary();
      const args = ['export', project_path, 'ai_compact'];
      
      if (output_file) {
        args.push(output_file);
      }
      
      const child = spawn(binary, args, {
        stdio: ['pipe', 'pipe', 'pipe']
      });
      
      let stdout = '';
      let stderr = '';
      
      child.stdout.on('data', (data) => {
        stdout += data.toString();
      });
      
      child.stderr.on('data', (data) => {
        stderr += data.toString();
      });
      
      child.on('close', (code) => {
        if (code === 0) {
          resolve({
            status: "success",
            ai_compact_analysis: stdout,
            output_file: output_file || "stdout",
            token_count: Math.ceil(stdout.length / 4), // Примерная оценка токенов
            compressed_ratio: `${((1 - stdout.length / 50000) * 100).toFixed(1)}%`
          });
        } else {
          reject(new Error(`Экспорт завершился с ошибкой (код ${code}): ${stderr}`));
        }
      });
      
      child.on('error', (error) => {
        reject(error);
      });
    });
    
    return {
      content: [{
        type: "text",
        text: JSON.stringify(result, null, 2)
      }]
    };
    
  } catch (error) {
    return {
      content: [{
        type: "text",
        text: JSON.stringify({
          status: "error",
          error: error.message,
          project_path
        }, null, 2)
      }],
      isError: true
    };
  }
}

// 📊 Получение структуры проекта
async function handleGetProjectStructure(args) {
  const { 
    project_path,
    show_metrics = true,
    max_files = 100
  } = args;
  
  try {
    if (!project_path) {
      throw new Error("project_path обязателен");
    }
    
    if (!fs.existsSync(project_path)) {
      throw new Error(`Путь не существует: ${project_path}`);
    }
    
    const result = await new Promise((resolve, reject) => {
      const binary = getArchLensBinary();
      const child = spawn(binary, ['structure', project_path], {
        stdio: ['pipe', 'pipe', 'pipe']
      });
      
      let stdout = '';
      let stderr = '';
      
      child.stdout.on('data', (data) => {
        stdout += data.toString();
      });
      
      child.stderr.on('data', (data) => {
        stderr += data.toString();
      });
      
      child.on('close', (code) => {
        if (code === 0) {
          try {
            const structure = JSON.parse(stdout);
            resolve({
              status: "success",
              structure,
              project_path,
              scanned_at: new Date().toISOString()
            });
                     } catch (e) {
             // Если JSON не распарсился, создаем структуру вручную
             resolve(createManualStructure(project_path, max_files));
           }
         } else {
           // Fallback: создаем структуру вручную
           resolve(createManualStructure(project_path, max_files));
         }
       });
       
       child.on('error', (error) => {
         // Fallback: создаем структуру вручную
         resolve(createManualStructure(project_path, max_files));
       });
    });
    
    return {
      content: [{
        type: "text",
        text: JSON.stringify(result, null, 2)
      }]
    };
    
  } catch (error) {
    return {
      content: [{
        type: "text",
        text: JSON.stringify({
          status: "error",
          error: error.message,
          project_path
        }, null, 2)
      }],
      isError: true
    };
  }
}

// 📈 Генерация архитектурных диаграмм
async function handleGenerateDiagram(args) {
  const { 
    project_path,
    diagram_type = "svg",
    output_file,
    include_metrics = true
  } = args;
  
  try {
    if (!project_path) {
      throw new Error("project_path обязателен");
    }
    
    const result = await new Promise((resolve, reject) => {
      const binary = getArchLensBinary();
      const args = ['diagram', project_path, diagram_type];
      
      if (output_file) {
        args.push(output_file);
      }
      
      const child = spawn(binary, args, {
        stdio: ['pipe', 'pipe', 'pipe']
      });
      
      let stdout = '';
      let stderr = '';
      
      child.stdout.on('data', (data) => {
        stdout += data.toString();
      });
      
      child.stderr.on('data', (data) => {
        stderr += data.toString();
      });
      
      child.on('close', (code) => {
        if (code === 0) {
          resolve({
            status: "success",
            diagram: stdout,
            diagram_type,
            output_file: output_file || "stdout",
            size: stdout.length
          });
        } else {
          reject(new Error(`Генерация диаграммы завершилась с ошибкой (код ${code}): ${stderr}`));
        }
      });
      
      child.on('error', (error) => {
        reject(error);
      });
    });
    
    return {
      content: [{
        type: "text",
        text: JSON.stringify(result, null, 2)
      }]
    };
    
  } catch (error) {
    return {
      content: [{
        type: "text",
        text: JSON.stringify({
          status: "error",
          error: error.message,
          project_path,
          diagram_type
        }, null, 2)
      }],
      isError: true
    };
  }
}

// 🔧 Вспомогательные функции
function createManualStructure(projectPath, maxFiles) {
  const structure = {
    total_files: 0,
    file_types: {},
    layers: [],
    files: []
  };
  
  try {
    const scanDirectory = (dir, depth = 0) => {
      if (depth > 5 || structure.files.length >= maxFiles) return;
      
      const items = fs.readdirSync(dir);
      
      for (const item of items) {
        const fullPath = path.join(dir, item);
        const stat = fs.statSync(fullPath);
        
        if (stat.isDirectory()) {
          if (!item.startsWith('.') && item !== 'node_modules' && item !== 'target') {
            scanDirectory(fullPath, depth + 1);
          }
        } else {
          const ext = path.extname(item).toLowerCase();
          const relativePath = path.relative(projectPath, fullPath);
          
          structure.total_files++;
          structure.file_types[ext] = (structure.file_types[ext] || 0) + 1;
          
          if (structure.files.length < maxFiles) {
            structure.files.push({
              path: relativePath,
              name: item,
              extension: ext,
              size: stat.size
            });
          }
        }
      }
    };
    
    scanDirectory(projectPath);
    
    // Определяем слои по структуре папок
    const commonLayers = ['src', 'lib', 'components', 'utils', 'api', 'core', 'ui'];
    structure.layers = commonLayers.filter(layer => {
      return fs.existsSync(path.join(projectPath, layer));
    });
    
  } catch (error) {
    structure.error = error.message;
  }
  
  return {
    status: "success",
    structure,
    project_path: projectPath,
    scanned_at: new Date().toISOString(),
    method: "manual_scan"
  };
}

// 📋 Регистрация инструментов MCP
server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: [
    {
      name: "export_ai_compact",
      description: "🤖 AI ЭКСПОРТ - Возвращает сжатый анализ архитектуры проекта (~2800 токенов) в удобном для ИИ формате. Включает критические проблемы, архитектурные паттерны, метрики качества, структуру и рекомендации.",
      inputSchema: {
        type: "object",
        properties: {
          project_path: {
            description: "Путь к проекту для анализа",
            type: "string"
          },
          output_file: {
            description: "Путь для сохранения результата (опционально)",
            type: "string"
          },
          focus_critical_only: {
            description: "Показывать только критические проблемы",
            type: "boolean"
          },
          include_diff_analysis: {
            description: "Включить сравнение с предыдущими версиями",
            type: "boolean"
          }
        },
        required: ["project_path"]
      }
    },
    {
      name: "analyze_project",
      description: "📊 КРАТКИЙ АНАЛИЗ - Возвращает базовую статистику проекта (файлы, строки, типы) в компактном формате для быстрого понимания масштаба проекта. Идеально для первичной оценки.",
      inputSchema: {
        type: "object",
        properties: {
          project_path: {
            description: "Путь к проекту для анализа",
            type: "string"
          },
          verbose: {
            description: "Подробный вывод анализа",
            type: "boolean"
          },
          analyze_dependencies: {
            description: "Анализировать зависимости между модулями",
            type: "boolean"
          },
          extract_comments: {
            description: "Извлекать комментарии и документацию",
            type: "boolean"
          },
          generate_summaries: {
            description: "Генерировать краткие описания компонентов",
            type: "boolean"
          },
          include_patterns: {
            description: "Паттерны файлов для включения (например: ['**/*.rs', '**/*.ts'])",
            type: "array",
            items: { type: "string" }
          },
          exclude_patterns: {
            description: "Паттерны файлов для исключения",
            type: "array",
            items: { type: "string" }
          },
          max_depth: {
            description: "Максимальная глубина сканирования директорий",
            type: "integer"
          }
        },
        required: ["project_path"]
      }
    },
    {
      name: "generate_diagram",
      description: "📈 ГЕНЕРАЦИЯ ДИАГРАММ - Создает архитектурную диаграмму проекта в указанном формате. Для Mermaid диаграмм возвращает готовый код с описанием структуры, компонентов и связей.",
      inputSchema: {
        type: "object",
        properties: {
          project_path: {
            description: "Путь к проекту для анализа",
            type: "string"
          },
          diagram_type: {
            description: "Тип диаграммы: mermaid (по умолчанию), svg, dot",
            type: "string",
            enum: ["mermaid", "svg", "dot"]
          },
          include_metrics: {
            description: "Включить метрики в диаграмму",
            type: "boolean"
          },
          output_file: {
            description: "Путь для сохранения диаграммы (опционально)",
            type: "string"
          }
        },
        required: ["project_path"]
      }
    },
    {
      name: "get_project_structure",
      description: "📁 СТРУКТУРА ПРОЕКТА - Возвращает иерархическую структуру проекта с типами файлов, базовыми метриками и описанием архитектурных слоев. Оптимизировано для понимания ИИ.",
      inputSchema: {
        type: "object",
        properties: {
          project_path: {
            description: "Путь к проекту",
            type: "string"
          },
          show_metrics: {
            description: "Включить базовые метрики (размер файлов, количество строк и т.д.)",
            type: "boolean"
          },
          max_files: {
            description: "Максимальное количество файлов в выводе (по умолчанию 50)",
            type: "integer"
          }
        },
        required: ["project_path"]
      }
    }
  ]
}));

// 🎯 Обработка вызовов инструментов
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;
  
  try {
    if (name === 'analyze_project') {
      const projectPath = args.project_path || '.';
      const analyzeArgs = ['analyze', projectPath];
      
      if (args.verbose) {
        analyzeArgs.push('--verbose');
      }
      
      console.error(`[MCP] Анализ проекта: ${projectPath}`);
      
      try {
        const result = await runArchlensCommand(analyzeArgs, 'analyze');
        console.error(`[MCP] Анализ завершен успешно`);
        
        // Парсим JSON результат
        let analysisData;
        try {
          analysisData = typeof result === 'string' ? JSON.parse(result) : result;
        } catch {
          analysisData = result;
        }
        
        // Создаем краткий ИИ-дружественный анализ
        const aiAnalysis = `# 🔍 КРАТКИЙ АНАЛИЗ ПРОЕКТА

**Путь:** ${projectPath}
**Анализ выполнен:** ${new Date().toLocaleString('ru-RU')}

## 📊 Основные метрики
- **Всего файлов:** ${analysisData.total_files || 'н/д'}
- **Строк кода:** ${analysisData.total_lines || 'н/д'}
- **Дата сканирования:** ${analysisData.scanned_at ? new Date(analysisData.scanned_at).toLocaleString('ru-RU') : 'н/д'}

## 🗂️ Распределение по типам файлов
${analysisData.file_types ? Object.entries(analysisData.file_types)
  .sort(([,a], [,b]) => b - a)
  .slice(0, 10)
  .map(([ext, count]) => `- **.${ext}**: ${count} файл(ов)`)
  .join('\n') : 'Данные недоступны'}

## 📈 Архитектурная оценка
${analysisData.total_files && analysisData.total_files > 100 ? 
  '⚠️ **КРУПНЫЙ ПРОЕКТ** - рекомендуется модульная архитектура' : 
  analysisData.total_files > 50 ? 
    '✅ **СРЕДНИЙ ПРОЕКТ** - хорошо управляемый размер' : 
    '✅ **МАЛЫЙ ПРОЕКТ** - компактная структура'}

## 🎯 Рекомендации для ИИ
- Используйте \`export_ai_compact\` для полного анализа архитектуры (~2800 токенов)
- Используйте \`generate_diagram\` для визуализации структуры  
- Используйте \`get_project_structure\` для детального изучения файлов

*Это краткая сводка. Для глубокого анализа используйте другие инструменты.*`;
        
        return {
          content: [
            {
              type: 'text',
              text: aiAnalysis
            }
          ]
        };
      } catch (error) {
        console.error(`[MCP] Ошибка анализа: ${error.message}`);
        return {
          content: [
            {
              type: 'text',
              text: `❌ ОШИБКА АНАЛИЗА ПРОЕКТА
              
Не удалось выполнить анализ проекта: ${projectPath}

**Причина:** ${error.message}

**Рекомендации по устранению:**
- Проверьте права доступа к файлам и папкам
- Убедитесь что путь существует и содержит исходный код
- Временно отключите антивирус
- Попробуйте запустить от имени администратора
- Проверьте что проект не поврежден

**Альтернативы:**
- Попробуйте \`export_ai_compact\` для альтернативного анализа
- Используйте \`get_project_structure\` для быстрого обзора

**Путь к проекту:** ${projectPath}
**Время ошибки:** ${new Date().toLocaleString('ru-RU')}`
            }
          ]
        };
      }
    } else if (name === "export_ai_compact") {
      const projectPath = args.project_path || '.';
      const outputFile = args.output_file;
      const focusCriticalOnly = args.focus_critical_only || false;
      const includeDiffAnalysis = args.include_diff_analysis || false;
      
      const exportArgs = ['export', projectPath, 'ai_compact'];
      
      if (focusCriticalOnly) {
        exportArgs.push('--focus-critical');
      }
      
      if (includeDiffAnalysis) {
        exportArgs.push('--include-diff');
      }
      
      if (outputFile) {
        exportArgs.push(outputFile);
      }
      
      console.error(`[MCP] AI Compact экспорт: ${projectPath}`);
      
      try {
        const result = await runArchlensCommand(exportArgs, 'ai_compact');
        console.error(`[MCP] AI Compact экспорт завершен успешно`);
        
        // Возвращаем прямой контент анализа для ИИ
        const analysisContent = result.output || JSON.stringify(result, null, 2);
        
        return {
          content: [
            {
              type: 'text',
              text: analysisContent  // Прямой контент без JSON обертки
            }
          ]
        };
      } catch (error) {
        console.error(`[MCP] Ошибка AI Compact экспорта: ${error.message}`);
        return {
          content: [
            {
              type: 'text',
              text: `❌ ОШИБКА АНАЛИЗА АРХИТЕКТУРЫ
              
Не удалось выполнить AI Compact экспорт для проекта: ${projectPath}

**Причина:** ${error.message}

**Рекомендации:**
- Проверьте корректность пути к проекту
- Убедитесь что у ArchLens есть права доступа к файлам
- Проверьте что проект содержит исходный код
- Попробуйте запустить с правами администратора

**Путь к проекту:** ${projectPath}
**Время ошибки:** ${new Date().toISOString()}`
            }
          ]
        };
      }
    } else if (name === "generate_diagram") {
      const projectPath = args.project_path || '.';
      const diagramType = args.diagram_type || 'mermaid';
      const outputFile = args.output_file;
      const includeMetrics = args.include_metrics || false;
      
      const diagramArgs = ['diagram', projectPath, diagramType];
      
      if (includeMetrics) {
        diagramArgs.push('--include-metrics');
      }
      
      if (outputFile) {
        diagramArgs.push(outputFile);
      }
      
      console.error(`[MCP] Генерация диаграммы: ${projectPath} (${diagramType})`);
      
      try {
        const result = await runArchlensCommand(diagramArgs, 'diagram');
        console.error(`[MCP] Генерация диаграммы завершена успешно`);
        
        // Возвращаем прямой контент диаграммы для ИИ
        const diagramContent = result.output || result.diagram || JSON.stringify(result, null, 2);
        
        // Если это Mermaid диаграмма, добавляем дополнительное форматирование
        let formattedContent = diagramContent;
        if (diagramType === 'mermaid') {
          formattedContent = `# 📊 АРХИТЕКТУРНАЯ ДИАГРАММА

**Проект:** ${projectPath}
**Тип:** ${diagramType}
**Создана:** ${new Date().toISOString()}

## Mermaid Диаграмма

\`\`\`mermaid
${diagramContent}
\`\`\`

## Описание

Эта диаграмма показывает архитектурную структуру проекта, включая:
- Основные компоненты и модули
- Связи между компонентами
- Зависимости и потоки данных
- Слои архитектуры

*Сгенерировано ArchLens для AI анализа*`;
        }
        
        return {
          content: [
            {
              type: 'text',
              text: formattedContent  // Прямой контент диаграммы
            }
          ]
        };
      } catch (error) {
        console.error(`[MCP] Ошибка генерации диаграммы: ${error.message}`);
        return {
          content: [
            {
              type: 'text',
              text: `❌ ОШИБКА ГЕНЕРАЦИИ ДИАГРАММЫ
              
Не удалось создать диаграмму для проекта: ${projectPath}

**Тип диаграммы:** ${diagramType}
**Причина:** ${error.message}

**Рекомендации:**
- Проверьте что проект содержит исходный код
- Убедитесь что путь к проекту корректен
- Попробуйте другой тип диаграммы (mermaid, svg, dot)
- Проверьте права доступа к файлам

**Путь к проекту:** ${projectPath}
**Время ошибки:** ${new Date().toISOString()}`
            }
          ]
        };
      }
    } else if (name === "get_project_structure") {
      const projectPath = args.project_path || '.';
      const showMetrics = args.show_metrics || false;
      const maxFiles = args.max_files || 50;
      
      const structureArgs = ['structure', projectPath];
      
      if (showMetrics) {
        structureArgs.push('--show-metrics');
      }
      
      if (maxFiles !== 50) {
        structureArgs.push('--max-files', maxFiles.toString());
      }
      
      console.error(`[MCP] Получение структуры проекта: ${projectPath}`);
      
      try {
        const result = await runArchlensCommand(structureArgs, 'structure');
        console.error(`[MCP] Получение структуры завершено успешно`);
        
        // Парсим JSON результат
        let structureData;
        try {
          structureData = typeof result === 'string' ? JSON.parse(result) : result;
        } catch {
          structureData = result;
        }
        
        // Создаем краткую ИИ-дружественную структуру
        const structureOverview = `# 📁 ОБЗОР СТРУКТУРЫ ПРОЕКТА

**Путь:** ${projectPath}
**Анализ выполнен:** ${new Date().toLocaleString('ru-RU')}

## 📊 Общая статистика
- **Всего файлов:** ${structureData.total_files || 'н/д'}
- **Показано файлов:** ${Math.min(maxFiles, structureData.total_files || 0)}

## 🗂️ Типы файлов
${structureData.file_types ? Object.entries(structureData.file_types)
  .sort(([,a], [,b]) => b - a)
  .map(([ext, count]) => `- **.${ext}**: ${count} файл(ов)`)
  .join('\n') : 'Данные недоступны'}

## 🏗️ Архитектурные слои
${structureData.layers ? structureData.layers.map(layer => `- **${layer}**`).join('\n') : 'Слои не определены'}

## 📄 Ключевые файлы (топ ${Math.min(15, maxFiles)})
${structureData.files ? structureData.files
  .slice(0, 15)
  .map(file => `- \`${file.path}\` (${file.extension}, ${(file.size / 1024).toFixed(1)}KB)`)
  .join('\n') : 'Файлы недоступны'}

${structureData.files && structureData.files.length > 15 ? `\n... и еще ${structureData.files.length - 15} файл(ов)` : ''}

## 💡 Рекомендации для детального анализа
- Используйте \`export_ai_compact\` для полного анализа архитектуры
- Используйте \`generate_diagram\` для визуализации зависимостей
- Для анализа конкретных файлов используйте стандартные инструменты чтения

*Краткий обзор структуры. Полный анализ доступен через другие инструменты.*`;
        
        return {
          content: [
            {
              type: 'text',
              text: structureOverview
            }
          ]
        };
      } catch (error) {
        console.error(`[MCP] Ошибка получения структуры: ${error.message}`);
        return {
          content: [
            {
              type: 'text',
              text: `❌ ОШИБКА ПОЛУЧЕНИЯ СТРУКТУРЫ
              
Не удалось получить структуру проекта: ${projectPath}

**Причина:** ${error.message}

**Возможные решения:**
- Проверьте что путь к проекту корректен
- Убедитесь что у вас есть права доступа к папке
- Проверьте что папка не пустая
- Попробуйте указать другой путь

**Альтернативы:**
- Попробуйте \`analyze_project\` для базовой статистики
- Используйте \`export_ai_compact\` для альтернативного анализа

**Путь к проекту:** ${projectPath}
**Время ошибки:** ${new Date().toLocaleString('ru-RU')}`
            }
          ]
        };
      }
    } else {
      return {
        content: [{ 
          type: "text", 
          text: JSON.stringify({ 
            status: "error",
            error: `❌ Неизвестный инструмент: ${name}`
          }, null, 2) 
        }],
        isError: true
      };
    }
  } catch (error) {
    return {
      content: [{ 
        type: "text", 
        text: JSON.stringify({ 
          status: "error",
          error: `❌ Ошибка выполнения ${name}: ${error.message}`
        }, null, 2) 
      }],
      isError: true
    };
  }
});

// 🚀 Запуск MCP сервера
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  
  console.error("🏗️ ArchLens MCP Server v1.0.0 запущен");
  console.error("✅ Готов к анализу архитектуры кода для AI");
  
  process.stdin.resume();
}

main().catch(console.error); 