use std::collections::HashMap;

use strsim::{damerau_levenshtein, jaro_winkler, sorensen_dice};
use unicode_normalization::UnicodeNormalization;

use crate::{errors::Error, types::SoundomeResult};

pub enum SimilarityAlgorithm {
    Smart,
    JaroWinkler,
    DamerauLevenshtein,
    SorensenDice,
}

/**
 * Computes a similarity score between two strings, ranging from 0 to 100.
 *
 * This function can either return a result based on a unique algorithm, or combines all three in a "Smart" mode:
 * - **Jaro-Winkler (50%)**: Gives higher importance to matching prefixes,
 *   making it useful for handling typos and small variations.
 * - **Normalized Damerau-Levenshtein (30%)**: Accounts for insertions, deletions,
 *   substitutions, and adjacent transpositions, helping with common spelling mistakes.
 * - **Sorensen-Dice (20%)**: Uses bigram comparison, making it more tolerant
 *   to changes in word order (e.g., "Justin Bieber - Love Yourself" vs. "Love Yourself - Justin Bieber").
 *
 * The final score is a weighted average of these three metrics, scaled to a 0-1 range.
 */
pub fn string_similarity(s1: &str, s2: &str, similarity_algorithm: SimilarityAlgorithm) -> f64 {
    let normalized_s1 = normalize_string(s1);
    let normalized_s2 = normalize_string(s2);

    if (normalized_s1.is_empty() && normalized_s2.is_empty()) || normalized_s1 == normalized_s2 {
        return 1.0;
    }

    match similarity_algorithm {
        SimilarityAlgorithm::Smart => {
            let jaro = jaro_winkler(&normalized_s1, &normalized_s2);
            let damerau = normalized_damerau_levenshtein(&normalized_s1, &normalized_s2);
            let dice = sorensen_dice(&normalized_s1, &normalized_s2);

            // Weighted average
            let score = (0.50 * jaro) + (0.30 * damerau) + (0.20 * dice);
            score
        }
        SimilarityAlgorithm::JaroWinkler => jaro_winkler(&normalized_s1, &normalized_s2),
        SimilarityAlgorithm::DamerauLevenshtein => {
            normalized_damerau_levenshtein(&normalized_s1, &normalized_s2)
        }
        SimilarityAlgorithm::SorensenDice => sorensen_dice(&normalized_s1, &normalized_s2),
    }
}

/**
 * Normalizes the Damerau-Levenshtein distance to a 0-1 range.
 */
fn normalized_damerau_levenshtein(s1: &str, s2: &str) -> f64 {
    let max_len = s1.len().max(s2.len());
    if max_len == 0 {
        return 1.0;
    } // If both strings are empty, they are identical
    let distance = damerau_levenshtein(s1, s2);
    1.0 - (distance as f64 / max_len as f64)
}

/**
 * Normalizes and cleans the input string:
 * - Converts to lowercase
 * - Normalizes Unicode to remove accents (NFD normalization)
 * - Strips non-ASCII characters
 */
pub fn normalize_string(s: &str) -> String {
    s.to_lowercase() // Convert to lowercase
        .nfd() // Normalize using NFD (decomposing accented characters)
        .filter(|c| c.is_ascii()) // Remove non-ASCII characters
        .collect() // Collect into a new string
}

/**
 * Converts a string into a URL-friendly slug.
 */
pub fn slugify(s: &str) -> String {
    return slug::slugify(s);
}

/**
 * Applies a template to a context, replacing {placeholders}
 * with values using tinytemplate
 */
pub fn render_template(template: &str, context: &HashMap<&str, &str>) -> SoundomeResult<String> {
    let mut tt = tinytemplate::TinyTemplate::new();
    tt.add_template("template", template)
        .map_err(Error::TemplateRenderingError)?;
    tt.render("template", &context)
        .map_err(Error::TemplateRenderingError)
}

/**
 * Remove excluded words from a string.
 */
pub fn remove_excluded_words(s: &str, excluded_words: &Vec<&str>) -> String {
    let mut result = s.to_string();
    for word in excluded_words {
        result = result.replace(word, "");
    }
    result
}

/**
 * Composite function
 */
