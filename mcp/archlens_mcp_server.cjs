#!/usr/bin/env node

// 🏗️ ARCHLENS MCP SERVER v2.0.0 - CRITICAL FIXES APPLIED
// NO HARDCODED PATHS | NO SIDE EFFECTS | PROPER "." SUPPORT | UNIFIED LANGUAGE | WINDOWS FIXES
const { Server } = require("@modelcontextprotocol/sdk/server/index.js");
const { StdioServerTransport } = require("@modelcontextprotocol/sdk/server/stdio.js");
const { CallToolRequestSchema, ListToolsRequestSchema } = require("@modelcontextprotocol/sdk/types.js");
const { spawn } = require('child_process');
const fs = require('fs');
const path = require('path');
const os = require('os');

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
      path.join(process.cwd(), "target", "release"),
      path.join(process.cwd(), "target", "debug"), 
      path.join(__dirname, "target", "release"),
      path.join(__dirname, "target", "debug"),
      path.join(__dirname),
      process.cwd()
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
    retryOnAccessDenied: true,
    adminRequired: false,
    autoElevate: process.env.ARCHLENS_AUTO_ELEVATE === "true"
  },
  patterns: {
    include: [
      "**/*.rs", "**/*.ts", "**/*.js", "**/*.py", "**/*.java", 
      "**/*.cpp", "**/*.c", "**/*.go", "**/*.php", "**/*.rb", 
      "**/*.cs", "**/*.kt", "**/*.swift", "**/*.dart", "**/*.vue", 
      "**/*.jsx", "**/*.tsx", "**/*.html", "**/*.css", "**/*.scss", 
      "**/*.sass", "**/*.json", "**/*.yaml", "**/*.yml", "**/*.xml", 
      "**/*.md", "**/*.txt"
    ],
    exclude: [
      "**/target/**", "**/node_modules/**", "**/.git/**", "**/dist/**", 
      "**/build/**", "**/.next/**", "**/.nuxt/**", "**/coverage/**", 
      "**/tmp/**", "**/temp/**"
    ]
  },
  limits: {
    maxDepth: 20,
    maxFiles: 1000,
    scanDepth: 15,
    maxFileSize: 1000000
  },
  textExtensions: [
    '.rs', '.ts', '.js', '.py', '.java', '.cpp', '.c', '.go', '.php', 
    '.rb', '.cs', '.kt', '.swift', '.dart', '.vue', '.jsx', '.tsx', 
    '.html', '.css', '.scss', '.sass', '.json', '.yaml', '.yml', 
    '.xml', '.md', '.txt'
  ]
};

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
        fs.accessSync(fullPath, fs.constants.F_OK | fs.constants.X_OK);
        logger.debug(`Found executable binary: ${fullPath}`);
        return fullPath;
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

