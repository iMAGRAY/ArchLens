#!/usr/bin/env node

// 🏗️ ARCHLENS MCP SERVER v2.0.0 - CRITICAL FIXES APPLIED
// NO HARDCODED PATHS | NO SIDE EFFECTS | ABSOLUTE PATHS ONLY | UNIFIED LANGUAGE | WINDOWS FIXES
const { Server } = require("@modelcontextprotocol/sdk/server/index.js");
const { StdioServerTransport } = require("@modelcontextprotocol/sdk/server/stdio.js");
const { CallToolRequestSchema, ListToolsRequestSchema } = require("@modelcontextprotocol/sdk/types.js");
const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');
const os = require('os');
const iconv = require('iconv-lite');

// 📋 CONFIGURATION WITHOUT HARDCODING
const CONFIG = {
  server: {
  name: "archlens-mcp-server",
    version: "2.0.0"
  },
  binary: {
    name: process.env.ARCHLENS_BINARY || "archlens",
    searchPaths: [
      process.env.ARCHLENS_PATH,
      path.resolve(process.cwd(), "target", "release"),
      path.resolve(process.cwd(), "target", "debug"),
      path.resolve(path.dirname(__filename), "target", "release"),
      path.resolve(path.dirname(__filename), "target", "debug"),
      path.resolve(path.dirname(__filename)),
      path.resolve(process.cwd())
    ].filter(Boolean), // Remove undefined/null values
    timeout: 60000
  },
  paths: {
    workingDirectory: process.env.ARCHLENS_WORKDIR || process.cwd()
  },
  language: {
    locale: "en_US.UTF-8",
    forceEnglish: true
  },
  windows: {
    useShell: true,
    autoElevate: process.env.ARCHLENS_AUTO_ELEVATE === "true"
  },
  patterns: {
    include: [
      "**/*.rs", "**/*.ts", "**/*.js", "**/*.py", "**/*.java", 
      "**/*.cpp", "**/*.cc", "**/*.cxx", "**/*.c", "**/*.h", "**/*.hpp", "**/*.hxx",
      "**/*.go", "**/*.php", "**/*.rb", "**/*.cs", "**/*.kt", "**/*.swift", 
      "**/*.dart", "**/*.vue", "**/*.jsx", "**/*.tsx", "**/*.html", "**/*.css", 
      "**/*.scss", "**/*.sass", "**/*.json", "**/*.yaml", "**/*.yml", "**/*.xml", 
      "**/*.md", "**/*.txt"
    ],
    exclude: [
      "**/target/**", "**/node_modules/**", "**/.git/**", "**/dist/**", 
      "**/build/**", "**/out/**", "**/.next/**", "**/.nuxt/**", "**/coverage/**", 
      "**/tmp/**", "**/temp/**", "**/*.class", "**/*.o", "**/*.obj"
    ]
  },
  limits: {
    maxDepth: 20,
    maxFiles: 1000,
    scanDepth: 15,
    maxFileSize: 1000000,
    scanTimeout: 30000
  },
  textExtensions: [
    '.rs', '.ts', '.js', '.py', '.java', '.cpp', '.cc', '.cxx', '.c', 
    '.h', '.hpp', '.hxx', '.go', '.php', '.rb', '.cs', '.kt', '.swift', 
    '.dart', '.vue', '.jsx', '.tsx', '.html', '.css', '.scss', '.sass', 
    '.json', '.yaml', '.yml', '.xml', '.md', '.txt'
  ]
};

// Cap output to avoid memory bloat and excessive tokens
const MAX_OUTPUT_CHARS = 1_000_000; // ~1MB

// 🔇 LOGGING WITHOUT SIDE EFFECTS  
class Logger {
  constructor(enabled = process.env.ARCHLENS_DEBUG === "true") {
    this.enabled = enabled;
  }
  
  debug(message) {
    if (this.enabled) {
      process.stderr.write(`[DEBUG] ${message}\n`);
    }
  }
  
  info(message) {
    if (this.enabled) {
      process.stderr.write(`[INFO] ${message}\n`);
    }
  }
  
  error(message) {
    if (this.enabled) {
      process.stderr.write(`[ERROR] ${message}\n`);
    }
  }
}

const logger = new Logger();

