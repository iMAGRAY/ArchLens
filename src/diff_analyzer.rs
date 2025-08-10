// Анализатор изменений архитектуры между версиями

use crate::types::Result;
use crate::types::*;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Анализатор diff между версиями архитектуры
#[derive(Debug)]
pub struct DiffAnalyzer {
    change_threshold: f32,
    impact_calculator: ImpactCalculator,
}

/// Калькулятор влияния изменений
#[derive(Debug)]
pub struct ImpactCalculator {
    breaking_change_patterns: Vec<String>,
    major_change_patterns: Vec<String>,
    complexity_change_threshold: f32,
}

impl DiffAnalyzer {
    pub fn new() -> Self {
        Self {
            change_threshold: 0.1,
            impact_calculator: ImpactCalculator::new(),
        }
    }

    /// Основной метод для анализа различий между двумя состояниями архитектуры
    pub fn analyze_diff(
        &self,
        current: &CapsuleGraph,
        previous: &CapsuleGraph,
    ) -> Result<DiffAnalysis> {
        let mut changes = Vec::new();

        // Анализ изменений в компонентах
        self.analyze_component_changes(current, previous, &mut changes)?;

        // Анализ изменений в связях
        self.analyze_relation_changes(current, previous, &mut changes)?;

        // Расчет различий в метриках
        let metrics_diff = self.calculate_metrics_diff(current, previous)?;

        // Оценка тренда качества
        let quality_trend = self.calculate_quality_trend(&metrics_diff, &changes)?;

        // Генерация рекомендаций
        let recommendations =
            self.generate_recommendations(&changes, &metrics_diff, &quality_trend)?;

        // Создание резюме
        let summary = self.generate_summary(&changes, &metrics_diff, &quality_trend)?;

        Ok(DiffAnalysis {
            changes,
            metrics_diff,
            quality_trend,
            recommendations,
            summary,
        })
    }

    /// Анализ изменений компонентов
    fn analyze_component_changes(
        &self,
        current: &CapsuleGraph,
        previous: &CapsuleGraph,
        changes: &mut Vec<ArchitectureChange>,
    ) -> Result<()> {
        let current_components: HashMap<String, &Capsule> = current
            .capsules
            .values()
            .map(|c| (c.name.clone(), c))
            .collect();

        let previous_components: HashMap<String, &Capsule> = previous
            .capsules
            .values()
            .map(|c| (c.name.clone(), c))
            .collect();

        // Найти добавленные компоненты
        for (name, capsule) in &current_components {
            if !previous_components.contains_key(name) {
                changes.push(ArchitectureChange {
                    change_type: ChangeType::Added,
                    component: name.clone(),
                    description: format!(
                        "Добавлен новый компонент '{}' типа {:?}",
                        name, capsule.capsule_type
                    ),
                    impact: self.impact_calculator.calculate_add_impact(capsule),
                    related_components: self.find_related_components(capsule, current),
                });
            }
        }

        // Найти удаленные компоненты
        for (name, capsule) in &previous_components {
            if !current_components.contains_key(name) {
                changes.push(ArchitectureChange {
                    change_type: ChangeType::Removed,
                    component: name.clone(),
                    description: format!(
                        "Удален компонент '{}' типа {:?}",
                        name, capsule.capsule_type
                    ),
                    impact: self
                        .impact_calculator
                        .calculate_remove_impact(capsule, previous),
                    related_components: self.find_related_components(capsule, previous),
                });
            }
        }

        // Найти измененные компоненты
        for (name, current_capsule) in &current_components {
            if let Some(previous_capsule) = previous_components.get(name) {
                self.analyze_component_modifications(current_capsule, previous_capsule, changes)?;
            }
        }

        Ok(())
    }

    /// Анализ модификаций компонента
    fn analyze_component_modifications(
        &self,
        current: &Capsule,
        previous: &Capsule,
        changes: &mut Vec<ArchitectureChange>,
    ) -> Result<()> {
        // Изменение сложности
        let complexity_delta = current.complexity as f32 - previous.complexity as f32;

        if complexity_delta.abs() > self.change_threshold {
            let change_type = if complexity_delta > 0.0 {
                ChangeType::ComplexityIncrease
            } else if complexity_delta < 0.0 {
                ChangeType::ComplexityDecrease
            } else {
                ChangeType::Modified
            };

            changes.push(ArchitectureChange {
                change_type,
                component: current.name.clone(),
                description: format!(
                    "Сложность '{}' изменилась с {} на {} (Δ{:+.1})",
                    current.name, previous.complexity, current.complexity, complexity_delta
                ),
                impact: if complexity_delta > 5.0 {
                    ChangeImpact::Major
                } else {
                    ChangeImpact::Minor
                },
                related_components: Vec::new(),
            });
        }

        // Изменение слоя
        if current.layer != previous.layer {
            changes.push(ArchitectureChange {
                change_type: ChangeType::Moved,
                component: current.name.clone(),
                description: format!(
                    "Компонент '{}' перемещен из слоя '{:?}' в слой '{:?}'",
                    current.name, previous.layer, current.layer
                ),
                impact: ChangeImpact::Refactoring,
                related_components: Vec::new(),
            });
        }

        // Изменение предупреждений
        if current.warnings.len() != previous.warnings.len() {
            let warning_delta = current.warnings.len() as i32 - previous.warnings.len() as i32;

            changes.push(ArchitectureChange {
                change_type: ChangeType::Modified,
                component: current.name.clone(),
                description: format!(
                    "Количество предупреждений для '{}' изменилось на {:+}",
                    current.name, warning_delta
                ),
                impact: if warning_delta > 0 {
                    ChangeImpact::Quality
                } else {
                    ChangeImpact::Quality
                },
                related_components: Vec::new(),
            });
        }

        Ok(())
    }

