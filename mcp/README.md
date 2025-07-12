# 🔌 ArchLens MCP Server

<div align="center">

![MCP Server](https://img.shields.io/badge/MCP-Server-purple?style=for-the-badge&logo=anthropic&logoColor=white)
[![Node.js](https://img.shields.io/badge/node.js-6DA55F?style=for-the-badge&logo=node.js&logoColor=white)](https://nodejs.org/)
[![Claude Compatible](https://img.shields.io/badge/Claude-Compatible-blue?style=for-the-badge&logo=anthropic)](https://claude.ai/)
[![Cursor Compatible](https://img.shields.io/badge/Cursor-Compatible-orange?style=for-the-badge&logo=cursor)](https://cursor.sh/)

**🤖 Model Context Protocol server for seamless AI integration with ArchLens**

*Connect your favorite AI assistant directly to powerful architecture analysis tools*

[🚀 Quick Setup](#-quick-setup) • [🛠️ Configuration](#️-configuration) • [📖 API Reference](#-api-reference) • [🔧 Troubleshooting](#-troubleshooting)

</div>

---

## 🌟 Overview

The ArchLens MCP Server provides a **Model Context Protocol** interface that allows AI assistants like Claude, Cursor, and others to directly access ArchLens architecture analysis capabilities. No more copy-pasting analysis results - your AI can now understand your codebase architecture in real-time!

### ✨ **Key Features**

| Feature | Description | Benefit |
|---------|-------------|---------|
| 🔍 **Real-time Analysis** | Direct project scanning and analysis | Up-to-date insights without manual exports |
| 🤖 **AI-Optimized Output** | Structured data perfect for AI consumption | Better AI understanding and recommendations |
| 🚫 **No Admin Rights** | Works without elevated permissions | Seamless integration in any environment |
| 🔧 **Auto-Discovery** | Automatic binary detection and path resolution | Zero configuration in most cases |
| 📊 **Rich Diagnostics** | Comprehensive error handling and debugging | Easy troubleshooting when issues arise |

---

## 🚀 Quick Setup

### 📋 Prerequisites

- **Node.js 18+** (for MCP server)
- **ArchLens binary** (compiled Rust tool)
- **AI Assistant** supporting MCP (Claude Desktop, Cursor, etc.)

### ⚡ Installation Steps

#### 1️⃣ **Build ArchLens Binary**
```bash
# In the project root
cargo build --release
# Binary will be at ./target/release/archlens.exe (Windows) or ./target/release/archlens (Unix)
```

#### 2️⃣ **Install MCP Dependencies**
```bash
cd mcp
npm install
```

#### 3️⃣ **Test MCP Server**
```bash
# Test if server starts correctly
node archlens_mcp_server.cjs
# Should show: "🏗️ ArchLens MCP Server v2.0.0 started"
```

#### 4️⃣ **Configure Your AI Assistant**

##### For Claude Desktop:
Create or edit `~/.config/claude-desktop/mcp_settings.json`:
```json
{
  "mcpServers": {
    "archlens": {
      "command": "node",
      "args": ["/absolute/path/to/archlens/mcp/archlens_mcp_server.cjs"],
      "env": {
        "ARCHLENS_DEBUG": "false"
      }
    }
  }
}
```

##### For Cursor:
Add to `.cursor/mcp_settings.json` in your workspace:
```json
{
  "mcpServers": {
    "archlens": {
      "command": "node",
              "args": ["/absolute/path/to/your/project/mcp/archlens_mcp_server.cjs"],
      "env": {
        "ARCHLENS_DEBUG": "false"
      }
    }
  }
}
```

#### 5️⃣ **Restart Your AI Assistant**

#### 6️⃣ **Test Integration**
Ask your AI assistant: *"Can you analyze the current project structure using ArchLens?"*

---

## 🛠️ Configuration

### 🔧 Environment Variables

| Variable | Default | Description | Example |
|----------|---------|-------------|---------|
| `ARCHLENS_DEBUG` | `false` | Enable debug logging | `true` |
| `ARCHLENS_BINARY` | `archlens` | Custom binary name | `archlens-custom` |
| `ARCHLENS_PATH` | auto-detect | Custom binary path | `/usr/local/bin/archlens` |
| `ARCHLENS_WORKDIR` | `process.cwd()` | Working directory | `/projects/myapp` |
| `ARCHLENS_AUTO_ELEVATE` | `false` | Auto admin elevation (Windows) | `true` |

### 📁 Advanced Configuration

Create `.archlens-mcp.json` in your project root:

```json
{
  "server": {
    "timeout": 60000,
    "maxFiles": 1000,
    "maxDepth": 20
  },
  "analysis": {
    "includePatterns": ["**/*.rs", "**/*.ts", "**/*.js"],
    "excludePatterns": ["**/target/**", "**/node_modules/**"],
    "languages": ["rust", "typescript", "javascript", "python"]
  },
  "output": {
    "forceEnglish": true,
    "includeDiagrams": true,
    "verboseErrors": false
  }
}
```

---

## 📖 API Reference

### 🔍 **analyze_project**

Quick project overview with basic statistics and risk assessment.

**Parameters:**
- `project_path` (string, required): Path to analyze (use absolute path or relative path)
- `verbose` (boolean, optional): Include detailed warnings
- `analyze_dependencies` (boolean, optional): Analyze module dependencies
- `extract_comments` (boolean, optional): Extract documentation quality metrics

**Example Usage:**
```javascript
// AI Assistant can call:
analyze_project({
  "project_path": "/path/to/your/project",
  "verbose": true,
  "analyze_dependencies": true
})
```

**Sample Output:**
```markdown
# 🔍 PROJECT ANALYSIS BRIEF

**Path:** /your/project
**Analysis performed:** 2024-01-15, 14:30:22

## 📊 Key metrics
- **Total files:** 127
- **Lines of code:** 15,432
- **Technical debt:** 23.5%

## 🗂️ File distribution
- **.rs**: 45 file(s)
- **.ts**: 32 file(s)
- **.js**: 28 file(s)

## 📈 Architectural assessment
⚠️ **MEDIUM PROJECT** (127 files)
- Manageable size, moderate architectural risks
- Possible circular dependencies detected
```

---

### 🤖 **export_ai_compact**

Comprehensive architecture analysis optimized for AI consumption (~2800 tokens).

**Parameters:**
- `project_path` (string, required): Path to analyze
- `output_file` (string, optional): Save to file
- `focus_critical_only` (boolean, optional): Show only critical problems
- `include_diff_analysis` (boolean, optional): Include degradation analysis

**Example Usage:**
```javascript
export_ai_compact({
  "project_path": "/path/to/your/project",
  "focus_critical_only": false,
  "include_diff_analysis": true
})
```

**What It Analyzes:**
- **🔍 Code Smells**: 20+ types including long methods, magic numbers, code duplication
- **🏗️ SOLID Principles**: Single responsibility, open/closed, Liskov substitution violations
- **⚠️ Architectural Antipatterns**: God Objects, tight coupling, circular dependencies
- **📊 Quality Metrics**: Cyclomatic complexity, cognitive complexity, maintainability index
- **💳 Technical Debt**: Quantified debt assessment with refactoring recommendations

---

### 📁 **get_project_structure**

Hierarchical project structure with structural problem detection.

**Parameters:**
- `project_path` (string, required): Path to analyze
- `show_metrics` (boolean, optional): Include file metrics
- `max_files` (integer, optional): Limit files in output (default: 1000)

**Example Usage:**
```javascript
get_project_structure({
  "project_path": "/path/to/your/project",
  "show_metrics": true,
  "max_files": 500
})
```

**Output Includes:**
- **📊 Statistics**: Total files, lines of code, file type distribution
- **🏗️ Architectural Layers**: Detected layers (domain, infrastructure, application, etc.)
- **📄 File Listing**: Key files with sizes and metrics
- **⚠️ Structure Issues**: Incorrect layer organization, candidates for refactoring

---

### 📊 **generate_diagram**

Creates architectural diagrams with problem visualization.

**Parameters:**
- `project_path` (string, required): Path to analyze
- `diagram_type` (string, optional): "mermaid" (default), "svg", "dot"
- `include_metrics` (boolean, optional): Include quality metrics
- `output_file` (string, optional): Save diagram to file

**Example Usage:**
```javascript
generate_diagram({
  "project_path": "/path/to/your/project",
  "diagram_type": "mermaid",
  "include_metrics": true
})
```

**Diagram Features:**
- **🔗 Dependencies**: Component relationships and data flow
- **🔴 Problem Highlighting**: Circular dependencies marked in red
- **📊 Complexity Metrics**: Visual complexity indicators
- **🏗️ Layer Visualization**: Architectural layer separation

---

## 🎯 Use Cases & Examples

### 🔍 **Code Review Automation**

**AI Prompt:**
> "Analyze this project and identify the top 3 architectural issues that should be addressed before the next release."

**MCP Calls:**
1. `analyze_project` - Get overview
2. `export_ai_compact` - Deep analysis
3. `generate_diagram` - Visual representation

---

### 🤖 **AI-Assisted Refactoring**

**AI Prompt:**
> "I want to refactor this codebase to follow clean architecture principles. What's the current state and what should I change?"

**MCP Calls:**
1. `get_project_structure` - Current organization
2. `export_ai_compact` - SOLID violations and antipatterns
3. `generate_diagram` - Current vs. desired architecture

---

### 📊 **Technical Debt Assessment**

**AI Prompt:**
> "Calculate the technical debt in this project and prioritize which components need immediate attention."

**MCP Calls:**
1. `export_ai_compact` - Comprehensive debt analysis
2. `analyze_project` - Risk assessment
3. `get_project_structure` - Component-level breakdown

---

## 🔧 Troubleshooting

### ❌ **Common Issues**

#### **1. "Binary not found" Error**

**Symptoms:**
```
❌ ArchLens binary 'archlens' not found in search paths
```

**Solutions:**
```bash
# Option 1: Build the binary
cargo build --release

# Option 2: Set custom path
export ARCHLENS_PATH=/path/to/your/archlens

# Option 3: Add to PATH
export PATH=$PATH:/path/to/archlens/target/release
```

#### **2. "Permission denied" Error**

**Symptoms:**
```
⚠️ PERMISSION ERROR DETECTED
```

**Solutions:**
- ✅ **Use alternative commands**: `get_project_structure`, `export_ai_compact`, `generate_diagram` work without admin
- ⚙️ **Run as admin** (only if absolutely needed): Right-click PowerShell → "Run as Administrator"
- 🔧 **Check antivirus**: Ensure binary isn't blocked by antivirus software

#### **3. MCP Server Not Starting**

**Symptoms:**
- AI assistant shows "MCP server failed to start"
- No response from ArchLens tools

**Debug Steps:**
```bash
# Test server manually
cd mcp
node archlens_mcp_server.cjs

# Enable debug mode
export ARCHLENS_DEBUG=true
node archlens_mcp_server.cjs
```

**Common Fixes:**
- ✅ **Check Node.js version**: Requires Node.js 18+
- ✅ **Verify paths**: Use absolute paths in MCP configuration
- ✅ **Install dependencies**: Run `npm install` in mcp/ directory

#### **4. "Command timed out" Error**

**Symptoms:**
```
Command execution timed out after 60000ms
```

**Solutions:**
```bash
# Increase timeout
export ARCHLENS_TIMEOUT=120000

# Reduce scope
export ARCHLENS_MAX_DEPTH=5
export ARCHLENS_MAX_FILES=500
```

### 🐛 **Debug Mode**

Enable comprehensive logging:

```bash
export ARCHLENS_DEBUG=true
node mcp/archlens_mcp_server.cjs
```

**Debug Output Includes:**
- 🔍 Binary discovery process
- 📁 Path resolution details
- ⚙️ Command execution logs
- 🔧 Configuration validation
- ❌ Detailed error messages

### 📞 **Getting Help**

If you're still having issues:

1. **📋 Check the logs** with debug mode enabled
2. **🔍 Search existing issues** in the GitHub repository
3. **🐛 Create a new issue** with:
   - Operating system and version
   - Node.js version (`node --version`)
   - ArchLens version
   - Full error message
   - Debug logs (if applicable)

---

## 🔄 Version History

### **v2.0.0** - Current
- ✅ **Fixed admin rights requirement** - No longer needs elevated permissions
- ✅ **Absolute path resolution** - Works from any directory
- ✅ **Enhanced error handling** - Better diagnostics and recovery
- ✅ **Unified language output** - Consistent English output
- ✅ **Windows compatibility** - Native Windows support

### **v1.0.0** - Legacy
- ⚠️ Required admin rights (fixed in v2.0.0)
- ⚠️ Relative path issues (fixed in v2.0.0)
- ✅ Basic MCP functionality
- ✅ Core analysis tools

---

## 📚 Additional Resources

### 🔗 **Links**
- 🏠 [Main Project](../README.md)
- 📖 [MCP Protocol Documentation](https://modelcontextprotocol.io/)
- 🤖 [Claude Desktop Setup](https://claude.ai/desktop)
- 🔧 [Cursor Editor](https://cursor.sh/)

### 📖 **Related Documentation**
- [ArchLens CLI Reference](../README.md#-documentation)
- [Architecture Analysis Guide](../README.md#-architecture)
- [Contributing Guidelines](../CONTRIBUTING.md)

---

<div align="center">

**🔌 Seamless AI Integration for Better Architecture**

[![MCP Protocol](https://img.shields.io/badge/MCP-Protocol-purple?style=flat&logo=anthropic)](https://modelcontextprotocol.io/)
[![AI Powered](https://img.shields.io/badge/AI-Powered-blue?style=flat&logo=openai)](https://openai.com/)
[![Real-time](https://img.shields.io/badge/Real--time-Analysis-green?style=flat&logo=speedtest)](https://github.com)

*Connect your AI assistant to powerful architecture analysis today!*

</div> 