# 🤝 Contributing to ArchLens

<div align="center">

![Contributing](https://img.shields.io/badge/contributions-welcome-brightgreen?style=for-the-badge&logo=github)
[![Code of Conduct](https://img.shields.io/badge/code%20of-conduct-blue?style=for-the-badge)](CODE_OF_CONDUCT.md)
[![Discord](https://img.shields.io/badge/Discord-Join%20Us-7289da?style=for-the-badge&logo=discord)](https://discord.gg/archlens)

**Thank you for your interest in contributing to ArchLens!**

*Every contribution, no matter how small, helps make ArchLens better for everyone.*

</div>

---

## 🌟 Ways to Contribute

### 🐛 **Bug Reports**
Found a bug? Help us fix it!

### ✨ **Feature Requests** 
Have an idea? We'd love to hear it!

### 📝 **Documentation**
Improve docs, examples, or tutorials

### 🧪 **Testing**
Test new features, write tests, improve coverage

### 🔧 **Code Contributions**
Fix bugs, implement features, optimize performance

### 🌍 **Translations**
Help make ArchLens accessible worldwide

---

## 🚀 Quick Start for Contributors

### 1️⃣ **Setup Development Environment**

```bash
# Fork and clone the repository
git clone https://github.com/yourusername/archlens.git
cd archlens

# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Node.js 18+ (for MCP server)
# Visit https://nodejs.org/ or use your package manager

# Build the project
cargo build

# Install MCP dependencies
cd mcp && npm install && cd ..

# Run tests
cargo test

# Run the development version
cargo run -- analyze .
```

### 2️⃣ **Verify Everything Works**

```bash
# Test CLI functionality
cargo run --release -- analyze .
cargo run --release -- structure .
cargo run --release -- export . ai_compact

# Test MCP server
cd mcp
node archlens_mcp_server.cjs &
# Should show: "🏗️ ArchLens MCP Server v2.0.0 started"
```

### 3️⃣ **Make Your Changes**

```bash
# Create a feature branch
git checkout -b feature/amazing-feature

# Make your changes
# ... edit code ...

# Test your changes
cargo test
cargo build --release

# Commit your changes
git add .
git commit -m "Add amazing feature"

# Push to your fork
git push origin feature/amazing-feature
```

### 4️⃣ **Submit a Pull Request**

1. Go to your fork on GitHub
2. Click "New Pull Request"
3. Fill out the PR template
4. Wait for review and feedback

---

## 📋 Development Guidelines

### 🦀 **Rust Code Style**

#### **Formatting**
```bash
# Use rustfmt for consistent formatting
cargo fmt

# Use clippy for linting
cargo clippy -- -D warnings
```

#### **Code Conventions**
```rust
// ✅ Good: Clear, descriptive names
fn analyze_project_structure(path: &Path) -> Result<ProjectStructure> {
    // Implementation
}

// ❌ Bad: Unclear abbreviations
fn analyze_proj_struct(p: &Path) -> Result<ProjStruct> {
    // Implementation
}

// ✅ Good: Comprehensive error handling
match fs::read_to_string(path) {
    Ok(content) => process_content(content),
    Err(e) => {
        eprintln!("⚠️ Warning: Cannot read file {:?}: {}", path, e);
        return Ok(Vec::new()); // Graceful degradation
    }
}

// ✅ Good: Detailed documentation
/// Analyzes the architectural structure of a project
/// 
/// # Arguments
/// * `project_path` - Path to the project root directory
/// * `config` - Analysis configuration options
/// 
/// # Returns
/// * `Ok(AnalysisResult)` - Successful analysis with metrics
/// * `Err(AnalysisError)` - Analysis failed with detailed error
/// 
/// # Examples
/// ```
/// let result = analyze_project("./src", &default_config())?;
/// println!("Found {} files", result.total_files);
/// ```
pub fn analyze_project(project_path: &Path, config: &AnalysisConfig) -> Result<AnalysisResult> {
    // Implementation
}
```

#### **Error Handling**
```rust
// ✅ Preferred: Use thiserror for custom errors
#[derive(thiserror::Error, Debug)]
pub enum AnalysisError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parsing error in file {file}: {message}")]
    Parse { file: String, message: String },
    
    #[error("Invalid configuration: {0}")]
    Config(String),
}

// ✅ Graceful error handling
fn scan_directory(path: &Path) -> Result<Vec<FileMetadata>> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("⚠️ Warning: Cannot access directory {:?}: {}", path, e);
            return Ok(Vec::new()); // Continue with empty result
        }
    };
    // Process entries...
}
```

### 🔌 **MCP Server Guidelines**

#### **JavaScript/Node.js Style**
```javascript
// ✅ Good: Async/await with proper error handling
async function executeArchlensCommand(command, args) {
    try {
        const result = await spawn(binary, [command, ...args]);
        return { success: true, data: result };
    } catch (error) {
        logger.error(`Command failed: ${error.message}`);
        return { success: false, error: error.message };
    }
}

// ✅ Good: Comprehensive logging
function resolveProjectPath(inputPath) {
    logger.debug(`Resolving path: "${inputPath}"`);
    
    const resolved = path.resolve(inputPath);
    logger.debug(`Resolved to: "${resolved}"`);
    
    if (!fs.existsSync(resolved)) {
        throw new Error(`Path does not exist: ${resolved}`);
    }
    
    return resolved;
}

// ✅ Good: Input validation
function validateMCPRequest(params) {
    if (!params.project_path) {
        throw new Error('project_path is required');
    }
    
    if (typeof params.project_path !== 'string') {
        throw new Error('project_path must be a string');
    }
    
    // Additional validation...
}
```

### 📝 **Documentation Standards**

#### **Code Documentation**
```rust
/// # ArchLens File Scanner
/// 
/// Provides functionality to recursively scan project directories
/// and extract metadata from source files.
/// 
/// ## Features
/// - Multi-language support (Rust, TypeScript, JavaScript, Python, etc.)
/// - Configurable include/exclude patterns
/// - Graceful error handling for inaccessible files
/// - Parallel processing for large projects
/// 
/// ## Example
/// ```rust
/// use archlens::FileScanner;
/// 
/// let scanner = FileScanner::new(
///     vec!["**/*.rs".to_string()],
///     vec!["**/target/**".to_string()],
///     Some(10)
/// )?;
/// 
/// let files = scanner.scan_project(Path::new("./src"))?;
/// println!("Found {} Rust files", files.len());
/// ```
pub struct FileScanner {
    // Fields...
}
```

#### **README and Guides**
- Use clear, descriptive headings
- Include practical examples
- Add troubleshooting sections
- Use emojis for visual appeal (but not excessively)
- Provide both English and Russian versions

### 🧪 **Testing Guidelines**

#### **Unit Tests**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_file_scanner_basic_functionality() {
        // Arrange
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "fn main() {}").unwrap();
        
        let scanner = FileScanner::new(
            vec!["**/*.rs".to_string()],
            vec![],
            None
        ).unwrap();
        
        // Act
        let files = scanner.scan_project(temp_dir.path()).unwrap();
        
        // Assert
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].file_type, FileType::Rust);
        assert_eq!(files[0].lines_count, 1);
    }
    
    #[test]
    fn test_graceful_error_handling() {
        let scanner = FileScanner::new(vec![], vec![], None).unwrap();
        
        // Should not panic on non-existent directory
        let result = scanner.scan_project(Path::new("/nonexistent"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }
}
```

#### **Integration Tests**
```rust
// tests/integration_test.rs
use std::process::Command;

#[test]
fn test_cli_analyze_command() {
    let output = Command::new("./target/release/archlens")
        .args(&["analyze", "."])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("total_files"));
    assert!(stdout.contains("total_lines"));
}
```

#### **MCP Server Tests**
```javascript
// mcp/test/integration.test.js
const { ArchLensMCPClient } = require('../examples/mcp_integration');

describe('MCP Server Integration', () => {
    let client;
    
    beforeEach(() => {
        client = new ArchLensMCPClient('./archlens_mcp_server.cjs');
    });
    
    test('analyze_project should return valid structure', async () => {
        const result = await client.callTool('analyze_project', {
            project_path: '.'
        });
        
        expect(result.content).toBeDefined();
        expect(result.content[0].text).toContain('PROJECT ANALYSIS');
    });
});
```

---

## 🔍 Code Review Process

### **For Contributors**

1. **Self-Review Checklist**
   - [ ] Code follows style guidelines
   - [ ] All tests pass (`cargo test`)
   - [ ] No clippy warnings (`cargo clippy`)
   - [ ] Documentation is updated
   - [ ] Examples work correctly
   - [ ] Commit messages are clear

2. **Pull Request Template**
```markdown
## 📋 Description
Brief description of changes and motivation.

## 🔧 Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## 🧪 Testing
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] New and existing unit tests pass locally with my changes
- [ ] I have tested the MCP server integration

## 📝 Checklist
- [ ] My code follows the style guidelines of this project
- [ ] I have performed a self-review of my own code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] I have made corresponding changes to the documentation
- [ ] My changes generate no new warnings
```

### **For Reviewers**

1. **Review Focus Areas**
   - Code quality and maintainability
   - Performance implications
   - Security considerations
   - Documentation completeness
   - Test coverage

2. **Review Guidelines**
   - Be constructive and specific
   - Ask questions if unclear
   - Suggest improvements
   - Approve when ready

---

## 🐛 Bug Reports

### **Before Reporting**

1. **Search existing issues** - Maybe it's already reported
2. **Try latest version** - Bug might be already fixed
3. **Minimal reproduction** - Create smallest possible example

### **Bug Report Template**

```markdown
## 🐛 Bug Report

**Describe the bug**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

**Expected behavior**
A clear and concise description of what you expected to happen.

**Environment:**
- OS: [e.g. Windows 11, macOS 13, Ubuntu 22.04]
- Rust version: [e.g. 1.70.0]
- Node.js version: [e.g. 18.17.0]
- ArchLens version: [e.g. 1.0.0]

**Additional context**
Add any other context about the problem here.

**Logs**
```
Paste relevant logs here
```
```

---

## ✨ Feature Requests

### **Feature Request Template**

```markdown
## 🚀 Feature Request

**Is your feature request related to a problem? Please describe.**
A clear and concise description of what the problem is.

**Describe the solution you'd like**
A clear and concise description of what you want to happen.

**Describe alternatives you've considered**
A clear and concise description of any alternative solutions or features you've considered.

**Use cases**
Describe specific use cases where this feature would be helpful.

**Additional context**
Add any other context or screenshots about the feature request here.
```

---

## 🏗️ Architecture Guidelines

### **Project Structure**
```
src/
├── analysis/           # Core analysis logic
├── architecture/       # Architecture-specific analysis
├── cli/               # Command-line interface
├── enrichment/        # Code quality analysis
├── export/            # Output generation
└── types.rs           # Shared types and structures

mcp/
├── archlens_mcp_server.cjs  # MCP server implementation
├── package.json             # Node.js dependencies
└── README.md               # MCP-specific documentation

examples/
├── basic_analysis.rs       # Rust CLI example
├── mcp_integration.js      # MCP integration example
└── README.md              # Examples documentation
```

### **Design Principles**

1. **Modularity** - Each module has a single responsibility
2. **Error Handling** - Graceful degradation, never panic in production
3. **Performance** - Optimize for large codebases
4. **Extensibility** - Easy to add new languages and analysis types
5. **AI-First** - Output optimized for AI consumption

### **Adding New Features**

#### **New Language Support**
1. Add language detection in `file_scanner.rs`
2. Implement parser in `parser_ast.rs`
3. Add language-specific patterns in `enrichment/`
4. Update tests and documentation

#### **New Analysis Type**
1. Create module in `enrichment/`
2. Integrate with `capsule_enricher.rs`
3. Add CLI command in `cli/`
4. Update MCP server if needed

#### **New Export Format**
1. Add format in `exporter.rs`
2. Implement formatter
3. Add CLI option
4. Update documentation

---

## 📚 Resources

### **Learning Resources**
- 🦀 [The Rust Book](https://doc.rust-lang.org/book/)
- 🔌 [MCP Protocol Specification](https://modelcontextprotocol.io/)
- 🏗️ [Clean Architecture Principles](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- 📊 [Code Quality Metrics](https://en.wikipedia.org/wiki/Software_metric)

### **Development Tools**
```bash
# Useful cargo commands
cargo fmt              # Format code
cargo clippy           # Lint code
cargo test             # Run tests
cargo doc --open       # Generate and open documentation
cargo bench            # Run benchmarks

# Git hooks (optional)
cargo install cargo-husky
```

### **IDE Setup**
- **VS Code**: Install Rust Analyzer, Even Better TOML
- **IntelliJ**: Install Rust plugin
- **Vim/Neovim**: Install rust.vim, coc-rust-analyzer

---

## 🎯 Contribution Opportunities

### **Good First Issues**
- 📝 Improve documentation
- 🧪 Add test cases
- 🌍 Add language translations
- 🔍 Fix small bugs
- ✨ Add code examples

### **Medium Complexity**
- 🔧 Add new CLI options
- 📊 Implement new metrics
- 🏗️ Improve architecture analysis
- 🔌 Enhance MCP server features

### **Advanced Contributions**
- 🚀 Add new language support
- 🧠 Implement ML-based analysis
- ⚡ Performance optimizations
- 🌐 Web interface development

---

## 🏆 Recognition

### **Contributors Hall of Fame**

We recognize and appreciate all our contributors:

- 🥇 **Top Contributors** - Listed in README
- 🎖️ **Special Recognition** - For significant contributions
- 📈 **Progress Tracking** - Contribution statistics
- 🎁 **Swag** - For major contributors (when available)

### **How to Get Recognized**

1. **Consistent Contributions** - Regular, quality contributions
2. **Community Engagement** - Help others, answer questions
3. **Documentation** - Improve docs and examples
4. **Testing** - Find and report bugs, write tests
5. **Innovation** - Propose and implement new features

---

## 📞 Getting Help

### **Communication Channels**

- 💬 **GitHub Discussions** - General questions and ideas
- 🐛 **GitHub Issues** - Bug reports and feature requests
- 📧 **Email** - contact@archlens.dev
- 🌐 **Discord** - Real-time chat (coming soon)

### **Response Times**

- 🐛 **Critical bugs** - Within 24 hours
- ✨ **Feature requests** - Within 1 week
- 📝 **Documentation** - Within 3 days
- 💬 **General questions** - Within 2 days

---

## 📄 Code of Conduct

We are committed to providing a welcoming and inclusive environment for all contributors. Please read our [Code of Conduct](CODE_OF_CONDUCT.md) before participating.

### **Our Pledge**

- 🤝 **Be respectful** - Treat everyone with respect
- 🌍 **Be inclusive** - Welcome people of all backgrounds
- 🎯 **Be constructive** - Focus on helping and improving
- 📚 **Be patient** - Help newcomers learn and grow

---

<div align="center">

**🙏 Thank you for contributing to ArchLens!**

[![Contributors](https://img.shields.io/github/contributors/yourusername/archlens?style=for-the-badge)](https://github.com/yourusername/archlens/graphs/contributors)

*Together, we're building better software architecture analysis tools.*

</div> 