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
  const arch = os.arch();
  
  // Ищем бинарник в разных местах
  const possiblePaths = [
    path.join(__dirname, '..', 'target', 'release', 'archlens.exe'),
    path.join(__dirname, '..', 'target', 'release', 'archlens'),
    path.join(__dirname, '..', 'target', 'debug', 'archlens.exe'),
    path.join(__dirname, '..', 'target', 'debug', 'archlens'),
    'archlens.exe',
    'archlens'
  ];
  
  for (const binPath of possiblePaths) {
    if (fs.existsSync(binPath)) {
      return binPath;
    }
  }
  
  throw new Error('ArchLens бинарник не найден. Убедитесь что проект собран: cargo build --release');
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
    
    // Создаем временный файл конфигурации
    const tempConfig = {
      project_path,
      include_patterns,
      exclude_patterns,
      max_depth,
      analyze_dependencies,
      extract_comments,
      generate_summaries,
      languages: ["Rust", "TypeScript", "JavaScript", "Python"]
    };
    
    const configPath = path.join(os.tmpdir(), `archlens_config_${Date.now()}.json`);
    fs.writeFileSync(configPath, JSON.stringify(tempConfig, null, 2));
    
    // Запускаем анализ через бинарник
    const result = await new Promise((resolve, reject) => {
      const binary = getArchLensBinary();
      const child = spawn(binary, ['analyze', '--config', configPath], {
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
        // Удаляем временный файл
        try {
          fs.unlinkSync(configPath);
        } catch (e) {}
        
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
    
    // Сначала выполняем анализ
    const analysisResult = await handleAnalyzeProject({ project_path });
    
    if (analysisResult.isError) {
      return analysisResult;
    }
    
    // Экспортируем в AI Compact формат
    const result = await new Promise((resolve, reject) => {
      const binary = getArchLensBinary();
      const args = ['export', '--format', 'ai_compact', '--project', project_path];
      
      if (output_file) {
        args.push('--output', output_file);
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
      const child = spawn(binary, ['structure', '--project', project_path, '--format', 'json'], {
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
      const args = ['diagram', '--project', project_path, '--type', diagram_type];
      
      if (output_file) {
        args.push('--output', output_file);
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
    if (name === "analyze_project") {
      return await handleAnalyzeProject(args);
    } else if (name === "export_ai_compact") {
      return await handleExportAICompact(args);
    } else if (name === "get_project_structure") {
      return await handleGetProjectStructure(args);
    } else if (name === "generate_diagram") {
      return await handleGenerateDiagram(args);
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