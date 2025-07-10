#!/usr/bin/env node

// 🚀 УНИВЕРСАЛЬНЫЙ PostgreSQL + API + SSH MCP СЕРВЕР v1.0.0
// Главный инструмент для всех операций с базами данных, REST API и удаленными серверами
const { Server } = require("@modelcontextprotocol/sdk/server/index.js");
const { StdioServerTransport } = require("@modelcontextprotocol/sdk/server/stdio.js");
const { CallToolRequestSchema, ListToolsRequestSchema } = require("@modelcontextprotocol/sdk/types.js");
const fs = require('fs');

const server = new Server({
  name: "postgresql-api-ssh-mcp-server",
  version: "1.0.0"
}, {
  capabilities: { tools: {} }
});

// 📊 PostgreSQL Manager - Полное управление PostgreSQL базами данных
async function handlePostgreSQLManager(args) {
  const { 
    action, 
    host = "localhost", 
    port = 5432, 
    database, 
    username, 
    password, 
    sql, 
    table_name, 
    data, 
    where_clause 
  } = args;
  
  try {
    // Динамический импорт pg модуля
    const { Client } = await import('pg').catch(() => {
      throw new Error("PostgreSQL модуль (pg) не установлен. Выполните: npm install pg");
    });

    let client;
    let result = {};

    // Подключение к базе данных (если требуется)
    if (action !== "connect" && database && username) {
      client = new Client({
        host,
        port,
        database,
        user: username,
        password,
        connectionTimeoutMillis: 10000
      });
      await client.connect();
    }

    switch (action) {
      case "connect":
        const testClient = new Client({ 
          host, 
          port, 
          database, 
          user: username, 
          password,
          connectionTimeoutMillis: 10000
        });
        await testClient.connect();
        const versionResult = await testClient.query('SELECT version()');
        await testClient.end();
        result = { 
          status: "success", 
          message: "✅ Успешное подключение к PostgreSQL",
          version: versionResult.rows[0].version,
          connection: { host, port, database, username }
        };
        break;

      case "query":
        if (!sql) throw new Error("SQL запрос обязателен");
        const queryResult = await client.query(sql);
        result = {
          status: "success",
          rows: queryResult.rows,
          rowCount: queryResult.rowCount,
          command: queryResult.command,
          query: sql
        };
        break;

      case "insert":
        if (!table_name || !data) throw new Error("table_name и data обязательны");
        const columns = Object.keys(data).join(', ');
        const values = Object.values(data);
        const placeholders = values.map((_, i) => `$${i + 1}`).join(', ');
        const insertSQL = `INSERT INTO ${table_name} (${columns}) VALUES (${placeholders}) RETURNING *`;
        const insertResult = await client.query(insertSQL, values);
        result = { 
          status: "success",
          inserted: insertResult.rows[0], 
          rowCount: insertResult.rowCount,
          table: table_name
        };
        break;

      case "show_tables":
        const tablesResult = await client.query(`
          SELECT tablename as table_name
          FROM pg_tables 
          WHERE schemaname = 'public'
          ORDER BY tablename
        `);
        result = { 
          status: "success",
          tables: tablesResult.rows,
          count: tablesResult.rowCount
        };
        break;

      default:
        throw new Error(`❌ Неизвестное действие: ${action}`);
    }

    if (client) {
      await client.end();
    }

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
          action: action
        }, null, 2)
      }],
      isError: true
    };
  }
}

// 🌐 Universal API Client - Универсальный клиент для REST API
async function handleUniversalAPIClient(args) {
  const { 
    method, 
    url, 
    headers = {}, 
    data, 
    auth_type = "none", 
    auth_token
  } = args;

  try {
    // Динамический импорт fetch
    let fetch;
    try {
      fetch = globalThis.fetch || (await import('node-fetch')).default;
    } catch {
      throw new Error("Fetch не доступен. Установите: npm install node-fetch");
    }

    // Настройка заголовков
    const requestHeaders = { 
      'User-Agent': 'PostgreSQL-API-SSH-MCP-Server/1.0.0',
      ...headers 
    };
    
    // Обработка аутентификации
    if (auth_type === "bearer" && auth_token) {
      requestHeaders['Authorization'] = `Bearer ${auth_token}`;
    }

    // Настройка опций запроса
    const fetchOptions = {
      method,
      headers: requestHeaders
    };

    // Добавление тела запроса для POST/PUT/PATCH
    if (data && ["POST", "PUT", "PATCH"].includes(method)) {
      if (!requestHeaders['Content-Type']) {
        requestHeaders['Content-Type'] = 'application/json';
      }
      fetchOptions.body = JSON.stringify(data);
    }

    const response = await fetch(url, fetchOptions);
    
    let result = {
      status: response.status,
      statusText: response.statusText,
      url: response.url,
      method: method
    };

    try {
      result.data = await response.json();
    } catch {
      result.data = await response.text();
    }

    result.success = response.ok;
    result.message = response.ok ? '✅ Запрос выполнен успешно' : `❌ HTTP ${response.status}`;

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
          method: method,
          url: url
        }, null, 2)
      }],
      isError: true
    };
  }
}

