# 📚 ArchLens Examples

This directory contains practical examples demonstrating how to use ArchLens in various scenarios.

## 🗂️ Available Examples

### 🦀 **basic_analysis.rs**
**Rust command-line example**

Demonstrates how to use ArchLens CLI programmatically from Rust code.

**Features:**
- ✅ Basic project analysis
- ✅ Structure analysis with metrics
- ✅ AI-ready export generation
- ✅ Architecture diagram creation
- ✅ Error handling and validation

**Usage:**
```bash
# Run the example
cargo script examples/basic_analysis.rs

# Or compile and run
rustc examples/basic_analysis.rs -o basic_analysis
./basic_analysis
```

---

### 🔌 **mcp_integration.js**
**Node.js MCP integration example**

Shows how to integrate ArchLens MCP server with your own applications or AI tools.

**Features:**
- ✅ MCP client implementation
- ✅ All 4 MCP tools demonstration
- ✅ Error handling patterns
- ✅ Common usage patterns
- ✅ Performance optimization tips

**Usage:**
```bash
# Install dependencies
cd mcp && npm install

# Run the example
node examples/mcp_integration.js
```

**Integration Patterns:**
- 🔍 **Code Review Automation** - Automated quality assessment
- 🤖 **AI Assistant Integration** - Context-rich analysis for AI
- 📊 **Technical Debt Monitoring** - Continuous debt tracking

---

## 🚀 Quick Start

### 1️⃣ **Prerequisites**
```bash
# Build ArchLens binary
cargo build --release

# Install MCP dependencies (for Node.js examples)
cd mcp && npm install
```

### 2️⃣ **Run Examples**
```bash
# Rust example
cargo script examples/basic_analysis.rs

# Node.js example  
node examples/mcp_integration.js
```

### 3️⃣ **Expected Output**
All examples will analyze the current ArchLens project and demonstrate:
- 📊 Project statistics and metrics
- 🏗️ Architecture structure analysis
- 🤖 AI-ready comprehensive reports
- 📈 Visual architecture diagrams

---

## 🎯 Use Case Examples

### 🔍 **Code Review Automation**
```rust
// In CI/CD pipeline
let analysis = run_archlens_analysis(&project_path)?;
if analysis.technical_debt_ratio > 25.0 {
    return Err("Technical debt too high for merge");
}
```

### 🤖 **AI Assistant Context**
```javascript
// Provide rich context to AI
const context = await getArchitectureContext('./src');
const aiResponse = await ai.analyze(`
    Based on this architecture analysis:
    ${context.analysis}
    
    Suggest refactoring improvements.
`);
```

### 📊 **Technical Debt Dashboard**
```javascript
// Monitor debt over time
const projects = ['./frontend', './backend', './shared'];
const debtReport = await Promise.all(
    projects.map(path => analyzeDebt(path))
);
```

---

## 🛠️ Custom Integration

### Creating Your Own Integration

1. **Choose Your Approach:**
   - 🦀 **CLI Integration**: Call ArchLens binary directly
   - 🔌 **MCP Integration**: Use MCP protocol for AI tools
   - 📚 **Library Integration**: Use ArchLens as a Rust library

2. **Basic Pattern:**
```rust
// CLI approach
use std::process::Command;

let output = Command::new("archlens")
    .args(&["analyze", project_path])
    .output()?;

let analysis = String::from_utf8(output.stdout)?;
```

```javascript
// MCP approach
const client = new ArchLensMCPClient(serverPath);
const analysis = await client.callTool('analyze_project', {
    project_path: '.'
});
```

3. **Error Handling:**
```rust
match run_analysis() {
    Ok(result) => process_result(result),
    Err(e) => handle_error(e),
}
```

---

## 📋 Example Scenarios

### **Scenario 1: Pre-commit Hook**
```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "🔍 Running architecture analysis..."
if ! cargo run --release -- analyze . --quiet; then
    echo "❌ Architecture issues detected. Commit blocked."
    exit 1
fi
echo "✅ Architecture validation passed."
```

### **Scenario 2: GitHub Actions**
```yaml
name: Architecture Analysis
on: [pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
      - name: Build ArchLens
        run: cargo build --release
      - name: Run Analysis
        run: ./target/release/archlens export . ai_compact
```

### **Scenario 3: VS Code Extension**
```typescript
// Extension integration
import { ArchLensMCPClient } from './archlens-client';

export async function analyzeWorkspace() {
    const client = new ArchLensMCPClient(serverPath);
    const analysis = await client.callTool('export_ai_compact', {
        project_path: vscode.workspace.rootPath
    });
    
    // Display results in VS Code
    showAnalysisPanel(analysis.content[0].text);
}
```

---

## 🔗 Related Resources

- 📖 [Main Documentation](../README.md)
- 🔌 [MCP Server Guide](../mcp/README.md)
- 🤖 [AI Integration Guide](../README.md#-ai-integration)
- 🛠️ [Development Guide](../CONTRIBUTING.md)

---

<div align="center">

**💡 Have an interesting use case?**

[Share it with the community!](https://github.com/yourusername/archlens/discussions)

</div> 

### Deep analysis (CLI)
```bash
./archlens analyze . --deep > deep_analysis.json
```

### MCP detail level
```json
{
  "project_path": ".",
  "detail_level": "summary", // or "standard" | "full"
  "deep": true
}
``` 