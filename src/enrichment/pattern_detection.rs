// Модуль обнаружения архитектурных паттернов

use crate::enrichment::semantic_analysis::SemanticLink;
use crate::types::*;
use regex::Regex;
use std::collections::HashMap;

/// Архитектурные паттерны
#[derive(Debug, Clone)]
pub struct ArchitecturalPattern {
    pub pattern_type: PatternType,
    pub confidence: f32,
    pub description: String,
    pub evidence: Vec<String>,
}

/// Типы архитектурных паттернов
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PatternType {
    Singleton,
    Factory,
    Observer,
    Repository,
    Service,
    Controller,
    Builder,
    Adapter,
    Decorator,
    Command,
    Strategy,
    Template,
    MVC,
    MVP,
    MVVM,
    Layered,
    Microservices,
    EventDriven,
    PipeAndFilter,
}

/// Детектор паттернов
pub struct PatternDetector {
    pattern_rules: HashMap<PatternType, Vec<PatternRule>>,
}

/// Правило для обнаружения паттерна
#[derive(Debug, Clone)]
pub struct PatternRule {
    pub name: String,
    pub pattern: Regex,
    pub weight: f32,
    pub description: String,
}

impl PatternDetector {
    pub fn new() -> Self {
        Self {
            pattern_rules: Self::create_pattern_rules(),
        }
    }

    pub fn detect_patterns(
        &self,
        content: &str,
        semantic_links: &[SemanticLink],
    ) -> Result<Vec<ArchitecturalPattern>> {
        let mut patterns = Vec::new();

        for (pattern_type, rules) in &self.pattern_rules {
            let mut total_confidence = 0.0;
            let mut evidence = Vec::new();

            for rule in rules {
                let matches: Vec<_> = rule.pattern.captures_iter(content).collect();
                if !matches.is_empty() {
                    total_confidence += rule.weight * matches.len() as f32;
                    evidence.push(format!(
                        "{}: найдено {} совпадений",
                        rule.name,
                        matches.len()
                    ));
                }
            }

            // Учитываем семантические связи
            let semantic_boost = self.calculate_semantic_boost(pattern_type, semantic_links);
            total_confidence += semantic_boost;

            if total_confidence > 0.3 {
                patterns.push(ArchitecturalPattern {
                    pattern_type: pattern_type.clone(),
                    confidence: (total_confidence / rules.len() as f32).min(1.0),
                    description: self.get_pattern_description(pattern_type),
                    evidence,
                });
            }
        }

        // Сортируем по убыванию уверенности
        patterns.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(patterns)
    }

    fn calculate_semantic_boost(
        &self,
        pattern_type: &PatternType,
        semantic_links: &[SemanticLink],
    ) -> f32 {
        match pattern_type {
            PatternType::Singleton => {
                // Singleton обычно имеет статические методы
                semantic_links
                    .iter()
                    .filter(|link| {
                        link.target_name.contains("instance")
                            || link.target_name.contains("get_instance")
                    })
                    .count() as f32
                    * 0.2
            }
            PatternType::Factory => {
                // Factory обычно создает объекты
                semantic_links
                    .iter()
                    .filter(|link| {
                        link.target_name.contains("create") || link.target_name.contains("new")
                    })
                    .count() as f32
                    * 0.1
            }
            PatternType::Observer => {
                // Observer имеет методы подписки
                semantic_links
                    .iter()
                    .filter(|link| {
                        link.target_name.contains("subscribe")
                            || link.target_name.contains("notify")
                            || link.target_name.contains("update")
                    })
                    .count() as f32
                    * 0.15
            }
            PatternType::Repository => {
                // Repository имеет CRUD операции
                semantic_links
                    .iter()
                    .filter(|link| {
                        link.target_name.contains("save")
                            || link.target_name.contains("find")
                            || link.target_name.contains("delete")
                    })
                    .count() as f32
                    * 0.1
            }
            _ => 0.0,
        }
    }

