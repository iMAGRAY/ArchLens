use std::path::Path;
use super::analysis::AnalysisExporter;
use super::metrics::MetricsCalculator;
use super::patterns::PatternDetector;
use super::utils::ExportUtils;

/// AI Compact exporter - generates compact AI-readable analysis
pub struct AICompactExporter;

impl AICompactExporter {
    /// Generates AI compact analysis report
    pub fn generate_ai_compact(project_path: &str) -> std::result::Result<String, String> {
        if !Path::new(project_path).exists() {
            return Err("Path does not exist".to_string());
        }
        
        let mut output = String::new();
        
        // Header
        output.push_str("# 🏗️ AI COMPACT ARCHITECTURE ANALYSIS\n\n");
        output.push_str(&format!("**Project:** {}\n", project_path));
        output.push_str(&format!("**Analysis date:** {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        output.push_str(&format!("**Analysis ID:** {}\n\n", uuid::Uuid::new_v4()));
        
        // Quick statistics
        let stats = AnalysisExporter::collect_basic_stats(project_path)?;
        output.push_str("## 📊 QUICK STATISTICS\n");
        output.push_str(&format!("- **Total files:** {}\n", stats.total_files));
        output.push_str(&format!("- **Lines of code:** {}\n", stats.total_lines));
        output.push_str(&format!("- **File types:** {}\n", stats.file_types.len()));
        output.push_str(&format!("- **Components:** {}\n", stats.components));
        output.push_str(&format!("- **Connections:** {}\n", stats.connections));
        output.push_str("\n");
        
        // Critical issues
        let issues = AnalysisExporter::analyze_critical_issues(project_path)?;
        if !issues.is_empty() {
            output.push_str("## 🚨 CRITICAL ISSUES\n");
            for issue in issues {
                output.push_str(&format!("- **{}:** {}\n", issue.severity, issue.description));
            }
            output.push_str("\n");
        }
        
        // Architectural patterns
        let patterns = PatternDetector::detect_architectural_patterns(project_path)?;
        if !patterns.is_empty() {
            output.push_str("## 🏛️ ARCHITECTURAL PATTERNS\n");
            for pattern in patterns {
                output.push_str(&format!("- **{}:** {} (confidence: {}%)\n", 
                                       pattern.name, pattern.description, pattern.confidence));
            }
            output.push_str("\n");
        }
        
        // Project structure
        let structure = AnalysisExporter::analyze_project_structure(project_path)?;
        output.push_str("## 📁 PROJECT STRUCTURE\n");
        output.push_str(&format!("```\n{}\n```\n\n", structure));
        
        // Key modules
        let modules = AnalysisExporter::analyze_key_modules(project_path)?;
        if !modules.is_empty() {
            output.push_str("## 🔧 KEY MODULES\n");
            for module in modules {
                output.push_str(&format!("- **{}** ({}): {}\n", 
                                       module.name, module.category, module.description));
            }
            output.push_str("\n");
        }
        
        // Recommendations
        let recommendations = AnalysisExporter::generate_recommendations(project_path)?;
        if !recommendations.is_empty() {
            output.push_str("## 💡 RECOMMENDATIONS\n");
            for rec in recommendations {
                output.push_str(&format!("- **{}:** {}\n", rec.priority, rec.description));
            }
            output.push_str("\n");
        }
        
        // Quality metrics
        let quality = MetricsCalculator::calculate_quality_metrics(project_path)?;
        output.push_str("## 📈 QUALITY METRICS\n");
        output.push_str(&format!("- **Maintainability index:** {}/100\n", quality.maintainability));
        output.push_str(&format!("- **Cyclomatic complexity:** {}\n", quality.complexity));
        output.push_str(&format!("- **Documentation coverage:** {}%\n", quality.documentation_coverage));
        output.push_str(&format!("- **Tech debt:** {}\n", quality.tech_debt));
        output.push_str("\n");
        
        output.push_str("---\n");
        output.push_str("*Generated by ArchLens AI Compact Export*\n");
        
        Ok(output)
    }
} 