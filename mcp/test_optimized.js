#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');

// Цвета для консоли
const colors = {
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m',
  white: '\x1b[37m',
  reset: '\x1b[0m'
};

function log(color, message) {
  console.log(`${colors[color]}${message}${colors.reset}`);
}

async function testMCPTool(toolName, args) {
  return new Promise((resolve) => {
    log('cyan', `\n🧪 Тестирую ${toolName}...`);
    
    const mcpRequest = JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "tools/call",
      params: {
        name: toolName,
        arguments: args
      }
    });
    
    const child = spawn('node', ['archlens_mcp_server.cjs'], {
      cwd: __dirname,
      stdio: ['pipe', 'pipe', 'pipe']
    });
    
    let output = '';
    let stderr = '';
    
    child.stdout.on('data', (data) => {
      output += data.toString();
    });
    
    child.stderr.on('data', (data) => {
      stderr += data.toString();
    });
    
    child.on('close', (code) => {
      try {
        // Ищем JSON ответ в выводе
        const jsonMatch = output.match(/\{.*\}/s);
        if (jsonMatch) {
          const response = JSON.parse(jsonMatch[0]);
          if (response.result && response.result.content) {
            const content = response.result.content[0].text;
            const tokenCount = content.length;
            log('green', `✅ ${toolName}: ${tokenCount} символов`);
            log('blue', `📝 Первые 200 символов:\n${content.substring(0, 200)}...`);
            resolve({ success: true, content, tokenCount });
          } else {
            log('red', `❌ ${toolName}: Нет контента в ответе`);
            resolve({ success: false, error: 'No content' });
          }
        } else {
          log('red', `❌ ${toolName}: Нет JSON ответа`);
          resolve({ success: false, error: 'No JSON response' });
        }
      } catch (error) {
        log('red', `❌ ${toolName}: Ошибка парсинга - ${error.message}`);
        resolve({ success: false, error: error.message });
      }
    });
    
    // Отправляем запрос
    child.stdin.write(mcpRequest + '\n');
    child.stdin.end();
    
    // Timeout через 15 секунд
    setTimeout(() => {
      child.kill();
      log('yellow', `⏰ ${toolName}: Timeout`);
      resolve({ success: false, error: 'Timeout' });
    }, 15000);
  });
}

async function main() {
  log('magenta', '🚀 Тестирование оптимизированных MCP инструментов');
  log('white', '═'.repeat(60));
  
  const tests = [
    {
      name: 'analyze_project',
      args: { project_path: '..' }  // Указываем корень проекта
    },
    {
      name: 'get_project_structure', 
      args: { project_path: '..', max_files: 10 }
    },
    {
      name: 'export_ai_compact',
      args: { project_path: '..' }
    }
  ];
  
  let passed = 0;
  let total = tests.length;
  
  for (const test of tests) {
    const result = await testMCPTool(test.name, test.args);
    if (result.success) {
      passed++;
    }
  }
  
  log('white', '\n' + '═'.repeat(60));
  log('magenta', '📊 Результаты оптимизации');
  log('white', `Тестов пройдено: ${passed}/${total}`);
  
  if (passed === total) {
    log('green', '🎉 Все инструменты оптимизированы для ИИ!');
  } else {
    log('yellow', '⚠️ Некоторые инструменты требуют доработки');
  }
}

main().catch(console.error); 