    fn get_pattern_description(&self, pattern_type: &PatternType) -> String {
        match pattern_type {
            PatternType::Singleton => {
                "Паттерн Singleton обеспечивает создание единственного экземпляра класса"
                    .to_string()
            }
            PatternType::Factory => {
                "Паттерн Factory создает объекты без указания их конкретных классов".to_string()
            }
            PatternType::Observer => {
                "Паттерн Observer определяет зависимость один-ко-многим между объектами".to_string()
            }
            PatternType::Strategy => {
                "Паттерн Strategy определяет семейство алгоритмов и делает их взаимозаменяемыми"
                    .to_string()
            }
            PatternType::Command => "Паттерн Command инкапсулирует запрос как объект".to_string(),
            PatternType::Builder => {
                "Паттерн Builder конструирует сложные объекты пошагово".to_string()
            }
            PatternType::Adapter => {
                "Паттерн Adapter позволяет объектам с несовместимыми интерфейсами работать вместе"
                    .to_string()
            }
            PatternType::Repository => {
                "Паттерн Repository инкапсулирует логику доступа к данным".to_string()
            }
            PatternType::Service => "Паттерн Service содержит бизнес-логику приложения".to_string(),
            PatternType::Controller => {
                "Паттерн Controller обрабатывает входящие запросы".to_string()
            }
            PatternType::Decorator => {
                "Паттерн Decorator добавляет новую функциональность объектам".to_string()
            }
            PatternType::Template => {
                "Паттерн Template Method определяет скелет алгоритма в базовом классе".to_string()
            }
            PatternType::MVC => {
                "Паттерн MVC разделяет приложение на модель, представление и контроллер".to_string()
            }
            PatternType::MVP => {
                "Паттерн MVP использует презентер для управления представлением".to_string()
            }
            PatternType::MVVM => {
                "Паттерн MVVM связывает представление с моделью через view model".to_string()
            }
            PatternType::Layered => {
                "Слоистая архитектура организует код в логические слои".to_string()
            }
            PatternType::Microservices => {
                "Микросервисная архитектура разделяет приложение на независимые сервисы".to_string()
            }
            PatternType::EventDriven => {
                "Событийно-ориентированная архитектура основана на публикации и подписке на события"
                    .to_string()
            }
            PatternType::PipeAndFilter => {
                "Архитектура Pipe and Filter обрабатывает данные через цепочку фильтров".to_string()
            }
        }
    }

