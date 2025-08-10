// Продвинутые метрики для анализа качества кода и архитектуры
// Рефакторенная версия - использует модульную архитектуру

use crate::enrichment::{QualityAnalyzer, SemanticEnricher};
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Калькулятор продвинутых метрик - композитный класс, использующий специализированные анализаторы
#[derive(Debug)]
pub struct AdvancedMetricsCalculator {
    semantic_enricher: SemanticEnricher,
    quality_analyzer: QualityAnalyzer,
}

/// Результат расчета продвинутых метрик
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedMetrics {
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub maintainability_index: f32,
    pub solid_score: SOLIDScore,
    pub code_quality_index: f32,
    pub technical_debt_ratio: f32,
    pub halstead_metrics: HalsteadMetrics,
}

/// Оценка соответствия SOLID принципам
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SOLIDScore {
    pub srp_score: f32,
    pub ocp_score: f32,
    pub lsp_score: f32,
    pub isp_score: f32,
    pub dip_score: f32,
    pub overall_score: f32,
}

/// Метрики Холстеда
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HalsteadMetrics {
    pub vocabulary: u32,
    pub length: u32,
    pub volume: f32,
    pub difficulty: f32,
    pub effort: f32,
    pub time: f32,
    pub bugs: f32,
}

impl AdvancedMetricsCalculator {
    pub fn new() -> Self {
        Self {
            semantic_enricher: SemanticEnricher::new(),
            quality_analyzer: QualityAnalyzer::new(),
        }
    }

    /// Расчет продвинутых метрик для капсулы
    pub fn calculate_metrics(&self, capsule: &Capsule, content: &str) -> Result<AdvancedMetrics> {
        // Используем качественный анализ для получения основных метрик
        let quality_assessment = self.quality_analyzer.analyze_quality(capsule, content)?;

        // Используем семантический анализ для дополнительных метрик
        let semantic_analysis = self
            .semantic_enricher
            .perform_semantic_analysis(capsule, content)?;

        // Расчет цикломатической сложности
        let cyclomatic_complexity = self.calculate_cyclomatic_complexity(content)?;

        // Расчет когнитивной сложности
        let cognitive_complexity = semantic_analysis.quality_metrics.cognitive_complexity;

        // Расчет SOLID оценки
        let solid_score = self.calculate_solid_score(content)?;

        // Расчет метрик Холстеда
        let halstead_metrics = self.calculate_halstead_metrics(content)?;

        // Расчет индекса сопровождаемости
        let maintainability_index = quality_assessment.maintainability_index;

        // Расчет индекса качества кода
        let code_quality_index = quality_assessment.overall_score;

        // Расчет коэффициента технического долга
        let technical_debt_ratio = quality_assessment.technical_debt_score / 100.0;

        Ok(AdvancedMetrics {
            cyclomatic_complexity,
            cognitive_complexity,
            maintainability_index,
            solid_score,
            code_quality_index,
            technical_debt_ratio,
            halstead_metrics,
        })
    }

    /// Расчет цикломатической сложности
    fn calculate_cyclomatic_complexity(&self, content: &str) -> Result<u32> {
        let mut complexity = 1; // Базовая сложность

        // Простой подсчет операторов управления потоком
        complexity += content.matches("if ").count() as u32;
        complexity += content.matches("else").count() as u32;
        complexity += content.matches("while ").count() as u32;
        complexity += content.matches("for ").count() as u32;
        complexity += content.matches("match ").count() as u32;
        complexity += content.matches("switch ").count() as u32;
        complexity += content.matches("case ").count() as u32;
        complexity += content.matches("catch ").count() as u32;
        complexity += content.matches("||").count() as u32;
        complexity += content.matches("&&").count() as u32;

        Ok(complexity)
    }

