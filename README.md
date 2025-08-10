# 🏗️ ArchLens - Advanced Architecture Analysis Tool

<div align="center">

![ArchLens Logo](https://img.shields.io/badge/ArchLens-Architecture%20Analysis-blue?style=for-the-badge&logo=rust&logoColor=white)

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen?style=for-the-badge&logo=github-actions)](https://github.com)
[![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-purple?style=for-the-badge&logo=anthropic)](https://modelcontextprotocol.io/)

**🔍 Intelligent code architecture analysis with AI-powered insights**

*Discover code smells, architectural antipatterns, and technical debt in your projects*

[📖 Documentation](#-documentation) • [🚀 Quick Start](#-quick-start) • [🤖 AI Integration](#-ai-integration)

</div>

---

## 🌟 Features

### 🔍 **Deep Code Analysis**
- **Code Smells Detection**: Long methods, magic numbers, code duplication
- **SOLID Principles**: Violations of single responsibility, open/closed principles
- **Architectural Antipatterns**: God Objects, tight coupling, circular dependencies
- **Quality Metrics**: Cyclomatic complexity, cognitive complexity, maintainability index

### 🏗️ **Architecture Insights**
- **Project Structure**: Hierarchical analysis with layer detection
- **Dependency Mapping**: Import/export relationships and circular dependencies
- **Technical Debt**: Quantified debt assessment with refactoring recommendations
- **Risk Assessment**: Automated architectural risk evaluation

### 🤖 **AI-Ready Output**
- **MCP Server (Rust, STDIO)**: Official STDIO JSON-RPC with JSON Schema publishing (no HTTP)
- **Structured Reports**: JSON/Markdown exports optimized for AI consumption
- **Interactive Diagrams**: Mermaid diagrams for visual architecture representation
- **Context-Rich**: Detailed explanations suitable for AI-assisted refactoring

### 🛠️ **Developer Experience**
- **Multi-Language**: Rust, TypeScript, JavaScript, Python, Java, Go, C/C++
- **Cross-Platform**: Windows, macOS, Linux support
- **CLI & Library**: Command-line interface and Rust library API
- **No Admin Rights**: Works without elevated permissions

---

## 🚀 Quick Start

### 📦 Installation

#### Build from Source
```bash
# Clone repository
git clone https://github.com/yourusername/archlens.git
cd archlens

# Build release binaries
cargo build --release

# Binaries will be available at ./target/release/archlens and ./target/release/archlens-mcp
```

### 🔍 Basic Usage (CLI)

#### 📊 Project Analysis
```bash
# Analyze current directory
./target/release/archlens analyze .

# Analyze specific project
./target/release/archlens analyze /path/to/project
```

#### 📁 Project Structure
```bash
./target/release/archlens structure . --show-metrics
```

#### 🤖 AI-Ready Export
```bash
# Export comprehensive analysis for AI (summary detail level by default)
./target/release/archlens export . ai_compact --output analysis.md
```

---

## 🤖 AI Integration

### 🔌 MCP Server (Model Context Protocol, Rust, STDIO)

The ArchLens MCP server provides a clean STDIO JSON‑RPC interface (no HTTP). It exposes:
- Methods: `tools/list`, `tools/call`, `resources/list`, `resources/read`, `prompts/list`, `prompts/get`
- Tool names returned via `tools/list` use underscore format (e.g., `export_ai_summary_json`), while calls accept both underscore and dotted aliases (e.g., `export.ai_summary_json`).

#### 🛠️ Run with MCP Inspector
```bash
# Build the server
cargo build --release --bin archlens-mcp

# Start MCP Inspector against the server (stdio)
npx @modelcontextprotocol/inspector -- ./target/release/archlens-mcp
# Open the printed Inspector URL to interactively test tools/resources/prompts
```

#### 🧪 STDIO Examples
```json
{"jsonrpc":"2.0","id":1,"method":"tools/list"}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"export_ai_summary_json","arguments":{"project_path":".","top_n":5}}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ai_recommend","arguments":{"project_path":".","json":{}}}}
```

---

## 🛠️ Development

### 🔧 Building
```bash
# Development build
cargo build

# Release build with optimizations
cargo build --release

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run --bin archlens -- analyze .
```

### 🧪 Testing
```bash
# Run all tests
cargo test

# With output
cargo test -- --nocapture
```

---

## 📄 License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
