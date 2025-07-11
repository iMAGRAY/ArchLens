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
  tools: [{
    name: "analyze_project",
    description: "🔍 АНАЛИЗ АРХИТЕКТУРЫ ПРОЕКТА - Полный анализ структуры кода, метрик сложности, зависимостей и архитектурных слоев",
    inputSchema: {
      type: "object",
      properties: {
        project_path: { 
          type: "string", 
          description: "Путь к корню проекта для анализа" 
        },
        include_patterns: { 
          type: "array", 
          items: { type: "string" },
          description: "Паттерны файлов для включения (например: ['**/*.rs', '**/*.ts'])" 
        },
        exclude_patterns: { 
          type: "array", 
          items: { type: "string" },
          description: "Паттерны файлов для исключения" 
        },
        max_depth: { 
          type: "integer", 
          description: "Максимальная глубина сканирования директорий" 
        },
        analyze_dependencies: { 
          type: "boolean", 
          description: "Анализировать зависимости между модулями" 
        },
        extract_comments: { 
          type: "boolean", 
          description: "Извлекать комментарии и документацию" 
        },
        generate_summaries: { 
          type: "boolean", 
          description: "Генерировать краткие описания компонентов" 
        }
      },
      required: ["project_path"]
    }
  }, {
    name: "export_ai_compact",
    description: "🤖 AI COMPACT ЭКСПОРТ - Сжатый анализ архитектуры для AI моделей (~2800 токенов): паттерны, аномалии, критические проблемы",
    inputSchema: {
      type: "object", 
      properties: {
        project_path: { 
          type: "string", 
          description: "Путь к проекту для анализа" 
        },
        output_file: { 
          type: "string", 
          description: "Путь для сохранения результата (опционально)" 
        },
        include_diff_analysis: { 
          type: "boolean", 
          description: "Включить сравнение с предыдущими версиями" 
        },
        focus_critical_only: { 
          type: "boolean", 
          description: "Показывать только критические проблемы" 
        }
      },
      required: ["project_path"]
    }
  }, {
    name: "get_project_structure", 
    description: "📊 СТРУКТУРА ПРОЕКТА - Быстрый обзор файлов, типов, слоев и базовых метрик проекта",
    inputSchema: {
      type: "object",
      properties: {
        project_path: { 
          type: "string", 
          description: "Путь к проекту" 
        },
        show_metrics: { 
          type: "boolean", 
          description: "Включить базовые метрики" 
        },
        max_files: { 
          type: "integer", 
          description: "Максимальное количество файлов в выводе" 
        }
      },
      required: ["project_path"]
    }
  }, {
    name: "generate_diagram",
    description: "📈 ГЕНЕРАЦИЯ ДИАГРАММ - Создание архитектурных диаграмм (SVG, Mermaid) с визуализацией компонентов и связей",
    inputSchema: {
      type: "object",
      properties: {
        project_path: { 
          type: "string", 
          description: "Путь к проекту" 
        },
        diagram_type: { 
          type: "string", 
          enum: ["svg", "mermaid", "dot"], 
          description: "Тип диаграммы" 
        },
        output_file: { 
          type: "string", 
          description: "Файл для сохранения диаграммы" 
        },
        include_metrics: { 
          type: "boolean", 
          description: "Включить метрики в диаграмму" 
        }
      },
      required: ["project_path"]
    }
  }]
}));

// 🎯 Обработка вызовов инструментов
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;
  
  try {
    if (name === 'analyze_project') {
      const projectPath = args.project_path || '.';
      const analyzeArgs = ['analyze', projectPath];
      
      console.error(`[MCP] Анализ проекта: ${projectPath}`);
      
      try {
        const result = await runArchlensCommand(analyzeArgs, 'analyze');
        console.error(`[MCP] Анализ завершен успешно`);
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify(result, null, 2)
            }
          ]
        };
      } catch (error) {
        console.error(`[MCP] Ошибка анализа: ${error.message}`);
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                status: 'error',
                error: `Анализ завершился с ошибкой: ${error.message}`,
                project_path: projectPath,
                troubleshooting: [
                  'Проверьте права доступа к файлам и папкам',
                  'Убедитесь что путь существует',
                  'Временно отключите антивирус',
                  'Попробуйте запустить от имени администратора'
                ]
              }, null, 2)
            }
          ]
        };
      }
    } else if (name === "export_ai_compact") {
      const projectPath = args.project_path || '.';
      const outputFile = args.output_file;
      const exportArgs = ['export', projectPath, 'ai_compact'];
      
      if (outputFile) {
        exportArgs.push(outputFile);
      }
      
      console.error(`[MCP] AI Compact экспорт: ${projectPath}`);
      
      try {
        const result = await runArchlensCommand(exportArgs, 'ai_compact');
        console.error(`[MCP] AI Compact экспорт завершен успешно`);
        
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                status: 'success',
                ai_compact_analysis: result.output || result,
                project_path: projectPath,
                output_file: outputFile || 'stdout',
                token_count: Math.ceil((result.output || JSON.stringify(result)).length / 4),
                exported_at: new Date().toISOString()
              }, null, 2)
            }
          ]
        };
      } catch (error) {
        console.error(`[MCP] Ошибка AI Compact экспорта: ${error.message}`);
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                status: 'error',
                error: `AI Compact экспорт завершился с ошибкой: ${error.message}`,
                project_path: projectPath
              }, null, 2)
            }
          ]
        };
      }
    } else if (name === "generate_diagram") {
      const projectPath = args.project_path || '.';
      const diagramType = args.diagram_type || 'mermaid';
      const outputFile = args.output_file;
      const diagramArgs = ['diagram', projectPath, diagramType];
      
      if (outputFile) {
        diagramArgs.push(outputFile);
      }
      
      console.error(`[MCP] Генерация диаграммы: ${projectPath} (${diagramType})`);
      
      try {
        const result = await runArchlensCommand(diagramArgs, 'diagram');
        console.error(`[MCP] Генерация диаграммы завершена успешно`);
        
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                status: 'success',
                diagram_generated: true,
                project_path: projectPath,
                diagram_type: diagramType,
                output_file: outputFile || 'stdout',
                content: result.output || result,
                generated_at: new Date().toISOString()
              }, null, 2)
            }
          ]
        };
      } catch (error) {
        console.error(`[MCP] Ошибка генерации диаграммы: ${error.message}`);
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                status: 'error',
                error: `Генерация диаграммы завершилась с ошибкой: ${error.message}`,
                project_path: projectPath
              }, null, 2)
            }
          ]
        };
      }
    } else if (name === "get_project_structure") {
      const projectPath = args.project_path || '.';
      const structureArgs = ['structure', projectPath];
      
      console.error(`[MCP] Получение структуры проекта: ${projectPath}`);
      
      try {
        const result = await runArchlensCommand(structureArgs, 'structure');
        console.error(`[MCP] Получение структуры завершено успешно`);
        
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                status: 'success',
                structure: result,
                project_path: projectPath,
                retrieved_at: new Date().toISOString()
              }, null, 2)
            }
          ]
        };
      } catch (error) {
        console.error(`[MCP] Ошибка получения структуры: ${error.message}`);
        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                status: 'error',
                error: `Получение структуры завершилось с ошибкой: ${error.message}`,
                project_path: projectPath
              }, null, 2)
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