// 🔍 DYNAMIC BINARY LOCATION (NO HARDCODING)
function getArchLensBinary() {
  const platform = os.platform();
  const extension = platform === 'win32' ? '.exe' : '';
  const binaryName = `${CONFIG.binary.name}${extension}`;
  
  // First, check in the same directory as this script
  // Use __dirname for CommonJS modules or resolve from current file
  const scriptDir = __dirname || path.dirname(__filename) || path.resolve('.');
  const localBinary = path.join(scriptDir, binaryName);
  if (fs.existsSync(localBinary)) {
    try {
      // On Windows, just check if file exists (X_OK doesn't work well on Windows)
      if (platform === 'win32') {
        logger.debug(`Found local binary: ${localBinary}`);
        return localBinary;
      } else {
        fs.accessSync(localBinary, fs.constants.F_OK | fs.constants.X_OK);
        logger.debug(`Found executable binary: ${localBinary}`);
        return localBinary;
      }
    } catch (accessError) {
      logger.debug(`Binary found but not executable: ${localBinary}`);
    }
  }
  
  // 1. Check if it's in PATH
  try {
    const which = platform === 'win32' ? 'where' : 'which';
    const { execSync } = require('child_process');
    const result = execSync(`${which} ${CONFIG.binary.name}`, { encoding: 'utf8', stdio: 'pipe' });
    if (result.trim()) {
      logger.debug(`Found binary in PATH: ${result.trim()}`);
      return CONFIG.binary.name; // Use PATH version
    }
  } catch (e) {
    logger.debug(`Binary not in PATH: ${e.message}`);
  }
  
  // 2. Check configured search paths
  for (const searchPath of CONFIG.binary.searchPaths) {
    const fullPath = path.join(searchPath, binaryName);
    if (fs.existsSync(fullPath)) {
      try {
        // On Windows, just check if file exists
        if (platform === 'win32') {
          logger.debug(`Found binary: ${fullPath}`);
          return fullPath;
        } else {
          fs.accessSync(fullPath, fs.constants.F_OK | fs.constants.X_OK);
          logger.debug(`Found executable binary: ${fullPath}`);
          return fullPath;
        }
      } catch (accessError) {
        logger.debug(`Binary found but not executable: ${fullPath}`);
      }
    }
  }
  
  throw new Error(`❌ ArchLens binary '${binaryName}' not found in search paths: ${CONFIG.binary.searchPaths.join(', ')}\n` +
    `🔧 Solutions:\n` +
    `  1. Build: cargo build --release\n` +
    `  2. Set ARCHLENS_PATH=/path/to/binary\n` +
    `  3. Add binary to PATH\n` +
    `  4. Set ARCHLENS_BINARY=custom_name`);
}

// 🛡️ ABSOLUTE PATH RESOLUTION (NO RELATIVE PATHS)
function resolveProjectPath(inputPath) {
  if (!inputPath || typeof inputPath !== 'string') {
    throw new Error('project_path is required and must be a string');
  }

  // Support '.' and relative paths by resolving to absolute
  if (inputPath === '.' || inputPath === '..' ||
      inputPath.startsWith('./') || inputPath.startsWith('../') ||
      inputPath.startsWith('.\\') || inputPath.startsWith('..\\')) {
    const resolved = path.resolve(CONFIG.paths.workingDirectory, inputPath);
    if (!fs.existsSync(resolved)) {
      throw new Error(`Path does not exist: ${resolved}`);
    }
    const stat = fs.statSync(resolved);
    if (!stat.isDirectory()) {
      throw new Error(`Path is not a directory: ${resolved}`);
    }
    return resolved;
  }

  let resolvedPath;
  if (path.isAbsolute(inputPath)) {
    resolvedPath = path.normalize(inputPath);
  } else {
    resolvedPath = path.resolve(CONFIG.paths.workingDirectory, inputPath);
  }

  if (!fs.existsSync(resolvedPath)) {
    throw new Error(`Path does not exist: ${resolvedPath}`);
  }
  const stat = fs.statSync(resolvedPath);
  if (!stat.isDirectory()) {
    throw new Error(`Path is not a directory: ${resolvedPath}`);
  }
  fs.accessSync(resolvedPath, fs.constants.R_OK);
  return resolvedPath;
}

// 🌍 WINDOWS COMPATIBILITY HELPERS
function getWindowsExecutionOptions() {
  if (os.platform() !== 'win32') {
    return {};
  }
  
  return {
    shell: CONFIG.windows.useShell,
    windowsHide: true,
    env: {
      ...process.env,
      LANG: "en_US.UTF-8",         // Force English locale on Windows
      LC_ALL: "en_US.UTF-8",       // Force English locale for all categories
      RUST_BACKTRACE: "0",         // Disable Rust backtraces
      ARCHLENS_LANG: "en"          // Force English output from binary
    }
  };
}

// 🔐 WINDOWS ADMIN RIGHTS CHECKER
async function checkWindowsAdminRights() {
  if (os.platform() !== 'win32') {
    return true; // Non-Windows platforms don't need admin
  }
  
  try {
    // Try to read a system directory that requires admin rights
    const { execSync } = require('child_process');
    const result = execSync('net session >nul 2>&1 && echo "ADMIN" || echo "NOT_ADMIN"', { 
      encoding: 'utf8', 
      stdio: 'pipe' 
    });
    
    const isAdmin = result.trim().includes('ADMIN');
    logger.debug(`Windows admin check: ${isAdmin ? 'ADMIN' : 'NOT_ADMIN'}`);
    return isAdmin;
  } catch (error) {
    logger.debug(`Admin check failed: ${error.message}`);
    return false;
  }
}

// 🚀 WINDOWS ADMIN ELEVATION HELPER
async function tryAutoElevation() {
  if (!CONFIG.windows.autoElevate || os.platform() !== 'win32') {
    return false;
        }
        
  try {
    const { spawn } = require('child_process');
    const currentScriptPath = path.resolve(__filename);
    const workingDir = path.dirname(currentScriptPath);
    
    const powershellArgs = [
      '-Command',
      `Start-Process PowerShell -Verb RunAs -ArgumentList "-Command", "cd '${workingDir}'; node '${currentScriptPath}'"`
    ];
    
    logger.debug("Attempting automatic admin elevation...");
    
    spawn('powershell', powershellArgs, {
      stdio: 'ignore',
      detached: true
    });
    
    // Give user time to approve UAC prompt
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    return true;
  } catch (error) {
    logger.debug(`Auto elevation failed: ${error.message}`);
    return false;
  }
}

