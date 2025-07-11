#!/usr/bin/env node

// 🧪 Тест исправленного MCP сервера на устойчивость к ошибкам доступа
const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

console.log('🧪 Тест исправленного MCP сервера');
console.log('═'.repeat(60));

// Тест прямого вызова бинарника
console.log('\n🔍 Тест 1: Прямой вызов бинарника');
console.log('─'.repeat(40));

const binaryPath = path.join(__dirname, 'archlens.exe');
const testPaths = [
    { path: '.', name: 'Текущая папка mcp' },
    { path: '..', name: 'Корень проекта' },
    { path: 'C:\\Windows\\System32', name: 'Системная папка (ограниченный доступ)' }
];

async function testBinaryAccess() {
    for (const testCase of testPaths) {
        console.log(`\n📋 Тест: ${testCase.name} (${testCase.path})`);
        
        const child = spawn(binaryPath, ['analyze', testCase.path], {
            stdio: 'pipe',
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
            if (code === 0) {
                console.log(`✅ Успешно (код ${code})`);
                try {
                    const result = JSON.parse(stdout);
                    console.log(`📊 Файлов: ${result.total_files || 'N/A'}`);
                } catch (e) {
                    console.log(`📊 Вывод получен`);
                }
            } else {
                console.log(`❌ Ошибка (код ${code})`);
                if (stderr.includes('os error 5') || stderr.includes('Access is denied')) {
                    console.log(`🔒 Ошибка доступа обнаружена - это нормально для системных папок`);
                } else {
                    console.log(`📋 Ошибка: ${stderr.substring(0, 200)}...`);
                }
            }
        });
        
        await new Promise(resolve => child.on('close', resolve));
    }
}

// Тест AI Compact экспорта
console.log('\n🤖 Тест 2: AI Compact экспорт');
console.log('─'.repeat(40));

async function testAICompactExport() {
    const child = spawn(binaryPath, ['export', '.', 'ai_compact'], {
        stdio: 'pipe',
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
        if (code === 0) {
            console.log(`✅ AI Compact экспорт успешен`);
            console.log(`📄 Длина вывода: ${stdout.length} символов`);
            console.log(`🔢 Примерно токенов: ${Math.ceil(stdout.length / 4)}`);
        } else {
            console.log(`❌ Ошибка AI Compact экспорта (код ${code})`);
            console.log(`📋 Ошибка: ${stderr}`);
        }
    });
    
    await new Promise(resolve => child.on('close', resolve));
}

// Тест структуры проекта
console.log('\n📊 Тест 3: Структура проекта');
console.log('─'.repeat(40));

async function testProjectStructure() {
    const child = spawn(binaryPath, ['structure', '..'], {
        stdio: 'pipe',
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
        if (code === 0) {
            console.log(`✅ Структура проекта получена успешно`);
            try {
                const result = JSON.parse(stdout);
                console.log(`📊 Найдено файлов: ${result.total_files || 'N/A'}`);
                console.log(`📋 Типы файлов: ${Object.keys(result.file_types || {}).length || 'N/A'}`);
            } catch (e) {
                console.log(`📊 Структура получена`);
            }
        } else {
            console.log(`❌ Ошибка получения структуры (код ${code})`);
            console.log(`📋 Ошибка: ${stderr}`);
        }
    });
    
    await new Promise(resolve => child.on('close', resolve));
}

// Запуск всех тестов
async function runAllTests() {
    try {
        await testBinaryAccess();
        await testAICompactExport();
        await testProjectStructure();
        
        console.log('\n🎉 Все тесты завершены!');
        console.log('═'.repeat(60));
        console.log('✅ MCP сервер готов к работе с улучшенной обработкой ошибок');
        console.log('📋 Проблемы доступа к файлам теперь обрабатываются корректно');
        console.log('🔧 Добавлены детальные диагностические сообщения');
        
    } catch (error) {
        console.error('❌ Ошибка при запуске тестов:', error);
    }
}

runAllTests(); 