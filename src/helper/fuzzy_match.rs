//! This module regroups utilities linked to fuzzy matching.

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::database::Subject;

pub fn match_subjects(subjects: &[Subject], query: &str, number_of_results: usize) -> Vec<Subject> {
    info!("Fuzzy matching subjects with query: {}", query);

    let mut matched: Vec<(i64, &Subject)> = subjects
        .iter()
        .filter_map(|subject| {
            SkimMatcherV2::default()
                .fuzzy_match(&subject.name, query)
                .map(|score| (score, subject))
        })
        .collect();

    matched.sort_by(|a, b| b.0.cmp(&a.0));

    matched
        .into_iter()
        .take(number_of_results)
        .map(|(_, subject)| (*subject).clone())
        .collect()
}