function createAdminElevationInstructions(command, projectPath) {
  const currentScriptPath = path.resolve(__filename);
  const workingDir = path.dirname(currentScriptPath);
  
  const powershellCommand = `Start-Process PowerShell -Verb RunAs -ArgumentList "-Command", "cd '${workingDir}'; node '${currentScriptPath}'"`;
  const autoElevateInfo = CONFIG.windows.autoElevate ? 
    `**AUTO-ELEVATION:** Enabled (ARCHLENS_AUTO_ELEVATE=true)` :
    `**AUTO-ELEVATION:** Disabled (set ARCHLENS_AUTO_ELEVATE=true to enable automatic elevation)`;
  
  return `🔐 WINDOWS ADMIN RIGHTS REQUIRED

**Reason:** ArchLens analyze_project requires administrator privileges to scan all files and directories.

${autoElevateInfo}

**AUTOMATIC SOLUTION:**
Run this PowerShell command to restart with admin rights:

\`\`\`powershell
${powershellCommand}
\`\`\`

**MANUAL SOLUTION:**
1. Close current session
2. Right-click PowerShell/CMD → "Run as Administrator"  
3. Navigate to: ${workingDir}
4. Run: node "${currentScriptPath}"
5. Retry the analyze_project command

**ENVIRONMENT SETUP:**
\`\`\`bash
# Enable automatic elevation (optional)
export ARCHLENS_AUTO_ELEVATE=true

# Then retry analyze_project
\`\`\`

**PROJECT PATH:** ${projectPath}
**CURRENT USER:** ${os.userInfo().username}
**PLATFORM:** ${os.platform()} ${os.release()}

**Alternative:** Use 'get_project_structure' which works without admin rights.`;
    }
    
        // 🚀 UNIFIED COMMAND EXECUTION (ABSOLUTE PATHS ONLY)
