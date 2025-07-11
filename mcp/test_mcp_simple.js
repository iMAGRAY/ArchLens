#!/usr/bin/env node

// 🧪 Простой тест MCP сервера
const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');

console.log('🧪 Тест MCP Server - Диагностика проблем доступа');
console.log('═'.repeat(60));

// Проверка бинарника
const binaryPath = path.join(__dirname, 'archlens.exe');
console.log(`📋 Проверка бинарника: ${binaryPath}`);

if (!fs.existsSync(binaryPath)) {
    console.error('❌ Бинарник не найден!');
    process.exit(1);
}

const stats = fs.statSync(binaryPath);
console.log(`✅ Бинарник найден (${(stats.size / 1024 / 1024).toFixed(1)} MB)`);

// Тест разных папок
const testPaths = [
    '.',
    '..',
    path.join(__dirname, '..', 'src'),
];

async function testPathAnalysis(testPath) {
    return new Promise((resolve) => {
        console.log(`\n🔍 Тест анализа: ${testPath}`);
        console.log('─'.repeat(40));
        
        const child = spawn(binaryPath, ['analyze', testPath], {
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
                    console.log(`📊 Файлов: ${result.total_files}, строк: ${result.total_lines}`);
                } catch (e) {
                    console.log(`📊 Вывод: ${stdout.substring(0, 100)}...`);
                }
            } else {
                console.log(`❌ Ошибка (код ${code})`);
                console.log(`📋 STDERR: ${stderr}`);
                console.log(`📋 STDOUT: ${stdout}`);
            }
            resolve(code);
        });
        
        child.on('error', (err) => {
            console.log(`❌ Ошибка запуска: ${err.message}`);
            resolve(-1);
        });
    });
}

// Тест всех путей
async function runAllTests() {
    console.log('\n🚀 Запуск тестов анализа...');
    
    for (const testPath of testPaths) {
        await testPathAnalysis(testPath);
    }
    
    console.log('\n🎯 Тест AI Compact экспорта...');
    console.log('─'.repeat(40));
    
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
            console.log(`✅ AI Compact успешно (код ${code})`);
            console.log(`📄 Длина вывода: ${stdout.length} символов`);
        } else {
            console.log(`❌ AI Compact ошибка (код ${code})`);
            console.log(`📋 STDERR: ${stderr}`);
        }
    });
}

runAllTests().catch(console.error); 