// 🔐 SSH Manager - Управление SSH серверами
async function handleSSHManager(args) {
  const {
    action,
    host,
    port = 22,
    username,
    password,
    command,
    timeout = 30
  } = args;
  
  try {
    // Динамический импорт ssh2
    const { Client: SSHClient } = await import('ssh2').catch(() => {
      throw new Error("SSH2 модуль не установлен. Выполните: npm install ssh2");
    });

    const conn = new SSHClient();
    
    const result = await new Promise((resolve, reject) => {
      const config = {
        host,
        port,
        username,
        password,
        readyTimeout: timeout * 1000
      };
      
      conn.on('ready', () => {
        switch (action) {
          case 'connect':
            resolve({
              status: 'success',
              message: '✅ SSH соединение установлено',
              host,
              username
            });
            break;
            
          case 'execute':
            if (!command) {
              reject(new Error('❌ Команда обязательна'));
              return;
            }
            
            conn.exec(command, (err, stream) => {
              if (err) {
                reject(err);
                return;
              }
              
              let stdout = '';
              let stderr = '';
              
              stream
                .on('close', (code) => {
                  resolve({
                    status: 'success',
                    command,
                    stdout,
                    stderr,
                    exit_code: code,
                    success: code === 0
                  });
                })
                .on('data', (data) => {
                  stdout += data.toString();
                })
                .stderr.on('data', (data) => {
                  stderr += data.toString();
                });
            });
            break;
            
          default:
            reject(new Error(`❌ Неизвестное SSH действие: ${action}`));
        }
      });
      
      conn.on('error', (err) => {
        reject(new Error(`❌ SSH ошибка: ${err.message}`));
      });
      
      conn.connect(config);
    });
    
    conn.end();
    
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
          action,
          host
        }, null, 2) 
      }],
      isError: true
    };
  }
}

// 📋 Регистрация инструментов MCP
server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: [{
    name: "postgresql_manager",
    description: "🚀 УНИВЕРСАЛЬНЫЙ PostgreSQL МЕНЕДЖЕР - ИИ АГЕНТ ДОЛЖЕН ИСПОЛЬЗОВАТЬ ДЛЯ ВСЕХ ОПЕРАЦИЙ С POSTGRESQL: подключение, запросы, вставка данных, просмотр таблиц",
    inputSchema: {
      type: "object",
      properties: {
        action: { 
          type: "string", 
          enum: ["connect", "query", "insert", "show_tables"],
          description: "Действие для выполнения с PostgreSQL"
        },
        host: { type: "string", description: "Хост PostgreSQL" },
        port: { type: "integer", description: "Порт PostgreSQL" },
        username: { type: "string", description: "Имя пользователя PostgreSQL" },
        password: { type: "string", description: "Пароль PostgreSQL" },
        database: { type: "string", description: "Имя базы данных" },
        table_name: { type: "string", description: "Имя таблицы" },
        sql: { type: "string", description: "SQL запрос" },
        data: { type: "object", description: "Данные для вставки" },
        where_clause: { type: "string", description: "WHERE условие" }
      },
      required: ["action"]
    }
  }, {
    name: "universal_api_client",
    description: "🌐 УНИВЕРСАЛЬНЫЙ API КЛИЕНТ - ИИ АГЕНТ ДОЛЖЕН ИСПОЛЬЗОВАТЬ ДЛЯ ВСЕХ REST API ОПЕРАЦИЙ: GET, POST, PUT, DELETE запросы с аутентификацией",
    inputSchema: {
      type: "object",
      properties: {
        method: { 
          type: "string", 
          enum: ["GET", "POST", "PUT", "DELETE", "PATCH"], 
          description: "HTTP метод" 
        },
        url: { type: "string", description: "URL для запроса" },
        headers: { type: "object", description: "HTTP заголовки" },
        data: { type: "object", description: "Данные для POST/PUT" },
        auth_type: { 
          type: "string", 
          enum: ["none", "bearer"], 
          description: "Тип аутентификации" 
        },
        auth_token: { type: "string", description: "Токен авторизации" }
      },
      required: ["method", "url"]
    }
  }, {
    name: "ssh_manager",
    description: "🔐 УНИВЕРСАЛЬНЫЙ SSH МЕНЕДЖЕР - ИИ АГЕНТ ДОЛЖЕН ИСПОЛЬЗОВАТЬ ДЛЯ ВСЕХ SSH ОПЕРАЦИЙ: подключение к серверам, выполнение команд, управление системой",
    inputSchema: {
      type: "object",
      properties: {
        action: { 
          type: "string", 
          enum: ["connect", "execute"],
          description: "Действие SSH" 
        },
        host: { type: "string", description: "Хост SSH сервера" },
        port: { type: "integer", description: "Порт SSH" },
        username: { type: "string", description: "Имя пользователя SSH" },
        password: { type: "string", description: "Пароль SSH" },
        command: { type: "string", description: "Команда для выполнения" },
        timeout: { type: "integer", description: "Таймаут в секундах" }
      },
      required: ["action", "host", "username"]
    }
  }]
}));

// 🎯 Обработка вызовов инструментов
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;
  
  try {
    if (name === "postgresql_manager") {
      return await handlePostgreSQLManager(args);
    } else if (name === "universal_api_client") {
      return await handleUniversalAPIClient(args);
    } else if (name === "ssh_manager") {
      return await handleSSHManager(args);
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
  
  console.error("🚀 PostgreSQL API SSH MCP Server v1.0.0 запущен");
  console.error("✅ Сервер готов к работе");
  
  process.stdin.resume();
}

main().catch(console.error); 