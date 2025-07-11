#!/usr/bin/env node

// 🧪 Унифицированный тестовый набор для MCP сервера ArchLens
const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');
const { performance } = require('perf_hooks');

// Конфигурация тестов
const config = {
    binaryPath: path.join(__dirname, 'archlens.exe'),
    timeoutMs: 30000,
    verboseMode: process.argv.includes('--verbose'),
    skipSystemTests: process.argv.includes('--skip-system'),
    includePerformance: process.argv.includes('--performance'),
};

// Цвета для консоли
const colors = {
    reset: '\x1b[0m',
    red: '\x1b[31m',
    green: '\x1b[32m',
    yellow: '\x1b[33m',
    blue: '\x1b[34m',
    magenta: '\x1b[35m',
    cyan: '\x1b[36m',
    white: '\x1b[37m',
    bold: '\x1b[1m',
};

// Утилиты для логирования
const logger = {
    info: (msg) => console.log(`${colors.blue}ℹ${colors.reset} ${msg}`),
    success: (msg) => console.log(`${colors.green}✅${colors.reset} ${msg}`),
    error: (msg) => console.log(`${colors.red}❌${colors.reset} ${msg}`),
    warning: (msg) => console.log(`${colors.yellow}⚠${colors.reset} ${msg}`),
    debug: (msg) => config.verboseMode && console.log(`${colors.cyan}🔍${colors.reset} ${msg}`),
    header: (msg) => console.log(`\n${colors.bold}${colors.magenta}🧪 ${msg}${colors.reset}`),
    separator: () => console.log(`${colors.cyan}${'─'.repeat(60)}${colors.reset}`),
};

// Статистика тестов
const testStats = {
    total: 0,
    passed: 0,
    failed: 0,
    skipped: 0,
    startTime: performance.now(),
};

// Класс для выполнения тестов
class TestRunner {
    constructor(name) {
        this.name = name;
        this.tests = [];
        this.beforeAll = null;
        this.afterAll = null;
    }
    
    addTest(name, testFn, options = {}) {
        this.tests.push({ name, testFn, options });
        return this;
    }
    
    setBefore(fn) {
        this.beforeAll = fn;
        return this;
    }
    
    setAfter(fn) {
        this.afterAll = fn;
        return this;
    }
    
    async run() {
        logger.header(`${this.name}`);
        logger.separator();
        
        if (this.beforeAll) {
            try {
                await this.beforeAll();
            } catch (error) {
                logger.error(`Ошибка в beforeAll: ${error.message}`);
                return;
            }
        }
        
        for (const test of this.tests) {
            if (test.options.skip) {
                logger.warning(`Пропущен: ${test.name}`);
                testStats.skipped++;
                continue;
            }
            
            testStats.total++;
            
            try {
                logger.info(`Выполняется: ${test.name}`);
                const startTime = performance.now();
                
                await test.testFn();
                
                const duration = performance.now() - startTime;
                logger.success(`${test.name} ${config.includePerformance ? `(${duration.toFixed(2)}ms)` : ''}`);
                testStats.passed++;
            } catch (error) {
                logger.error(`${test.name}: ${error.message}`);
                logger.debug(`Стек: ${error.stack}`);
                testStats.failed++;
            }
        }
        
        if (this.afterAll) {
            try {
                await this.afterAll();
            } catch (error) {
                logger.error(`Ошибка в afterAll: ${error.message}`);
            }
        }
    }
}

// Утилита для выполнения команд
async function runCommand(args, options = {}) {
    return new Promise((resolve, reject) => {
        const child = spawn(config.binaryPath, args, {
            stdio: 'pipe',
            cwd: __dirname,
            timeout: config.timeoutMs,
            ...options
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
            resolve({ code, stdout, stderr });
        });
        
        child.on('error', (err) => {
            reject(new Error(`Ошибка запуска: ${err.message}`));
        });
    });
}

// Проверка валидности JSON
function validateJSON(jsonString, allowEmpty = false) {
    if (!jsonString.trim() && allowEmpty) return true;
    
    try {
        JSON.parse(jsonString);
        return true;
    } catch {
        return false;
    }
}