    fn create_pattern_rules() -> HashMap<PatternType, Vec<PatternRule>> {
        let mut rules = HashMap::new();

        // Singleton
        rules.insert(
            PatternType::Singleton,
            vec![
                PatternRule {
                    name: "Приватный конструктор".to_string(),
                    pattern: Regex::new(r"private\s+\w+\s*\(").unwrap(),
                    weight: 0.4,
                    description:
                        "Приватный конструктор для предотвращения создания экземпляров извне"
                            .to_string(),
                },
                PatternRule {
                    name: "Статический экземпляр".to_string(),
                    pattern: Regex::new(r"static\s+.*instance").unwrap(),
                    weight: 0.5,
                    description: "Статическое поле для хранения единственного экземпляра"
                        .to_string(),
                },
                PatternRule {
                    name: "Метод получения экземпляра".to_string(),
                    pattern: Regex::new(r"(get_instance|getInstance)\s*\(").unwrap(),
                    weight: 0.6,
                    description: "Публичный метод для получения экземпляра".to_string(),
                },
            ],
        );

        // Factory
        rules.insert(
            PatternType::Factory,
            vec![
                PatternRule {
                    name: "Создание объектов".to_string(),
                    pattern: Regex::new(r"(create|make|build)\w*\s*\(").unwrap(),
                    weight: 0.3,
                    description: "Методы создания объектов".to_string(),
                },
                PatternRule {
                    name: "Фабричный метод".to_string(),
                    pattern: Regex::new(r"fn\s+create_\w+").unwrap(),
                    weight: 0.4,
                    description: "Фабричные методы создания".to_string(),
                },
                PatternRule {
                    name: "Абстрактная фабрика".to_string(),
                    pattern: Regex::new(r"trait\s+\w*Factory").unwrap(),
                    weight: 0.5,
                    description: "Трейт абстрактной фабрики".to_string(),
                },
            ],
        );

        // Observer
        rules.insert(
            PatternType::Observer,
            vec![
                PatternRule {
                    name: "Подписка на события".to_string(),
                    pattern: Regex::new(r"(subscribe|add_observer|register)\s*\(").unwrap(),
                    weight: 0.4,
                    description: "Методы подписки на события".to_string(),
                },
                PatternRule {
                    name: "Уведомление".to_string(),
                    pattern: Regex::new(r"(notify|update|on_\w+)\s*\(").unwrap(),
                    weight: 0.5,
                    description: "Методы уведомления наблюдателей".to_string(),
                },
                PatternRule {
                    name: "Список наблюдателей".to_string(),
                    pattern: Regex::new(r"(observers|listeners|subscribers)").unwrap(),
                    weight: 0.3,
                    description: "Коллекция наблюдателей".to_string(),
                },
            ],
        );

        // Repository
        rules.insert(
            PatternType::Repository,
            vec![
                PatternRule {
                    name: "CRUD операции".to_string(),
                    pattern: Regex::new(r"(save|find|delete|update|create)\s*\(").unwrap(),
                    weight: 0.4,
                    description: "Базовые операции с данными".to_string(),
                },
                PatternRule {
                    name: "Репозиторий в названии".to_string(),
                    pattern: Regex::new(r"\w*Repository").unwrap(),
                    weight: 0.5,
                    description: "Название содержит Repository".to_string(),
                },
                PatternRule {
                    name: "Трейт репозитория".to_string(),
                    pattern: Regex::new(r"trait\s+\w*Repository").unwrap(),
                    weight: 0.6,
                    description: "Трейт для репозитория".to_string(),
                },
            ],
        );

        // Service
        rules.insert(
            PatternType::Service,
            vec![
                PatternRule {
                    name: "Сервис в названии".to_string(),
                    pattern: Regex::new(r"\w*Service").unwrap(),
                    weight: 0.4,
                    description: "Название содержит Service".to_string(),
                },
                PatternRule {
                    name: "Бизнес-логика".to_string(),
                    pattern: Regex::new(r"(process|handle|execute|perform)\s*\(").unwrap(),
                    weight: 0.3,
                    description: "Методы обработки бизнес-логики".to_string(),
                },
            ],
        );

        // Controller
        rules.insert(
            PatternType::Controller,
            vec![
                PatternRule {
                    name: "Контроллер в названии".to_string(),
                    pattern: Regex::new(r"\w*Controller").unwrap(),
                    weight: 0.5,
                    description: "Название содержит Controller".to_string(),
                },
                PatternRule {
                    name: "HTTP методы".to_string(),
                    pattern: Regex::new(r"(get|post|put|delete|patch)_\w+").unwrap(),
                    weight: 0.4,
                    description: "Методы HTTP обработки".to_string(),
                },
            ],
        );

        // Builder
        rules.insert(
            PatternType::Builder,
            vec![
                PatternRule {
                    name: "Билдер в названии".to_string(),
                    pattern: Regex::new(r"\w*Builder").unwrap(),
                    weight: 0.5,
                    description: "Название содержит Builder".to_string(),
                },
                PatternRule {
                    name: "Цепочка вызовов".to_string(),
                    pattern: Regex::new(r"with_\w+\s*\(").unwrap(),
                    weight: 0.4,
                    description: "Методы цепочки построения".to_string(),
                },
                PatternRule {
                    name: "Метод build".to_string(),
                    pattern: Regex::new(r"build\s*\(").unwrap(),
                    weight: 0.6,
                    description: "Финальный метод построения".to_string(),
                },
            ],
        );

        rules
    }
}

impl Default for PatternDetector {
    fn default() -> Self {
        Self::new()
    }
}
