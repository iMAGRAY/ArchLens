#!/usr/bin/env node

// 🧪 Тест MCP сервера ArchLens
// Демонстрация базовой функциональности

const { spawn } = require('child_process');
const path = require('path');

async function testMCPServer() {
  console.log('🧪 Тестирование ArchLens MCP Server...\n');
  
  try {
    // Тестируем анализ структуры текущего проекта
    const testRequest = {
      jsonrpc: "2.0",
      id: 1,
      method: "tools/call",
      params: {
        name: "get_project_structure",
        arguments: {
          project_path: path.join(__dirname, ".."),
          max_files: 10,
          show_metrics: true
        }
      }
    };
    
    console.log('📤 Отправляем запрос:');
    console.log(JSON.stringify(testRequest, null, 2));
    console.log('\n⏳ Ожидаем ответ от MCP сервера...\n');
    
    const child = spawn('node', ['archlens_mcp_server.cjs'], {
      stdio: ['pipe', 'pipe', 'pipe'],
      cwd: __dirname
    });
    
    let response = '';
    
    child.stdout.on('data', (data) => {
      response += data.toString();
    });
    
    child.stderr.on('data', (data) => {
      console.log('ℹ️ Лог сервера:', data.toString().trim());
    });
    
    // Отправляем запрос
    child.stdin.write(JSON.stringify(testRequest) + '\n');
    
    // Ждем ответ
    setTimeout(() => {
      console.log('📥 Ответ от сервера:');
      if (response) {
        try {
          const parsed = JSON.parse(response);
          console.log(JSON.stringify(parsed, null, 2));
        } catch (e) {
          console.log(response);
        }
      } else {
        console.log('❌ Нет ответа от сервера');
      }
      
      child.kill();
      console.log('\n✅ Тест завершен');
    }, 3000);
    
  } catch (error) {
    console.error('❌ Ошибка тестирования:', error.message);
  }
}

// Запуск теста
if (require.main === module) {
  testMCPServer();
}

module.exports = { testMCPServer }; 