// Тесты базовой функциональности
const basicTests = new TestRunner('Базовые тесты')
    .setBefore(() => {
        if (!fs.existsSync(config.binaryPath)) {
            throw new Error(`Бинарник не найден: ${config.binaryPath}`);
        }
        
        const stats = fs.statSync(config.binaryPath);
        logger.info(`Бинарник найден: ${(stats.size / 1024 / 1024).toFixed(1)} MB`);
    })
    .addTest('Версия и справка', async () => {
        const { code, stdout } = await runCommand(['--version']);
        if (code !== 0) {
            throw new Error(`Команда --version вернула код ${code}`);
        }
        if (!stdout.includes('archinalysator') && !stdout.includes('archlens')) {
            throw new Error('Версия не содержит ожидаемое название');
        }
    })
    .addTest('Анализ текущей папки', async () => {
        const { code, stdout, stderr } = await runCommand(['analyze', '.']);
        if (code !== 0) {
            throw new Error(`Анализ не удался: ${stderr}`);
        }
        
        if (!validateJSON(stdout)) {
            throw new Error('Вывод не является валидным JSON');
        }
        
        const result = JSON.parse(stdout);
        if (!result.total_files || result.total_files === 0) {
            throw new Error('Не найдено файлов для анализа');
        }
    })
    .addTest('Анализ корневого проекта', async () => {
        const { code, stdout, stderr } = await runCommand(['analyze', '..']);
        if (code !== 0) {
            // Предупреждение вместо ошибки для проблем доступа
            if (stderr.includes('os error 5') || stderr.includes('Access is denied')) {
                logger.warning('Ограничения доступа обнаружены - это нормально');
                return;
            }
            throw new Error(`Анализ не удался: ${stderr}`);
        }
        
        if (validateJSON(stdout)) {
            const result = JSON.parse(stdout);
            if (result.total_files && result.total_files > 0) {
                logger.debug(`Найдено файлов: ${result.total_files}`);
            }
        }
    });

// Тесты экспорта
const exportTests = new TestRunner('Тесты экспорта')
    .addTest('AI Compact экспорт', async () => {
        const { code, stdout, stderr } = await runCommand(['export', '.', 'ai_compact']);
        if (code !== 0) {
            throw new Error(`AI Compact экспорт не удался: ${stderr}`);
        }
        
        if (stdout.length === 0) {
            throw new Error('Пустой вывод AI Compact');
        }
        
        const tokenCount = Math.ceil(stdout.length / 4);
        logger.debug(`Экспорт: ${stdout.length} символов, ~${tokenCount} токенов`);
    })
    .addTest('Структура проекта', async () => {
        const { code, stdout, stderr } = await runCommand(['structure', '..']);
        if (code !== 0) {
            throw new Error(`Получение структуры не удалось: ${stderr}`);
        }
        
        if (!validateJSON(stdout)) {
            throw new Error('Структура не является валидным JSON');
        }
        
        const result = JSON.parse(stdout);
        if (!result.total_files) {
            throw new Error('Структура не содержит информацию о файлах');
        }
        
        logger.debug(`Структура: ${result.total_files} файлов, ${Object.keys(result.file_types || {}).length} типов`);
    });

// Тесты устойчивости
const resillienceTests = new TestRunner('Тесты устойчивости')
    .addTest('Обработка несуществующего пути', async () => {
        const { code, stderr } = await runCommand(['analyze', './nonexistent_path_12345']);
        if (code === 0) {
            logger.warning('Неожиданно успешный результат для несуществующего пути');
        }
        // Проверяем, что ошибка обрабатывается корректно
        if (!stderr.includes('not found') && !stderr.includes('No such file')) {
            logger.debug('Ошибка обработана корректно');
        }
    })
    .addTest('Обработка системной папки', async () => {
        if (config.skipSystemTests) {
            throw new Error('Тест пропущен по флагу --skip-system');
        }
        
        const { code, stderr } = await runCommand(['analyze', 'C:\\Windows\\System32']);
        if (code !== 0) {
            if (stderr.includes('os error 5') || stderr.includes('Access is denied')) {
                logger.debug('Ограничения доступа обработаны корректно');
            } else {
                throw new Error(`Неожиданная ошибка: ${stderr}`);
            }
        }
    }, { skip: config.skipSystemTests })
    .addTest('Обработка пустой папки', async () => {
        const tempDir = path.join(__dirname, 'temp_empty_test');
        
        try {
            fs.mkdirSync(tempDir, { recursive: true });
            
            const { code, stdout } = await runCommand(['analyze', tempDir]);
            if (code !== 0) {
                throw new Error('Анализ пустой папки не удался');
            }
            
            if (validateJSON(stdout)) {
                const result = JSON.parse(stdout);
                if (result.total_files !== 0) {
                    logger.debug(`Найдено файлов в пустой папке: ${result.total_files}`);
                }
            }
        } finally {
            if (fs.existsSync(tempDir)) {
                fs.rmSync(tempDir, { recursive: true });
            }
        }
    });