    /// Анализ изменений связей
    fn analyze_relation_changes(
        &self,
        current: &CapsuleGraph,
        previous: &CapsuleGraph,
        changes: &mut Vec<ArchitectureChange>,
    ) -> Result<()> {
        let current_relations: HashSet<(String, String)> = current
            .relations
            .iter()
            .filter_map(|r| {
                let from_name = current.capsules.get(&r.from_id)?.name.clone();
                let to_name = current.capsules.get(&r.to_id)?.name.clone();
                Some((from_name, to_name))
            })
            .collect();

        let previous_relations: HashSet<(String, String)> = previous
            .relations
            .iter()
            .filter_map(|r| {
                let from_name = previous.capsules.get(&r.from_id)?.name.clone();
                let to_name = previous.capsules.get(&r.to_id)?.name.clone();
                Some((from_name, to_name))
            })
            .collect();

        // Новые зависимости
        for (from, to) in &current_relations {
            if !previous_relations.contains(&(from.clone(), to.clone())) {
                changes.push(ArchitectureChange {
                    change_type: ChangeType::NewDependency,
                    component: from.clone(),
                    description: format!("Добавлена новая зависимость: '{}' -> '{}'", from, to),
                    impact: ChangeImpact::Minor,
                    related_components: vec![to.clone()],
                });
            }
        }

        // Удаленные зависимости
        for (from, to) in &previous_relations {
            if !current_relations.contains(&(from.clone(), to.clone())) {
                changes.push(ArchitectureChange {
                    change_type: ChangeType::RemovedDependency,
                    component: from.clone(),
                    description: format!("Удалена зависимость: '{}' -> '{}'", from, to),
                    impact: ChangeImpact::Refactoring,
                    related_components: vec![to.clone()],
                });
            }
        }

        Ok(())
    }

    /// Расчет разницы метрик
    fn calculate_metrics_diff(
        &self,
        current: &CapsuleGraph,
        previous: &CapsuleGraph,
    ) -> Result<MetricsDiff> {
        Ok(MetricsDiff {
            complexity_delta: current.metrics.complexity_average
                - previous.metrics.complexity_average,
            coupling_delta: current.metrics.coupling_index - previous.metrics.coupling_index,
            cohesion_delta: current.metrics.cohesion_index - previous.metrics.cohesion_index,
            component_count_delta: current.metrics.total_capsules as i32
                - previous.metrics.total_capsules as i32,
            relation_count_delta: current.metrics.total_relations as i32
                - previous.metrics.total_relations as i32,
            new_warnings: current.capsules.values().map(|c| c.warnings.len()).sum(),
            resolved_warnings: previous.capsules.values().map(|c| c.warnings.len()).sum(),
        })
    }

    /// Расчет тренда качества
    fn calculate_quality_trend(
        &self,
        metrics_diff: &MetricsDiff,
        changes: &[ArchitectureChange],
    ) -> Result<QualityTrend> {
        let mut quality_score = 0.0;

        // Анализ метрик
        if metrics_diff.complexity_delta < 0.0 {
            quality_score += 1.0;
        }
        if metrics_diff.coupling_delta < 0.0 {
            quality_score += 1.0;
        }
        if metrics_diff.cohesion_delta > 0.0 {
            quality_score += 1.0;
        }

        // Анализ предупреждений
        if metrics_diff.new_warnings < metrics_diff.resolved_warnings {
            quality_score += 1.0;
        }

        // Анализ изменений
        let breaking_changes = changes
            .iter()
            .filter(|c| c.impact == ChangeImpact::Breaking)
            .count();
        let quality_changes = changes
            .iter()
            .filter(|c| c.impact == ChangeImpact::Quality)
            .count();

        if breaking_changes > 0 {
            quality_score -= 2.0;
        }

        if quality_changes > 0 {
            quality_score += 0.5;
        }

        let trend = match quality_score {
            s if (s as f32).abs() <= 1.0 => QualityTrend::Stable,
            s if s > 1.0 => QualityTrend::Improving,
            s if s < -1.0 => QualityTrend::Degrading,
            _ => QualityTrend::Mixed,
        };

        Ok(trend)
    }

