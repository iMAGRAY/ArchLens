use crate::types::Capsule;
use std::collections::HashMap;

/// Capsule analyzer - provides analysis capabilities for capsules
pub struct CapsuleAnalyzer;

impl CapsuleAnalyzer {
    /// Analyzes capsule quality
    pub fn analyze_quality(capsule: &Capsule) -> f64 {
        let mut score: f64 = 1.0;
        
        // Complexity penalty
        if capsule.complexity > 10 {
            score *= 0.5;
        } else if capsule.complexity > 5 {
            score *= 0.8;
        }
        
        // Size penalty
        let size = capsule.line_end - capsule.line_start + 1;
        if size > 100 {
            score *= 0.6;
        } else if size > 50 {
            score *= 0.8;
        }
        
        // Warning penalty
        let warning_count = capsule.warnings.len();
        if warning_count > 5 {
            score *= 0.4;
        } else if warning_count > 2 {
            score *= 0.7;
        }
        
        // Documentation bonus
        if capsule.description.is_some() && capsule.slogan.is_some() {
            score *= 1.1;
        }
        
        score.max(0.0).min(1.0)
    }
    
    /// Analyzes capsule dependencies
    pub fn analyze_dependencies(capsules: &[Capsule]) -> HashMap<String, Vec<String>> {
        let mut dependencies = HashMap::new();
        
        for capsule in capsules {
            let deps: Vec<String> = capsule.dependencies
                .iter()
                .map(|dep| dep.to_string())
                .collect();
            dependencies.insert(capsule.name.clone(), deps);
        }
        
        dependencies
    }
    
    /// Finds capsules with high complexity
    pub fn find_complex_capsules(capsules: &[Capsule], threshold: u32) -> Vec<&Capsule> {
        capsules.iter()
            .filter(|capsule| capsule.complexity > threshold)
            .collect()
    }
    
    /// Finds capsules with many warnings
    pub fn find_problematic_capsules(capsules: &[Capsule], warning_threshold: usize) -> Vec<&Capsule> {
        capsules.iter()
            .filter(|capsule| capsule.warnings.len() > warning_threshold)
            .collect()
    }
    
    /// Calculates overall project metrics
    pub fn calculate_project_metrics(capsules: &[Capsule]) -> ProjectMetrics {
        let total_capsules = capsules.len();
        let total_complexity: u32 = capsules.iter().map(|c| c.complexity).sum();
        let total_warnings: usize = capsules.iter().map(|c| c.warnings.len()).sum();
        let total_lines: usize = capsules.iter().map(|c| c.line_end - c.line_start + 1).sum();
        
        let avg_complexity = if total_capsules > 0 {
            total_complexity as f64 / total_capsules as f64
        } else {
            0.0
        };
        
        let avg_quality: f64 = capsules.iter()
            .map(|c| Self::analyze_quality(c))
            .sum::<f64>() / total_capsules.max(1) as f64;
        
        ProjectMetrics {
            total_capsules,
            total_complexity,
            total_warnings,
            total_lines,
            avg_complexity,
            avg_quality,
        }
    }
}

/// Project metrics structure
#[derive(Debug, Clone)]
pub struct ProjectMetrics {
    pub total_capsules: usize,
    pub total_complexity: u32,
    pub total_warnings: usize,
    pub total_lines: usize,
    pub avg_complexity: f64,
    pub avg_quality: f64,
} 