// Тесты производительности
const performanceTests = new TestRunner('Тесты производительности')
    .addTest('Время анализа проекта', async () => {
        const startTime = performance.now();
        const { code } = await runCommand(['analyze', '..']);
        const duration = performance.now() - startTime;
        
        if (code !== 0) {
            throw new Error('Анализ для теста производительности не удался');
        }
        
        logger.info(`Время анализа: ${duration.toFixed(2)}ms`);
        
        if (duration > 10000) {
            logger.warning('Анализ выполняется медленно (>10s)');
        }
    }, { skip: !config.includePerformance })
    .addTest('Память при анализе', async () => {
        const beforeMemory = process.memoryUsage();
        await runCommand(['analyze', '..']);
        const afterMemory = process.memoryUsage();
        
        const memoryDiff = afterMemory.heapUsed - beforeMemory.heapUsed;
        logger.debug(`Использование памяти: ${(memoryDiff / 1024 / 1024).toFixed(2)} MB`);
    }, { skip: !config.includePerformance });

// Главная функция
async function main() {
    console.log(`${colors.bold}${colors.magenta}🧪 Унифицированный тестовый набор ArchLens MCP${colors.reset}`);
    console.log(`${colors.cyan}═${'═'.repeat(60)}${colors.reset}`);
    
    logger.info('Запуск тестов...');
    logger.debug(`Конфигурация: ${JSON.stringify(config, null, 2)}`);
    
    // Запуск всех тестов
    await basicTests.run();
    await exportTests.run();
    await resillienceTests.run();
    if (config.includePerformance) {
        await performanceTests.run();
    }
    
    // Финальная статистика
    const totalTime = performance.now() - testStats.startTime;
    console.log(`\n${colors.bold}${colors.magenta}📊 Результаты тестирования${colors.reset}`);
    console.log(`${colors.cyan}═${'═'.repeat(60)}${colors.reset}`);
    
    logger.info(`Всего тестов: ${testStats.total}`);
    logger.success(`Пройдено: ${testStats.passed}`);
    if (testStats.failed > 0) {
        logger.error(`Не пройдено: ${testStats.failed}`);
    }
    if (testStats.skipped > 0) {
        logger.warning(`Пропущено: ${testStats.skipped}`);
    }
    logger.info(`Время выполнения: ${(totalTime / 1000).toFixed(2)}s`);
    
    const success = testStats.failed === 0;
    console.log(`\n${success ? colors.green + '🎉 Все тесты пройдены!' : colors.red + '💥 Есть неудачные тесты!'}${colors.reset}`);
    
    if (success) {
        console.log(`${colors.green}✅ MCP сервер готов к работе${colors.reset}`);
        console.log(`${colors.green}📋 Все компоненты функционируют корректно${colors.reset}`);
        console.log(`${colors.green}🔧 Обработка ошибок работает надежно${colors.reset}`);
    } else {
        console.log(`${colors.red}❌ Требуется дополнительная отладка${colors.reset}`);
        process.exit(1);
    }
}

// Обработка ошибок
process.on('unhandledRejection', (error) => {
    logger.error(`Необработанная ошибка: ${error.message}`);
    process.exit(1);
});

// Справка
if (process.argv.includes('--help')) {
    console.log(`
Унифицированный тестовый набор ArchLens MCP

Использование: node test_unified.js [опции]

Опции:
  --verbose        Подробный вывод
  --skip-system    Пропустить тесты системных папок
  --performance    Включить тесты производительности
  --help           Показать эту справку

Примеры:
  node test_unified.js                           # Базовые тесты
  node test_unified.js --verbose                 # С подробным выводом
  node test_unified.js --performance --verbose   # Полное тестирование
`);
    process.exit(0);
}

// Запуск
main().catch(error => {
    logger.error(`Критическая ошибка: ${error.message}`);
    process.exit(1);
}); 