    /// Генерация рекомендаций
    fn generate_recommendations(
        &self,
        changes: &[ArchitectureChange],
        metrics_diff: &MetricsDiff,
        quality_trend: &QualityTrend,
    ) -> Result<Vec<String>> {
        let mut recommendations = Vec::new();

        // Рекомендации по метрикам
        if metrics_diff.complexity_delta > 1.0 {
            recommendations
                .push("Рассмотрите рефакторинг для снижения сложности компонентов".to_string());
        }

        if metrics_diff.coupling_delta > 0.1 {
            recommendations
                .push("Высокая связанность - используйте инверсию зависимостей".to_string());
        }

        if metrics_diff.cohesion_delta < -0.1 {
            recommendations
                .push("Низкая сплоченность - группируйте связанную функциональность".to_string());
        }

        // Рекомендации по изменениям
        let breaking_changes = changes
            .iter()
            .filter(|c| c.impact == ChangeImpact::Breaking)
            .count();
        if breaking_changes > 0 {
            recommendations.push(
                "Обнаружены критические изменения - обновите документацию и тесты".to_string(),
            );
        }

        let new_dependencies = changes
            .iter()
            .filter(|c| c.change_type == ChangeType::NewDependency)
            .count();
        if new_dependencies > 5 {
            recommendations
                .push("Много новых зависимостей - проверьте архитектурную целостность".to_string());
        }

        // Рекомендации по тренду качества
        match quality_trend {
            QualityTrend::Degrading => {
                recommendations
                    .push("Качество кода ухудшается - необходим технический аудит".to_string());
            }
            QualityTrend::Improving => {
                recommendations
                    .push("Качество кода улучшается - продолжайте в том же духе".to_string());
            }
            QualityTrend::Mixed => {
                recommendations.push(
                    "Смешанные изменения - сфокусируйтесь на критических областях".to_string(),
                );
            }
            QualityTrend::Stable => {
                recommendations.push(
                    "Архитектура стабильна - можно добавлять новую функциональность".to_string(),
                );
            }
        }

        Ok(recommendations)
    }

    /// Генерация краткого резюме
    fn generate_summary(
        &self,
        changes: &[ArchitectureChange],
        metrics_diff: &MetricsDiff,
        quality_trend: &QualityTrend,
    ) -> Result<String> {
        let total_changes = changes.len();
        let added_components = changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Added)
            .count();
        let removed_components = changes
            .iter()
            .filter(|c| c.change_type == ChangeType::Removed)
            .count();
        let modified_components = changes
            .iter()
            .filter(|c| {
                matches!(
                    c.change_type,
                    ChangeType::Modified
                        | ChangeType::ComplexityIncrease
                        | ChangeType::ComplexityDecrease
                )
            })
            .count();

        let summary = format!(
            "Обнаружено {} изменений: {} добавлено, {} удалено, {} изменено. \
            Сложность: {:+.1}, Связанность: {:+.2}, Сплоченность: {:+.2}. \
            Тренд качества: {:?}.",
            total_changes,
            added_components,
            removed_components,
            modified_components,
            metrics_diff.complexity_delta,
            metrics_diff.coupling_delta,
            metrics_diff.cohesion_delta,
            quality_trend
        );

        Ok(summary)
    }

    /// Поиск связанных компонентов
    fn find_related_components(&self, capsule: &Capsule, graph: &CapsuleGraph) -> Vec<String> {
        graph
            .relations
            .iter()
            .filter_map(|r| {
                if r.from_id == capsule.id {
                    graph.capsules.get(&r.to_id).map(|c| c.name.clone())
                } else if r.to_id == capsule.id {
                    graph.capsules.get(&r.from_id).map(|c| c.name.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}

impl ImpactCalculator {
    fn new() -> Self {
        Self {
            breaking_change_patterns: vec![
                "public.*removed".to_string(),
                "interface.*changed".to_string(),
                "signature.*modified".to_string(),
            ],
            major_change_patterns: vec![
                "dependency.*added".to_string(),
                "complexity.*increased".to_string(),
                "layer.*changed".to_string(),
            ],
            complexity_change_threshold: 5.0,
        }
    }

    fn calculate_add_impact(&self, capsule: &Capsule) -> ChangeImpact {
        match capsule.capsule_type {
            CapsuleType::Interface | CapsuleType::Class => ChangeImpact::Major,
            CapsuleType::Function | CapsuleType::Method => {
                if capsule.complexity > 10 {
                    ChangeImpact::Major
                } else {
                    ChangeImpact::Minor
                }
            }
            _ => ChangeImpact::Minor,
        }
    }

    fn calculate_remove_impact(&self, capsule: &Capsule, graph: &CapsuleGraph) -> ChangeImpact {
        let dependents_count = graph
            .relations
            .iter()
            .filter(|r| r.to_id == capsule.id)
            .count();

        if dependents_count > 5 {
            ChangeImpact::Breaking
        } else if dependents_count > 2 {
            ChangeImpact::Major
        } else {
            ChangeImpact::Minor
        }
    }
}

impl Default for DiffAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