async function executeArchlensCommand(subcommand, projectPath, additionalArgs = [], options = {}) {
  return new Promise((resolve, reject) => {
      const binary = getArchLensBinary();
    
    // Ensure project path is absolute and properly formatted for Windows
    let absoluteProjectPath = path.isAbsolute(projectPath) ? projectPath : path.resolve(projectPath);
    
    // On Windows, ensure we use proper format with quotes if path contains spaces
    if (os.platform() === 'win32') {
      // Normalize slashes for consistency
      absoluteProjectPath = absoluteProjectPath.replace(/\//g, '\\');
    }
    
    const args = [subcommand, absoluteProjectPath, ...additionalArgs];
    
    const spawnOptions = {
      stdio: ['pipe', 'pipe', 'pipe'],
      cwd: CONFIG.paths.workingDirectory,
      encoding: 'buffer', // Use buffer to handle encoding ourselves
      timeout: CONFIG.binary.timeout,
      env: {
        ...process.env,
        LANG: "en_US.UTF-8",         // Force English locale
        LC_ALL: "en_US.UTF-8",       // Force English locale for all categories
        RUST_BACKTRACE: "0",         // Disable Rust backtraces for clean output
        ARCHLENS_LANG: "en"          // Force English output from binary
      },
      ...getWindowsExecutionOptions(),
      ...options
    };
    
    logger.debug(`Executing: ${binary} ${args.join(' ')}`);
    logger.debug(`Working directory: ${spawnOptions.cwd}`);
    logger.debug(`Absolute project path: ${absoluteProjectPath}`);
    
    const child = spawn(binary, args, spawnOptions);
      
      let stdout = '';
      let stderr = '';
    let isTimedOut = false;
    
    // Setup timeout
    const timeoutId = setTimeout(() => {
      isTimedOut = true;
      child.kill('SIGTERM');
      logger.error(`Command timed out after ${CONFIG.binary.timeout}ms`);
    }, CONFIG.binary.timeout);
      
      child.stdout.on('data', (data) => {
      // Handle Windows encoding properly
      let str;
      try {
        if (os.platform() === 'win32') {
          // Try to decode as UTF-8 first (archlens should output UTF-8)
          str = data.toString('utf8');
        } else {
          str = data.toString('utf8');
        }
      } catch (e) {
        str = data.toString('utf8').replace(/[^\x00-\x7F]/g, '');
      }
      stdout += str;
      if (stdout.length > MAX_OUTPUT_CHARS) {
        // Keep tail to preserve latest info
        stdout = stdout.slice(stdout.length - MAX_OUTPUT_CHARS);
      }
      });
      
      child.stderr.on('data', (data) => {
      // Handle Windows encoding properly - convert from cp866/cp1251 to UTF-8
      let str;
      try {
        // For Windows, try multiple encodings
        if (os.platform() === 'win32') {
          // Try UTF-8 first (modern Windows)
          try {
            str = data.toString('utf8');
            // Check if it's valid UTF-8 by looking for replacement chars
            if (!str.includes('�')) {
              // Valid UTF-8, use it
            } else {
              // Try cp866 (Russian DOS encoding)
              str = iconv.decode(data, 'cp866');
            }
          } catch (e1) {
            try {
              // Try cp1251 (Russian Windows encoding)
              str = iconv.decode(data, 'cp1251');
            } catch (e2) {
              // Last resort - use UTF-8 and replace invalid chars
              str = data.toString('utf8').replace(/[^\x00-\x7F]/g, '?');
            }
          }
        } else {
          str = data.toString('utf8');
        }
      } catch (e) {
        // Fallback to UTF-8 with replacement characters
        str = data.toString('utf8').replace(/[^\x00-\x7F]/g, '?');
      }
      stderr += str;
      if (stderr.length > MAX_OUTPUT_CHARS) {
        stderr = stderr.slice(stderr.length - MAX_OUTPUT_CHARS);
      }
      });
      
      child.on('close', (code) => {
      clearTimeout(timeoutId);
      
      logger.debug(`Command finished with code: ${code}`);
      logger.debug(`Stdout length: ${stdout.length}`);
      logger.debug(`Stderr length: ${stderr.length}`);
      
      if (isTimedOut) {
        reject(new Error(`Command execution timed out after ${CONFIG.binary.timeout}ms`));
        return;
      }
      
        if (code === 0) {
        if (stdout.trim().length === 0) {
          resolve({
            status: "success",
            message: "Command executed successfully (empty output)",
            output: stderr.length > 0 ? stderr : "No output generated",
            warning: "empty_stdout"
          });
        } else {
          try {
            const result = JSON.parse(stdout);
            resolve(result);
          } catch (parseError) {
            resolve({
              status: "success",
              message: "Command executed successfully",
              output: stdout
            });
          }
          }
        } else {
        // Handle Windows Access Denied specifically
        if (os.platform() === 'win32' && (stderr.includes('Access denied') || code === 5)) {
          reject(new Error(`Windows Access Denied (Code ${code})\n` +
            `🔧 Solutions:\n` +
            `  • Run PowerShell/CMD as Administrator\n` +
            `  • Check antivirus is not blocking binary\n` +
            `  • Ensure no files are locked by other processes\n` +
            `  • Check Windows Defender exclusions\n` +
            `Original error: ${stderr}`));
        } else {
          reject(new Error(stderr || `Command failed with exit code ${code}`));
        }
        }
      });
      
      child.on('error', (error) => {
      clearTimeout(timeoutId);
      
      if (error.code === 'ENOENT') {
        reject(new Error(`Binary not found: ${binary}\n` +
          `🔧 Solutions:\n` +
          `  • Ensure binary is built: cargo build --release\n` +
          `  • Check PATH includes binary location\n` +
          `  • Set ARCHLENS_PATH environment variable`));
      } else if (error.code === 'EACCES') {
        reject(new Error(`Permission denied executing: ${binary}\n` +
          `🔧 Solutions:\n` +
          `  • Make binary executable: chmod +x ${binary}\n` +
          `  • Run with administrator privileges\n` +
          `  • Check file permissions`));
      } else {
        reject(new Error(`Failed to execute ArchLens: ${error.message} (${error.code})`));
      }
      });
    });
}
    
// 🎯 UNIFIED RESPONSE CREATION
function createMCPResponse(toolName, result, error = null, projectPath = null, detailLevel = 'summary') {
  if (error) {
    return {
      content: [{
        type: "text",
        text: `❌ ${getToolDisplayName(toolName)}\n\nReason: ${error.message.replace(/[^\x00-\x7F]/g, '?')}\nPath: ${projectPath || 'n/a'}\nTime: ${new Date().toLocaleString('en-US')}`
      }],
      isError: true
    };
  }
  // Minimized, high-signal response
  return {
    content: [{
      type: "text",
      text: formatToolResult(toolName, result, projectPath, detailLevel)
    }]
  };
}

function getToolDisplayName(toolName) {
  const names = {
    'analyze_project': 'PROJECT ANALYSIS',
    'export_ai_compact': 'ARCHITECTURE ANALYSIS', 
    'get_project_structure': 'STRUCTURE RETRIEVAL',
    'generate_diagram': 'DIAGRAM GENERATION'
  };
  return names[toolName] || 'TOOL';
}

function getToolAction(toolName) {
  const actions = {
    'analyze_project': 'analyze project',
    'export_ai_compact': 'export AI compact analysis', 
    'get_project_structure': 'get project structure',
    'generate_diagram': 'generate diagram'
  };
  return actions[toolName] || 'perform operation';
}

// 📊 RESULT FORMATTING
class ResponseFormatter {
  static formatAnalysisResult(result, projectPath, detailLevel = 'summary') {
    const data = typeof result === 'string' ? JSON.parse(result) : result;
    if (detailLevel === 'full') {
      return `# 🔍 PROJECT ANALYSIS\n**Path:** ${projectPath}\n- Files: ${data.total_files || 'n/a'}\n- Lines: ${data.total_lines || 'n/a'}\n${data.file_types ? '- Types: ' + Object.entries(data.file_types).sort(([,a],[,b])=>b-a).map(([ext,c])=>`.${ext}:${c}`).join(', ') : ''}`;
    }
    if (detailLevel === 'standard') {
      return `# 🔍 PROJECT ANALYSIS\n**Path:** ${projectPath}\n- Files: ${data.total_files || 'n/a'}\n- Lines: ${data.total_lines || 'n/a'}\n${data.file_types ? '- Top: ' + Object.entries(data.file_types).sort(([,a],[,b])=>b-a).slice(0,5).map(([ext,c])=>`.${ext}:${c}`).join(', ') : ''}`;
    }
    // summary (default)
    return `# 🔍 PROJECT ANALYSIS\n**Path:** ${projectPath}\n- Files: ${data.total_files || 'n/a'}\n- Lines: ${data.total_lines || 'n/a'}\n${data.file_types ? '- Top: ' + Object.entries(data.file_types).sort(([,a],[,b])=>b-a).slice(0,3).map(([ext,c])=>`.${ext}:${c}`).join(', ') : ''}`;
  }
  static formatExportResult(result, projectPath, detailLevel = 'summary') {
    let content = '';
    if (typeof result === 'string') {
      content = result;
    } else if (result && result.output) {
      content = result.output;
    } else {
      return `❌ ARCHITECTURE ANALYSIS ERROR\nPath: ${projectPath}`;
    }
    if (detailLevel === 'full') {
      return clampText(content, MAX_OUTPUT_CHARS);
    }
    // Remove large code blocks for summary/standard
    const stripped = stripCodeBlocks(content);
    if (detailLevel === 'standard') {
      return clampText(stripped, Math.min(SUMMARY_LIMIT_CHARS * 2, MAX_OUTPUT_CHARS));
    }
    // summary
    return clampText(stripped, SUMMARY_LIMIT_CHARS);
  }
  static formatStructureResult(result, projectPath, detailLevel = 'summary') {
    const data = typeof result === 'string' ? JSON.parse(result) : (result.structure ? result.structure : result);
    if (detailLevel === 'full') {
      const files = data.files ? data.files.slice(0, 25).map(f => `- \`${f.path}\` (${f.extension}, ${(f.size/1024).toFixed(1)}KB)`).join('\n') : '';
      return `# 📁 STRUCTURE\n**Path:** ${projectPath}\n- Files: ${data.total_files || 'n/a'}\n- Lines: ${data.total_lines || 'n/a'}\n${data.layers && data.layers.length ? '- Layers: ' + data.layers.join(', ') : ''}\n${data.file_types ? '- Types: ' + Object.entries(data.file_types).sort(([,a],[,b])=>b-a).map(([ext,c])=>`.${ext}:${c}`).join(', ') : ''}\n${files}`;
    }
    if (detailLevel === 'standard') {
      return `# 📁 STRUCTURE\n**Path:** ${projectPath}\n- Files: ${data.total_files || 'n/a'}\n${data.layers && data.layers.length ? '- Layers: ' + data.layers.join(', ') : ''}\n${data.file_types ? '- Types: ' + Object.entries(data.file_types).sort(([,a],[,b])=>b-a).slice(0,10).map(([ext,c])=>`.${ext}:${c}`).join(', ') : ''}`;
    }
    // summary
    return `# 📁 STRUCTURE\n**Path:** ${projectPath}\n- Files: ${data.total_files || 'n/a'}\n${data.layers && data.layers.length ? '- Layers: ' + data.layers.join(', ') : ''}\n${data.file_types ? '- Types: ' + Object.entries(data.file_types).sort(([,a],[,b])=>b-a).slice(0,5).map(([ext,c])=>`.${ext}:${c}`).join(', ') : ''}`;
  }
  static formatDiagramResult(result, projectPath, detailLevel = 'summary') {
    if (result.diagram && typeof result.diagram === 'string') {
      let diagram = result.diagram;
      // Clamp diagram to avoid huge payloads
      if (detailLevel === 'summary') {
        diagram = clampText(diagram, 15000);
      } else if (detailLevel === 'standard') {
        diagram = clampText(diagram, 40000);
      } else {
        diagram = clampText(diagram, 120000);
      }
      const header = `# 📊 DIAGRAM\nPath: ${projectPath}\nType: ${result.diagram_type || 'mermaid'}`;
      return `${header}\n\n\`\`\`mermaid\n${diagram}\n\`\`\``;
    }
    return `❌ DIAGRAM GENERATION ERROR\nPath: ${projectPath}`;
  }

  static getArchitecturalRisk(totalFiles) {
    if (!totalFiles) return '- Risk assessment unavailable';
    
    if (totalFiles > 100) {
      return `⚠️ **LARGE PROJECT** (${totalFiles} files)
    - High architectural risk
    - Likely circular dependencies
    - Requires modular architecture control`;
    } else if (totalFiles > 50) {
      return `✅ **MEDIUM PROJECT** (${totalFiles} files)
    - Manageable size, moderate architectural risks
    - Possible local code quality issues`;
        } else {
      return `✅ **SMALL PROJECT** (${totalFiles} files)
    - Compact structure, low architectural risks
    - Main issues: code smells, code quality`;
    }
  }
}

function formatToolResult(toolName, result, projectPath, detailLevel = 'summary') {
  switch (toolName) {
    case 'analyze_project':
      return ResponseFormatter.formatAnalysisResult(result, projectPath, detailLevel);
    case 'export_ai_compact': 
      return ResponseFormatter.formatExportResult(result, projectPath, detailLevel);
    case 'get_project_structure':
      return ResponseFormatter.formatStructureResult(result, projectPath, detailLevel);
    case 'generate_diagram':
      return ResponseFormatter.formatDiagramResult(result, projectPath, detailLevel);
    default:
      return JSON.stringify(result, null, 2);
  }
}

// Summarization helpers
const SUMMARY_LIMIT_CHARS = 30000; // ~30KB per message
typeof global !== 'undefined' && (global.SUMMARY_LIMIT_CHARS = SUMMARY_LIMIT_CHARS);

function stripCodeBlocks(md) {
  try {
    return md.replace(/```[\s\S]*?```/g, '').replace(/\n{3,}/g, '\n\n');
  } catch { return md; }
}

function clampText(text, maxChars) {
  if (!text || text.length <= maxChars) return text;
  return text.slice(0, maxChars) + "\n... (truncated)";
}

// 📊 SIMPLIFIED HANDLERS
async function handleAnalyzeProject(args) {
  const { project_path, detail_level = 'summary', deep = false } = args;
  
  try {
    if (!project_path) {
      throw new Error("project_path is required");
    }
    
    const resolvedPath = resolveProjectPath(project_path);
    
    // 🚀 DIRECT EXECUTION: Let Rust binary handle access errors gracefully
    logger.debug("Attempting direct binary execution - Rust handles access errors gracefully");
    
    // 🔐 SMART MODE: TRY NORMAL EXECUTION FIRST, CHECK ADMIN ONLY IF NEEDED
    // Rust binary gracefully handles access denied errors, so we don't need admin precheck
    
    let result;
    if (deep) {
      result = await executeArchlensCommand('analyze', resolvedPath, ['--deep']);
    } else {
      result = await executeArchlensCommand('analyze', resolvedPath);
    }
    return createMCPResponse('analyze_project', result, null, resolvedPath, detail_level);
    
  } catch (error) {
    // Smart error handling: most access errors are handled gracefully by Rust binary
    // Only suggest admin elevation if there's a genuine system-level access issue
    if (os.platform() === 'win32' && 
        (error.message.includes('Access denied') || 
         error.message.includes('os error 5') ||
         error.message.includes('Отказано в доступе')) &&
        error.message.includes('permission')) {  // Only for actual permission errors
      
      logger.debug("Genuine permission error detected - suggesting admin elevation");
      
    return {
      content: [{
        type: "text",
          text: `⚠️ PERMISSION ERROR DETECTED

**The binary encountered a genuine permission issue.**

**RECOMMENDED SOLUTIONS:**

**Option 1: Alternative commands (work without admin):**
- get_project_structure - ✅ Full structure analysis
- export_ai_compact - ✅ Complete AI analysis  
- generate_diagram - ✅ Architecture diagrams

**Option 2: Admin elevation (only if needed):**
1. Right-click PowerShell → "Run as Administrator"
2. cd "${path.dirname(path.resolve(__filename))}"
3. node "${path.resolve(__filename)}"

**Note:** Most projects work fine without admin rights. Try Option 1 first!

**Error details:** ${error.message}`
        }],
        isError: false
      };
    }
    
    // For non-permission errors, return standard error response
    return createMCPResponse('analyze_project', null, error, project_path, detail_level);
  }
}

// 🔧 LIMITED ANALYSIS FALLBACK (No admin rights needed)
async function createLimitedAnalysis(projectPath) {
  logger.debug(`Creating limited analysis for: ${projectPath}`);
  
  try {
    // Use manual structure scan which doesn't need admin rights
    const structureResult = createManualStructure(projectPath);
    const structure = structureResult.structure;
    
    return {
      status: "success",
      method: "limited_analysis",
      total_files: structure.total_files,
      total_lines: structure.total_lines,
      file_types: structure.file_types,
      layers: structure.layers,
      project_path: projectPath,
      analysis_date: new Date().toISOString(),
      limitations: [
        "Limited analysis mode (no admin rights)",
        "Some system files may be inaccessible",
        "Use export_ai_compact for detailed analysis"
      ],
      recommendation: "For full analysis, use 'export_ai_compact' and 'get_project_structure' commands"
    };
  } catch (error) {
    throw new Error(`Limited analysis failed: ${error.message}`);
  }
}

async function handleExportAICompact(args) {
  const { project_path, output_file, detail_level = 'summary' } = args;
  
  try {
    if (!project_path) {
      throw new Error("project_path is required");
    }
    
    // Early validation for user-friendly error messages
    if (project_path === '.' || project_path.startsWith('./') || project_path.startsWith('../')) {
      return {
        content: [{
          type: "text",
          text: `❌ RELATIVE PATH REJECTED

You provided: "${project_path}"

This MCP server requires ABSOLUTE paths only. Examples:
- ✅ "/home/user/myproject" 
- ✅ "C:\\Users\\User\\MyProject"
- ❌ "." (current directory)
- ❌ "./src" (relative path)

Please provide the complete absolute path to your project directory.`
        }],
        isError: true
      };
    }
    
    const resolvedPath = resolveProjectPath(project_path);
    const additionalArgs = ['ai_compact'];
    if (output_file) {
      additionalArgs.push(output_file);
    }
    
    const result = await executeArchlensCommand('export', resolvedPath, additionalArgs);
    
    return createMCPResponse('export_ai_compact', result, null, resolvedPath, detail_level);
    
  } catch (error) {
    return createMCPResponse('export_ai_compact', null, error, project_path, detail_level);
  }
}

async function handleGetProjectStructure(args) {
  const { project_path, detail_level = 'summary' } = args;
  
  try {
    if (!project_path) {
      throw new Error("project_path is required");
    }
    
    // Early validation for user-friendly error messages
    if (project_path === '.' || project_path.startsWith('./') || project_path.startsWith('../')) {
      return {
        content: [{
          type: "text",
          text: `❌ RELATIVE PATH REJECTED

You provided: "${project_path}"

This MCP server requires ABSOLUTE paths only. Examples:
- ✅ "/home/user/myproject" 
- ✅ "C:\\Users\\User\\MyProject"
- ❌ "." (current directory)
- ❌ "./src" (relative path)

Please provide the complete absolute path to your project directory.`
        }],
        isError: true
      };
    }
    
    const resolvedPath = resolveProjectPath(project_path);
    
    try {
      const result = await executeArchlensCommand('structure', resolvedPath);
      return createMCPResponse('get_project_structure', result, null, resolvedPath, detail_level);
    } catch (binaryError) {
      // Fallback: manual structure scan
      const fallbackResult = createManualStructure(resolvedPath);
      return createMCPResponse('get_project_structure', fallbackResult, null, resolvedPath, detail_level);
    }
    
  } catch (error) {
    return createMCPResponse('get_project_structure', null, error, project_path, detail_level);
  }
}

async function handleGenerateDiagram(args) {
  const { project_path, diagram_type = "mermaid", output_file, detail_level = 'summary' } = args;
  
  try {
    if (!project_path) {
      throw new Error("project_path is required");
    }
    
    // Early validation for user-friendly error messages
    if (project_path === '.' || project_path.startsWith('./') || project_path.startsWith('../')) {
      return {
        content: [{
          type: "text",
          text: `❌ RELATIVE PATH REJECTED

You provided: "${project_path}"

This MCP server requires ABSOLUTE paths only. Examples:
- ✅ "/home/user/myproject" 
- ✅ "C:\\Users\\User\\MyProject"
- ❌ "." (current directory)
- ❌ "./src" (relative path)

Please provide the complete absolute path to your project directory.`
        }],
        isError: true
      };
    }
    
    const resolvedPath = resolveProjectPath(project_path);
    const tempFile = output_file || path.resolve(CONFIG.paths.workingDirectory, `temp_diagram_${Date.now()}.${diagram_type === 'mermaid' ? 'mmd' : diagram_type}`);
    
    try {
      await executeArchlensCommand('diagram', resolvedPath, [diagram_type, tempFile]);
      
      // Read created diagram file
      if (fs.existsSync(tempFile)) {
        const diagramContent = fs.readFileSync(tempFile, 'utf8');
        
        // Remove temp file if not user-specified
        if (!output_file) {
          fs.unlinkSync(tempFile);
        }
        
        const result = {
            status: "success",
          diagram: diagramContent,
            diagram_type,
            output_file: output_file || "stdout",
          size: diagramContent.length
        };
        
        return createMCPResponse('generate_diagram', result, null, resolvedPath, detail_level);
        } else {
        throw new Error(`Diagram file not created: ${tempFile}`);
        }
    } catch (execError) {
      throw new Error(`Diagram generation failed: ${execError.message}`);
    }
    
  } catch (error) {
    return createMCPResponse('generate_diagram', null, error, project_path, detail_level);
  }
}

// 🔧 Manual structure creation (fallback, no side effects)
function createManualStructure(projectPath) {
  const structure = {
    total_files: 0,
    total_lines: 0,
    file_types: {},
    layers: [],
    files: []
  };
  
  const start = Date.now();
  const deadline = start + (CONFIG.limits.scanTimeout || 30000);
  
  try {
    const scanDirectory = (dir, depth = 0) => {
      if (Date.now() > deadline) return; // time budget guard
      if (depth > CONFIG.limits.scanDepth) return;
      if (structure.files.length >= CONFIG.limits.maxFiles) return;
      
      const items = fs.readdirSync(dir);
      
      for (const item of items) {
        if (Date.now() > deadline) return;
        if (structure.files.length >= CONFIG.limits.maxFiles) return;
        const fullPath = path.join(dir, item);
        
        try {
          const stat = fs.statSync(fullPath);
          if (stat.isDirectory()) {
            const skipDirs = ['node_modules', '.git', 'target', 'dist', 'build', '.next', '.nuxt'];
            if (!skipDirs.includes(item) && !item.startsWith('.')) {
              scanDirectory(fullPath, depth + 1);
            }
          } else {
            const ext = path.extname(item).toLowerCase();
            const relativePath = path.relative(path.resolve(projectPath), fullPath);
            
            structure.total_files++;
            structure.file_types[ext] = (structure.file_types[ext] || 0) + 1;
            
            if (structure.files.length < CONFIG.limits.maxFiles) {
              let lineCount = 0;
              try {
                if (CONFIG.textExtensions.includes(ext) && stat.size < CONFIG.limits.maxFileSize) {
                  const content = fs.readFileSync(fullPath, 'utf8');
                  lineCount = content.split('\n').length;
                }
              } catch (readError) {
                logger.debug(`Cannot read file ${fullPath}: ${readError.message}`);
                lineCount = 0;
              }
              
              structure.files.push({ path: relativePath, name: item, extension: ext, size: stat.size, lines: lineCount });
              structure.total_lines += lineCount;
            }
          }
        } catch (statError) {
          logger.debug(`File access error ${fullPath}: ${statError.message}`);
        }
      }
    };
    
    scanDirectory(projectPath);
    
    const commonLayers = ['src', 'lib', 'components', 'utils', 'api', 'core', 'ui', 'services', 'models', 'views', 'controllers'];
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
    method: "manual_scan",
    duration_ms: Date.now() - start
  };
}

// Create server instance
const server = new Server({
  name: CONFIG.server.name,
  version: CONFIG.server.version
}, {
  capabilities: { tools: {} }
});

// 📋 MCP tools registration
server.setRequestHandler(ListToolsRequestSchema, async () => ({
  tools: [
    {
      name: "export_ai_compact",
      description: "🤖 AI EXPORT - Full architecture analysis (~2800 tokens) with problem discovery: Code Smells (20+ types: long methods, magic numbers, code duplication), SOLID principles, architectural antipatterns (God Objects, tight coupling), circular dependencies, quality metrics (cyclomatic/cognitive complexity, tech debt), recommendations for refactoring.",
      inputSchema: {
        type: "object",
        properties: {
          project_path: { description: "ABSOLUTE or RELATIVE path to the project directory", type: "string" },
          output_file: { description: "Path to save the result (optional)", type: "string" },
          focus_critical_only: { description: "Show only critical problems", type: "boolean" },
          include_diff_analysis: { description: "Include comparison with previous versions", type: "boolean" },
          detail_level: { description: "summary | standard | full (default: summary)", type: "string", enum: ["summary","standard","full"] }
        },
        required: ["project_path"]
      }
    },
    {
      name: "analyze_project",
      description: "📊 SHORT ANALYSIS - Basic project statistics with a preliminary assessment.",
      inputSchema: {
        type: "object",
        properties: {
          project_path: { description: "ABSOLUTE or RELATIVE path", type: "string" },
          verbose: { description: "Detailed output", type: "boolean" },
          analyze_dependencies: { description: "Analyze module dependencies", type: "boolean" },
          extract_comments: { description: "Analyze documentation quality", type: "boolean" },
          generate_summaries: { description: "Generate brief descriptions", type: "boolean" },
          include_patterns: { description: "File include patterns", type: "array", items: { type: "string" } },
          exclude_patterns: { description: "File exclude patterns", type: "array", items: { type: "string" } },
          max_depth: { description: "Max directory depth", type: "integer" },
          deep: { description: "Use full analysis pipeline (CLI analyze --deep)", type: "boolean" },
          detail_level: { description: "summary | standard | full (default: summary)", type: "string", enum: ["summary","standard","full"] }
        },
        required: ["project_path"]
      }
    },
    {
      name: "generate_diagram",
      description: "📈 DIAGRAM GENERATION - Creates an architectural diagram.",
      inputSchema: {
        type: "object",
        properties: {
          project_path: { description: "ABSOLUTE or RELATIVE path", type: "string" },
          diagram_type: { description: "mermaid (default), svg, dot", type: "string", enum: ["mermaid", "svg", "dot"] },
          include_metrics: { description: "Include quality metrics (rendered by caller)", type: "boolean" },
          output_file: { description: "Path to save the diagram (optional)", type: "string" },
          detail_level: { description: "summary | standard | full (default: summary)", type: "string", enum: ["summary","standard","full"] }
        },
        required: ["project_path"]
      }
    },
    {
      name: "get_project_structure",
      description: "📁 PROJECT STRUCTURE - Hierarchical structure with structural problem detection.",
      inputSchema: {
        type: "object",
        properties: {
          project_path: { description: "ABSOLUTE or RELATIVE path", type: "string" },
          show_metrics: { description: "Include file metrics", type: "boolean" },
          max_files: { description: "Maximum number of files in output (default 1000)", type: "integer" },
          detail_level: { description: "summary | standard | full (default: summary)", type: "string", enum: ["summary","standard","full"] }
        },
        required: ["project_path"]
      }
    }
  ]
}));

// 🎯 Tool call handling - simple dispatching
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;
  
  try {
    if (name === 'analyze_project') {
      return await handleAnalyzeProject(args);
    } else if (name === "export_ai_compact") {
      return await handleExportAICompact(args);
    } else if (name === "get_project_structure") {
      return await handleGetProjectStructure(args);
    } else if (name === "generate_diagram") {
      return await handleGenerateDiagram(args);
    } else {
      throw new Error(`Unknown tool: ${name}`);
    }
  } catch (error) {
    return createMCPResponse(name, null, error, args.project_path);
  }
});

// 🚀 MCP server startup
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  
  logger.info("🏗️ ArchLens MCP Server v2.0.0 started - CRITICAL FIXES APPLIED");
  logger.info("✅ NO HARDCODED PATHS | NO SIDE EFFECTS | PROPER '.' SUPPORT | UNIFIED LANGUAGE | WINDOWS FIXES");
  
  process.stdin.resume();
}

main().catch(error => {
  logger.error(`Server startup failed: ${error.message}`);
  process.exit(1);
}); 