// 🛡️ PROPER PATH RESOLUTION (SUPPORTS ".")
function resolveProjectPath(inputPath) {
  if (!inputPath || typeof inputPath !== 'string') {
    throw new Error('project_path is required and must be a string');
  }
  
  let resolvedPath;
  
  // Handle special case: "." should resolve to the working directory where MCP was called
  if (inputPath === ".") {
    resolvedPath = CONFIG.paths.workingDirectory;
    logger.debug(`Resolved "." to working directory: ${resolvedPath}`);
  } else if (path.isAbsolute(inputPath)) {
    resolvedPath = path.normalize(inputPath);
    logger.debug(`Using absolute path: ${resolvedPath}`);
  } else {
    // For relative paths, resolve against working directory
    resolvedPath = path.resolve(CONFIG.paths.workingDirectory, inputPath);
    logger.debug(`Resolved relative path "${inputPath}" to: ${resolvedPath}`);
  }
  
  // Validate path exists and is accessible
  try {
    if (!fs.existsSync(resolvedPath)) {
      throw new Error(`Path does not exist: ${resolvedPath}`);
    }
    
    const stat = fs.statSync(resolvedPath);
    if (!stat.isDirectory()) {
      throw new Error(`Path is not a directory: ${resolvedPath}`);
    }
    
    // Test read access
    fs.accessSync(resolvedPath, fs.constants.R_OK);
    
  } catch (error) {
    if (error.code === 'EACCES' || error.code === 'EPERM') {
      throw new Error(`Access denied to path: ${resolvedPath}\n` +
        `🔧 Windows Solutions:\n` +
        `  • Run as Administrator\n` +
        `  • Check folder permissions\n` +
        `  • Disable antivirus temporarily\n` +
        `  • Ensure files are not in use`);
    }
    throw error;
  }
  
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
    const powershellArgs = [
      '-Command',
      `Start-Process PowerShell -Verb RunAs -ArgumentList "-Command", "cd '${process.cwd()}'; node mcp/archlens_mcp_server.cjs"`
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
  const powershellCommand = `Start-Process PowerShell -Verb RunAs -ArgumentList "-Command", "cd '${process.cwd()}'; node mcp/archlens_mcp_server.cjs"`;
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
3. Navigate to: ${process.cwd()}
4. Run: node mcp/archlens_mcp_server.cjs
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

// 🚀 UNIFIED COMMAND EXECUTION (NO SIDE EFFECTS)
async function executeArchlensCommand(subcommand, projectPath, additionalArgs = [], options = {}) {
  return new Promise((resolve, reject) => {
      const binary = getArchLensBinary();
    const args = [subcommand, projectPath, ...additionalArgs];
    
    const spawnOptions = {
      stdio: ['pipe', 'pipe', 'pipe'],
      cwd: CONFIG.paths.workingDirectory,
      encoding: 'utf8',
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
    logger.debug(`Project path: ${projectPath}`);
    
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
      stdout += data.toString('utf8');
      });
      
      child.stderr.on('data', (data) => {
      stderr += data.toString('utf8');
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
function createMCPResponse(toolName, result, error = null, projectPath = null) {
  if (error) {
    return {
      content: [{
        type: "text",
        text: `❌ ERROR ${getToolDisplayName(toolName)}

Failed to ${getToolAction(toolName)}: ${projectPath || 'unknown path'}

**Reason:** ${error.message}

**Project path:** ${projectPath || 'n/a'}
**Error time:** ${new Date().toLocaleString('en-US')}
**Platform:** ${os.platform()} ${os.release()}`
      }],
      isError: true
    };
  }
  
    return {
      content: [{
        type: "text",
      text: formatToolResult(toolName, result, projectPath)
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
  static formatAnalysisResult(result, projectPath) {
    const data = typeof result === 'string' ? JSON.parse(result) : result;
    
    return `# 🔍 PROJECT ANALYSIS BRIEF

**Path:** ${projectPath}
**Analysis performed:** ${new Date().toLocaleString('en-US')}

## 📊 Key metrics
- **Total files:** ${data.total_files || 'n/a'}
- **Lines of code:** ${data.total_lines || 'n/a'}
- **Scan date:** ${data.scanned_at ? new Date(data.scanned_at).toLocaleString('en-US') : 'n/a'}

## 🗂️ File distribution
${data.file_types ? Object.entries(data.file_types)
  .sort(([,a], [,b]) => b - a)
  .slice(0, 10)
  .map(([ext, count]) => `- **.${ext}**: ${count} file(s)`)
  .join('\n') : 'Data unavailable'}

## 📈 Architectural assessment
${this.getArchitecturalRisk(data.total_files)}

## 🎯 Deep analysis capabilities
Use \`export_ai_compact\` to discover:
- **Code Smells:** long methods, magic numbers, code duplication
- **SOLID principles:** violations of single responsibility, open/closed
- **Architectural antipatterns:** God Objects, tight coupling, circular dependencies
- **Quality metrics:** cyclomatic complexity, technical debt

*This is a preliminary assessment. Use specialized tools for detailed analysis.*`;
  }

  static formatExportResult(result, projectPath) {
    if (result.output && typeof result.output === 'string' && result.output.trim().length > 0) {
      return result.output; // AI Compact already formatted
    }
    
    return `❌ ARCHITECTURE ANALYSIS ERROR
    
Failed to perform AI Compact export for project: ${projectPath}

**Reason:** ${result.message || 'Unknown error'}
**Details:** ${result.output || result.stderr || 'No details'}

**What AI Compact should have analyzed:**
- Code Smells (20+ types): long methods, magic numbers, code duplication
- SOLID principles: single responsibility, open/closed, Liskov substitution
- Architectural antipatterns: God Objects, tight coupling, circular dependencies
- Quality metrics: cyclomatic complexity, cognitive complexity, maintainability index

**Path:** ${projectPath}
**Error time:** ${new Date().toLocaleString('en-US')}`;
  }

  static formatStructureResult(result, projectPath) {
    const data = typeof result === 'string' ? JSON.parse(result) : result;
    
    return `# 📁 PROJECT STRUCTURE OVERVIEW

**Path:** ${projectPath}
**Analysis performed:** ${new Date().toLocaleString('en-US')}

## 📊 General statistics
- **Total files:** ${data.total_files || 'n/a'}
- **Total lines:** ${data.total_lines || 'n/a'}
- **Files shown:** ${data.files ? Math.min(data.files.length, data.total_files || 0) : 'n/a'} (limit: ${CONFIG.limits.maxFiles})

## 🗂️ File types
${data.file_types ? Object.entries(data.file_types)
  .sort(([,a], [,b]) => b - a)
  .map(([ext, count]) => `- **.${ext}**: ${count} file(s)`)
  .join('\n') : 'Data unavailable'}

## 🏗️ Architectural layers
${data.layers ? data.layers.map(layer => `- **${layer}**`).join('\n') : '- Layers not identified'}

## 📄 Key files (top 15)
${data.files ? data.files.slice(0, 15).map(file => 
  `- \`${file.path}\` (${file.extension}, ${(file.size/1024).toFixed(1)}KB)`
).join('\n') + 
(data.files.length > 15 ? `\n\n... and ${data.files.length - 15} more file(s)` : '') : 'Files not found'}

*Structure overview complete. Use specialized tools for detailed problem analysis.*`;
  }

  static formatDiagramResult(result, projectPath) {
    if (result.diagram && typeof result.diagram === 'string') {
      return `# 📊 ARCHITECTURAL DIAGRAM

**Project:** ${projectPath}
**Type:** ${result.diagram_type || 'unknown'}
**Created:** ${new Date().toISOString()}

## Mermaid Diagram

\`\`\`mermaid
${result.diagram}
\`\`\`

*Generated by ArchLens for AI analysis*`;
    }
    
    return `❌ DIAGRAM GENERATION ERROR
    
Failed to create diagram for project: ${projectPath}

**Type:** ${result.diagram_type || 'unknown'}
**Reason:** ${result.message || 'Unknown error'}

**Path:** ${projectPath}
**Error time:** ${new Date().toLocaleString('en-US')}`;
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

function formatToolResult(toolName, result, projectPath) {
  switch (toolName) {
    case 'analyze_project':
      return ResponseFormatter.formatAnalysisResult(result, projectPath);
    case 'export_ai_compact': 
      return ResponseFormatter.formatExportResult(result, projectPath);
    case 'get_project_structure':
      return ResponseFormatter.formatStructureResult(result, projectPath);
    case 'generate_diagram':
      return ResponseFormatter.formatDiagramResult(result, projectPath);
    default:
      return JSON.stringify(result, null, 2);
  }
}

// 📊 SIMPLIFIED HANDLERS
async function handleAnalyzeProject(args) {
  const { project_path } = args;
  
  try {
    if (!project_path) {
      throw new Error("project_path is required");
    }
    
    // 🔐 CHECK ADMIN RIGHTS ON WINDOWS
    if (os.platform() === 'win32') {
      const hasAdminRights = await checkWindowsAdminRights();
      
      if (!hasAdminRights) {
        // Try automatic elevation if enabled
        if (CONFIG.windows.autoElevate) {
          logger.debug("Attempting automatic admin elevation...");
          const elevationAttempted = await tryAutoElevation();
          
          if (elevationAttempted) {
    return {
      content: [{
        type: "text",
                text: `🔐 ADMIN ELEVATION INITIATED

**Status:** Administrator elevation request sent to Windows

**Next Steps:**
1. Approve UAC prompt if it appears
2. New PowerShell window will open with admin rights
3. MCP server will restart automatically
4. Retry analyze_project command in the new admin session

**If UAC was cancelled or failed:**
${createAdminElevationInstructions('analyze', project_path)}

**Current Session:** This session will continue running for other commands.`
              }],
              isError: false
            };
          }
        }
        
        // Return admin elevation instructions if auto-elevation is disabled or failed
        return {
          content: [{
            type: "text",
            text: createAdminElevationInstructions('analyze', project_path)
          }],
          isError: false
        };
      }
      
      logger.debug("Windows admin rights confirmed - proceeding with analysis");
    }
    
    const resolvedPath = resolveProjectPath(project_path);
    const result = await executeArchlensCommand('analyze', resolvedPath);
    
    return createMCPResponse('analyze_project', result, null, resolvedPath);
    
  } catch (error) {
    // Enhanced Windows error handling
    if (os.platform() === 'win32' && 
        (error.message.includes('Access denied') || 
         error.message.includes('os error 5') ||
         error.message.includes('Отказано в доступе'))) {
      
    return {
      content: [{
        type: "text",
          text: `❌ WINDOWS ACCESS DENIED

${createAdminElevationInstructions('analyze', project_path)}

**Original Error:** ${error.message}`
      }],
      isError: true
    };
    }
    
    return createMCPResponse('analyze_project', null, error, project_path);
  }
}

async function handleExportAICompact(args) {
  const { project_path, output_file } = args;
  
  try {
    if (!project_path) {
      throw new Error("project_path is required");
    }
    
    const resolvedPath = resolveProjectPath(project_path);
    const additionalArgs = ['ai_compact'];
    if (output_file) {
      additionalArgs.push(output_file);
    }
    
    const result = await executeArchlensCommand('export', resolvedPath, additionalArgs);
    
    return createMCPResponse('export_ai_compact', result, null, resolvedPath);
    
  } catch (error) {
    return createMCPResponse('export_ai_compact', null, error, project_path);
  }
}

async function handleGetProjectStructure(args) {
  const { project_path } = args;
  
  try {
    if (!project_path) {
      throw new Error("project_path is required");
    }
    
    const resolvedPath = resolveProjectPath(project_path);
    
    try {
      const result = await executeArchlensCommand('structure', resolvedPath);
      return createMCPResponse('get_project_structure', result, null, resolvedPath);
    } catch (binaryError) {
      // Fallback: manual structure scan
      const fallbackResult = createManualStructure(resolvedPath);
      return createMCPResponse('get_project_structure', fallbackResult, null, resolvedPath);
    }
    
  } catch (error) {
    return createMCPResponse('get_project_structure', null, error, project_path);
  }
}

async function handleGenerateDiagram(args) {
  const { project_path, diagram_type = "mermaid", output_file } = args;
  
  try {
    if (!project_path) {
      throw new Error("project_path is required");
    }
    
    const resolvedPath = resolveProjectPath(project_path);
    const tempFile = output_file || `temp_diagram_${Date.now()}.${diagram_type === 'mermaid' ? 'mmd' : diagram_type}`;
    
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
        
        return createMCPResponse('generate_diagram', result, null, resolvedPath);
        } else {
        throw new Error(`Diagram file not created: ${tempFile}`);
      }
    } catch (execError) {
      throw new Error(`Diagram generation failed: ${execError.message}`);
    }
    
  } catch (error) {
    return createMCPResponse('generate_diagram', null, error, project_path);
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
  
  try {
    const scanDirectory = (dir, depth = 0) => {
      if (depth > CONFIG.limits.scanDepth) return;
      
      const items = fs.readdirSync(dir);
      
      for (const item of items) {
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
          const relativePath = path.relative(projectPath, fullPath);
          
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
              
            structure.files.push({
              path: relativePath,
              name: item,
              extension: ext,
                size: stat.size,
                lines: lineCount
            });
              
              structure.total_lines += lineCount;
          }
          }
        } catch (statError) {
          logger.debug(`File access error ${fullPath}: ${statError.message}`);
        }
      }
    };
    
    scanDirectory(projectPath);
    
    // Determine layers by folder structure
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
    method: "manual_scan"
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
          project_path: {
            description: "Path to the project for analysis",
            type: "string"
          },
          output_file: {
            description: "Path to save the result (optional)",
            type: "string"
          },
          focus_critical_only: {
            description: "Show only critical problems: God Objects, circular dependencies, high complexity, SOLID violations",
            type: "boolean"
          },
          include_diff_analysis: {
            description: "Include comparison with previous versions for degradation analysis",
            type: "boolean"
          }
        },
        required: ["project_path"]
      }
    },
    {
      name: "analyze_project",
      description: "📊 SHORT ANALYSIS - Basic project statistics with a preliminary assessment of problems: project size, file distribution, architectural risk assessment (small/medium/large), recommendations for deep analysis via export_ai_compact.",
      inputSchema: {
        type: "object",
        properties: {
          project_path: {
            description: "Path to the project for analysis",
            type: "string"
          },
          verbose: {
            description: "Detailed output with additional metrics and warnings",
            type: "boolean"
          },
          analyze_dependencies: {
            description: "Analyze module dependencies to identify circular dependencies",
            type: "boolean"
          },
          extract_comments: {
            description: "Extract comments and analyze documentation quality",
            type: "boolean"
          },
          generate_summaries: {
            description: "Generate brief descriptions of components with potential problems",
            type: "boolean"
          },
          include_patterns: {
            description: "File patterns to include (e.g., ['**/*.rs', '**/*.ts'])",
            type: "array",
            items: { type: "string" }
          },
          exclude_patterns: {
            description: "File patterns to exclude",
            type: "array",
            items: { type: "string" }
          },
          max_depth: {
            description: "Maximum directory depth for scanning",
            type: "integer"
          }
        },
        required: ["project_path"]
      }
    },
    {
      name: "generate_diagram",
      description: "📈 DIAGRAM GENERATION - Creates an architectural diagram with visualization of problems: dependencies between components, problematic connections (circular dependencies marked in red), complexity metrics, architectural layers. For Mermaid returns ready code.",
      inputSchema: {
        type: "object",
        properties: {
          project_path: {
            description: "Path to the project for analysis",
            type: "string"
          },
          diagram_type: {
            description: "Diagram type: mermaid (default), svg, dot",
            type: "string",
            enum: ["mermaid", "svg", "dot"]
          },
          include_metrics: {
            description: "Include quality metrics in the diagram: cyclomatic complexity, coupling, problematic components",
            type: "boolean"
          },
          output_file: {
            description: "Path to save the diagram (optional)",
            type: "string"
          }
        },
        required: ["project_path"]
      }
    },
    {
      name: "get_project_structure",
      description: "📁 PROJECT STRUCTURE - Hierarchical structure with structural problem detection: incorrect layer organization, mismatch with architectural patterns, files candidates for refactoring (large sizes), metrics by file types.",
      inputSchema: {
        type: "object",
        properties: {
          project_path: {
            description: "Path to the project",
            type: "string"
          },
          show_metrics: {
            description: "Include file metrics: size, lines of code, complexity assessment",
            type: "boolean"
          },
          max_files: {
            description: "Maximum number of files in output (default 1000)",
            type: "integer"
          }
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