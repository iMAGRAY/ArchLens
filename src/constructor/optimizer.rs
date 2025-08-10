use crate::types::{AnalysisWarning, Capsule, CapsuleStatus, CapsuleType, Priority, Result};
use std::collections::HashSet;

/// Capsule optimizer - optimizes capsule collections
pub struct CapsuleOptimizer;

impl CapsuleOptimizer {
    /// Optimizes capsules
    pub fn optimize_capsules(capsules: &mut Vec<Capsule>) -> Result<()> {
        // Remove duplicates
        Self::remove_duplicates(capsules);

        // Merge small capsules
        Self::merge_small_capsules(capsules)?;

        // Sort by priority
        capsules.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(())
    }

    /// Removes duplicate capsules
    fn remove_duplicates(capsules: &mut Vec<Capsule>) {
        let mut seen = HashSet::new();
        capsules.retain(|capsule| {
            let key = (
                capsule.name.clone(),
                capsule.file_path.clone(),
                capsule.line_start,
            );
            seen.insert(key)
        });
    }

    /// Merges small capsules into larger ones
    fn merge_small_capsules(capsules: &mut [Capsule]) -> Result<()> {
        let mut merged_indices = HashSet::new();
        let mut merge_groups: Vec<Vec<usize>> = Vec::new();

        // Group small capsules by merge criteria
        for i in 0..capsules.len() {
            if merged_indices.contains(&i) {
                continue;
            }

            let capsule = &capsules[i];

            // Check merge criteria
            if Self::should_merge_capsule(capsule) {
                let mut group = vec![i];
                merged_indices.insert(i);

                // Find similar capsules for merging
                for (j, other_capsule) in capsules.iter().enumerate().skip(i + 1) {
                    if merged_indices.contains(&j) {
                        continue;
                    }

                    if Self::should_merge_capsule(other_capsule)
                        && Self::can_merge_capsules(capsule, other_capsule)
                    {
                        group.push(j);
                        merged_indices.insert(j);
                    }
                }

                // If group contains more than one capsule, add it for merging
                if group.len() > 1 {
                    merge_groups.push(group);
                }
            }
        }

        // Execute merging of groups (in reverse index order)
        for group in merge_groups.into_iter().rev() {
            if group.len() > 1 {
                Self::merge_capsule_group(capsules, group)?;
            }
        }

        Ok(())
    }

    /// Checks if capsule should be merged
    fn should_merge_capsule(capsule: &Capsule) -> bool {
        let size = capsule.line_end - capsule.line_start + 1;

        // Merge criteria:
        // - Size < 10 lines
        // - Low complexity
        // - Utility types
        size < 10
            || capsule.complexity < 3
            || matches!(
                capsule.capsule_type,
                CapsuleType::Constant
                    | CapsuleType::Variable
                    | CapsuleType::Import
                    | CapsuleType::Export
            )
    }

    /// Checks if two capsules can be merged
    fn can_merge_capsules(capsule1: &Capsule, capsule2: &Capsule) -> bool {
        // Must be in same file
        if capsule1.file_path != capsule2.file_path {
            return false;
        }

        // Must be in same layer
        if capsule1.layer != capsule2.layer {
            return false;
        }

        // Must be compatible types
        Self::are_compatible_types(&capsule1.capsule_type, &capsule2.capsule_type)
    }

    /// Checks compatibility of capsule types
    fn are_compatible_types(type1: &CapsuleType, type2: &CapsuleType) -> bool {
        match (type1, type2) {
            // Constants and variables can be merged
            (CapsuleType::Constant, CapsuleType::Constant) => true,
            (CapsuleType::Variable, CapsuleType::Variable) => true,
            (CapsuleType::Constant, CapsuleType::Variable) => true,
            (CapsuleType::Variable, CapsuleType::Constant) => true,

            // Imports and exports can be merged
            (CapsuleType::Import, CapsuleType::Import) => true,
            (CapsuleType::Export, CapsuleType::Export) => true,
            (CapsuleType::Import, CapsuleType::Export) => true,
            (CapsuleType::Export, CapsuleType::Import) => true,

            // Functions and methods can be merged if they are small
            (CapsuleType::Function, CapsuleType::Function) => true,
            (CapsuleType::Method, CapsuleType::Method) => true,
            (CapsuleType::Function, CapsuleType::Method) => true,
            (CapsuleType::Method, CapsuleType::Function) => true,

            _ => false,
        }
    }

    /// Merges group of capsules into one
    fn merge_capsule_group(capsules: &mut [Capsule], group: Vec<usize>) -> Result<()> {
        if group.len() < 2 {
            return Ok(());
        }

        // Sort indices for stability
        let mut sorted_group = group;
        sorted_group.sort();

        // Create merged capsule based on first one
        let main_index = sorted_group[0];
        let mut merged_capsule = capsules[main_index].clone();

        // Merge data from other capsules
        let mut merged_names = vec![merged_capsule.name.clone()];
        let mut merged_warnings = merged_capsule.warnings.clone();
        let mut total_complexity = merged_capsule.complexity;
        let mut min_line = merged_capsule.line_start;
        let mut max_line = merged_capsule.line_end;

        for &index in &sorted_group[1..] {
            let capsule = &capsules[index];
            merged_names.push(capsule.name.clone());
            merged_warnings.extend(capsule.warnings.clone());
            total_complexity += capsule.complexity;
            min_line = min_line.min(capsule.line_start);
            max_line = max_line.max(capsule.line_end);
        }

        // Update merged capsule
        merged_capsule.name = format!("merged_{}", merged_names.join("_"));
        merged_capsule.slogan = Some(format!("Merged: {}", merged_names.join(", ")));
        merged_capsule.warnings = merged_warnings;
        merged_capsule.complexity = total_complexity;
        merged_capsule.line_start = min_line;
        merged_capsule.line_end = max_line;

        // Update priority based on new complexity
        merged_capsule.priority = if total_complexity > 15 {
            Priority::High
        } else if total_complexity > 8 {
            Priority::Medium
        } else {
            Priority::Low
        };

        // Write merged capsule
        capsules[main_index] = merged_capsule;

        // Mark other capsules as merged (can be removed later)
        for &index in &sorted_group[1..] {
            capsules[index].status = CapsuleStatus::Deprecated;
            capsules[index].warnings.push(AnalysisWarning {
                level: Priority::Low,
                message: "Merged into another capsule".to_string(),
                category: "optimization".to_string(),
                capsule_id: Some(capsules[index].id),
                suggestion: Some("Capsule was automatically merged for optimization".to_string()),
            });
        }

        Ok(())
    }
}