pub fn render_and_normalize_template(
    template: &str,
    context: &HashMap<&str, &str>,
    excluded_words: &Vec<&str>,
) -> SoundomeResult<String> {
    let rendered = render_template(template, context)?.to_lowercase();
    Ok(slugify(&normalize_string(&remove_excluded_words(
        &rendered,
        excluded_words,
    ))))
}

// ================================================================================================
// Tests
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_similarity_identical_strings() {
        let s1 = "Hello";
        let s2 = "Hello";

        // Test for identical strings, similarity should be 100
        let similarity = string_similarity(s1, s2, SimilarityAlgorithm::Smart);
        assert_eq!(
            similarity, 1.0,
            "Identical strings should have a similarity of 100."
        );
    }

    #[test]
    fn test_string_similarity_empty_strings() {
        let s1 = "";
        let s2 = "";

        // Test for empty strings, similarity should be 100
        let similarity = string_similarity(s1, s2, SimilarityAlgorithm::Smart);
        assert_eq!(
            similarity, 1.0,
            "Empty strings should have a similarity of 100."
        );
    }

    #[test]
    fn test_string_similarity_typo() {
        let s1 = "hello";
        let s2 = "helo"; // Small typo

        // Test for small typos, similarity should still be high
        let similarity = string_similarity(s1, s2, SimilarityAlgorithm::Smart);
        assert!(
            similarity > 0.8,
            "Strings with small typos should have high similarity."
        );
    }

    #[test]
    fn test_string_similarity_transposition() {
        let s1 = "hello";
        let s2 = "holle"; // Adjacent transposition

        // Test for transposition, similarity should still be reasonable
        let similarity = string_similarity(s1, s2, SimilarityAlgorithm::Smart);
        assert!(
            similarity >= 0.5 && similarity < 0.8,
            "Strings with adjacent transpositions should have medium similarity."
        );
    }

    #[test]
    fn test_string_similarity_different_strings() {
        let s1 = "apple";
        let s2 = "orange";

        // Test for completely different strings, similarity should be low
        let similarity = string_similarity(s1, s2, SimilarityAlgorithm::Smart);
        assert!(
            similarity < 0.5,
            "Completely different strings should have low similarity."
        );
    }

    #[test]
    fn test_normalized_damerau_levenshtein_identical_strings() {
        let s1 = "example";
        let s2 = "example";

        // Test for identical strings, Damerau-Levenshtein distance should be 0 (normalized similarity 1.0)
        let normalized_distance = normalized_damerau_levenshtein(s1, s2);
        assert_eq!(
            normalized_distance, 1.0,
            "Identical strings should have a normalized Damerau-Levenshtein similarity of 1.0."
        );
    }

    #[test]
    fn test_normalized_damerau_levenshtein_different_strings() {
        let s1 = "apple";
        let s2 = "orange";

        // Test for completely different strings, similarity should be low
        let normalized_distance = normalized_damerau_levenshtein(s1, s2);
        assert!(
            normalized_distance < 0.5,
            "Completely different strings should have low Damerau-Levenshtein similarity."
        );
    }

    #[test]
    fn test_normalize_string_with_accents() {
        let s = "école";

        // Test for string with accents, normalize to "ecole"
        let normalized = normalize_string(s);
        assert_eq!(
            normalized, "ecole",
            "String with accents should be normalized correctly."
        );
    }

    #[test]
    fn test_normalize_string_with_non_ascii() {
        let s = "naïve";

        // Test for string with non-ASCII characters, normalize to "naive"
        let normalized = normalize_string(s);
        assert_eq!(
            normalized, "naive",
            "String with non-ASCII characters should be normalized correctly."
        );
    }

    #[test]
    fn test_normalize_string_lowercase() {
        let s = "Hello";

        // Test for string normalization to lowercase
        let normalized = normalize_string(s);
        assert_eq!(
            normalized, "hello",
            "String should be normalized to lowercase."
        );
    }

    #[test]
    fn test_normalize_string_empty() {
        let s = "";

        // Test for empty string, should return empty string
        let normalized = normalize_string(s);
        assert_eq!(
            normalized, "",
            "Empty string should remain empty after normalization."
        );
    }
}