    /// Расчет SOLID оценки
    fn calculate_solid_score(&self, content: &str) -> Result<SOLIDScore> {
        // Упрощенная оценка SOLID принципов
        let mut srp_score: f32 = 1.0;
        let mut ocp_score: f32 = 1.0;
        let mut lsp_score: f32 = 1.0;
        let mut isp_score: f32 = 1.0;
        let mut dip_score: f32 = 1.0;

        // SRP - Single Responsibility Principle
        let function_count = content.matches("fn ").count()
            + content.matches("function ").count()
            + content.matches("def ").count();
        let lines_count = content.lines().count();
        if lines_count > 0 && function_count > 0 {
            let avg_lines_per_function = lines_count / function_count;
            if avg_lines_per_function > 50 {
                srp_score -= 0.3;
            }
        }

        // OCP - Open/Closed Principle
        if content.contains("trait ")
            || content.contains("interface ")
            || content.contains("abstract ")
        {
            ocp_score += 0.2;
        }

        // LSP - Liskov Substitution Principle
        if content.contains("panic!") || content.contains("throw ") {
            lsp_score -= 0.3;
        }

        // ISP - Interface Segregation Principle
        let interface_methods = content.matches("pub fn").count();
        if interface_methods > 10 {
            isp_score -= 0.2;
        }

        // DIP - Dependency Inversion Principle
        if content.contains("Box<dyn") || content.contains("impl ") {
            dip_score += 0.2;
        }

        let overall_score: f32 = (srp_score + ocp_score + lsp_score + isp_score + dip_score) / 5.0;

        Ok(SOLIDScore {
            srp_score: srp_score.max(0.0).min(1.0),
            ocp_score: ocp_score.max(0.0).min(1.0),
            lsp_score: lsp_score.max(0.0).min(1.0),
            isp_score: isp_score.max(0.0).min(1.0),
            dip_score: dip_score.max(0.0).min(1.0),
            overall_score: overall_score.max(0.0).min(1.0),
        })
    }

    /// Расчет метрик Холстеда
    fn calculate_halstead_metrics(&self, content: &str) -> Result<HalsteadMetrics> {
        // Упрощенный расчет метрик Холстеда
        let operators = self.count_operators(content);
        let operands = self.count_operands(content);

        let n1 = operators.len() as u32; // Количество уникальных операторов
        let n2 = operands.len() as u32; // Количество уникальных операндов
        let big_n1 = operators.values().sum::<u32>(); // Общее количество операторов
        let big_n2 = operands.values().sum::<u32>(); // Общее количество операндов

        let vocabulary = n1 + n2;
        let length = big_n1 + big_n2;
        let volume = if vocabulary > 0 {
            length as f32 * (vocabulary as f32).log2()
        } else {
            0.0
        };
        let difficulty = if n2 > 0 {
            (n1 as f32 / 2.0) * (big_n2 as f32 / n2 as f32)
        } else {
            0.0
        };
        let effort = difficulty * volume;
        let time = effort / 18.0; // Секунды
        let bugs = volume / 3000.0; // Предполагаемое количество ошибок

        Ok(HalsteadMetrics {
            vocabulary,
            length,
            volume,
            difficulty,
            effort,
            time,
            bugs,
        })
    }

    /// Подсчет операторов в коде
    fn count_operators(&self, content: &str) -> HashMap<String, u32> {
        let mut operators = HashMap::new();

        let operator_patterns = vec![
            "+", "-", "*", "/", "=", "==", "!=", "<", ">", "<=", ">=", "&&", "||", "!", "&", "|",
            "^", "<<", ">>", "%", "(", ")", "[", "]", "{", "}", ";", ",", ".",
        ];

        for pattern in operator_patterns {
            let count = content.matches(pattern).count() as u32;
            if count > 0 {
                operators.insert(pattern.to_string(), count);
            }
        }

        operators
    }

    /// Подсчет операндов в коде
    fn count_operands(&self, content: &str) -> HashMap<String, u32> {
        let mut operands = HashMap::new();

        // Простой подсчет идентификаторов и литералов
        for word in content.split_whitespace() {
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
            if !clean_word.is_empty()
                && (clean_word.chars().next().unwrap().is_alphabetic()
                    || clean_word.chars().all(|c| c.is_numeric()))
            {
                *operands.entry(clean_word.to_string()).or_insert(0) += 1;
            }
        }

        operands
    }
}

impl Default for AdvancedMetricsCalculator {
    fn default() -> Self {
        Self::new()
    }
}
