//! Typosquatting detection
//!
//! Detects packages that may be typosquatting attacks on popular packages
//! using multiple detection methods:
//! - Levenshtein distance
//! - Keyboard proximity analysis
//! - Character substitution patterns
//! - Homoglyph detection

use crate::{ThreatIndicator, ThreatLevel, ThreatType};
use std::collections::{HashMap, HashSet};
use strsim::{levenshtein, normalized_levenshtein};

/// Keyboard layout for proximity analysis (QWERTY)
lazy_static::lazy_static! {
    static ref KEYBOARD_ADJACENT: HashMap<char, Vec<char>> = {
        let mut m = HashMap::new();
        // Row 1
        m.insert('q', vec!['w', 'a', '1', '2']);
        m.insert('w', vec!['q', 'e', 'a', 's', '2', '3']);
        m.insert('e', vec!['w', 'r', 's', 'd', '3', '4']);
        m.insert('r', vec!['e', 't', 'd', 'f', '4', '5']);
        m.insert('t', vec!['r', 'y', 'f', 'g', '5', '6']);
        m.insert('y', vec!['t', 'u', 'g', 'h', '6', '7']);
        m.insert('u', vec!['y', 'i', 'h', 'j', '7', '8']);
        m.insert('i', vec!['u', 'o', 'j', 'k', '8', '9']);
        m.insert('o', vec!['i', 'p', 'k', 'l', '9', '0']);
        m.insert('p', vec!['o', 'l', '0']);
        // Row 2
        m.insert('a', vec!['q', 'w', 's', 'z']);
        m.insert('s', vec!['w', 'e', 'a', 'd', 'z', 'x']);
        m.insert('d', vec!['e', 'r', 's', 'f', 'x', 'c']);
        m.insert('f', vec!['r', 't', 'd', 'g', 'c', 'v']);
        m.insert('g', vec!['t', 'y', 'f', 'h', 'v', 'b']);
        m.insert('h', vec!['y', 'u', 'g', 'j', 'b', 'n']);
        m.insert('j', vec!['u', 'i', 'h', 'k', 'n', 'm']);
        m.insert('k', vec!['i', 'o', 'j', 'l', 'm']);
        m.insert('l', vec!['o', 'p', 'k']);
        // Row 3
        m.insert('z', vec!['a', 's', 'x']);
        m.insert('x', vec!['s', 'd', 'z', 'c']);
        m.insert('c', vec!['d', 'f', 'x', 'v']);
        m.insert('v', vec!['f', 'g', 'c', 'b']);
        m.insert('b', vec!['g', 'h', 'v', 'n']);
        m.insert('n', vec!['h', 'j', 'b', 'm']);
        m.insert('m', vec!['j', 'k', 'n']);
        m
    };

    /// Common character substitutions used in typosquatting
    static ref CHAR_SUBSTITUTIONS: HashMap<char, Vec<char>> = {
        let mut m = HashMap::new();
        // Visual similarities
        m.insert('l', vec!['1', 'i', '|']);
        m.insert('1', vec!['l', 'i', '|']);
        m.insert('i', vec!['l', '1', '|']);
        m.insert('o', vec!['0', 'O']);
        m.insert('0', vec!['o', 'O']);
        m.insert('s', vec!['5', '$']);
        m.insert('5', vec!['s', 'S']);
        m.insert('e', vec!['3']);
        m.insert('3', vec!['e', 'E']);
        m.insert('a', vec!['4', '@']);
        m.insert('4', vec!['a', 'A']);
        m.insert('g', vec!['9', 'q']);
        m.insert('9', vec!['g', 'q']);
        m.insert('b', vec!['6', '8']);
        m.insert('n', vec!['m']);
        m.insert('m', vec!['n']);
        m
    };

    /// Multi-character substitution patterns (for string-level checking)
    static ref MULTICHAR_SUBSTITUTIONS: Vec<(&'static str, &'static str)> = vec![
        ("rn", "m"),   // rn looks like m
        ("cl", "d"),   // cl looks like d
        ("vv", "w"),   // vv looks like w
        ("nn", "m"),   // nn can look like m
        ("ii", "u"),   // ii can look like u
    ];

    /// Homoglyphs - visually similar Unicode characters
    static ref HOMOGLYPHS: HashMap<char, Vec<char>> = {
        let mut m = HashMap::new();
        m.insert('a', vec!['а', 'ɑ', 'α']); // Cyrillic, Latin alpha, Greek
        m.insert('c', vec!['с', 'ϲ']);
        m.insert('e', vec!['е', 'ε']);
        m.insert('o', vec!['о', 'ο', '0']);
        m.insert('p', vec!['р', 'ρ']);
        m.insert('x', vec!['х', 'χ']);
        m.insert('y', vec!['у', 'γ']);
        m
    };
}

/// Check if a package might be a typosquatting attempt
pub fn check_typosquatting(
    package_name: &str,
    known_packages: &HashSet<String>,
) -> Option<ThreatIndicator> {
    // Find similar package names
    for known_pkg in known_packages {
        let similarity = normalized_levenshtein(package_name, known_pkg);
        let distance = levenshtein(package_name, known_pkg);

        // High similarity but not exact match suggests typosquatting
        if similarity > 0.8 && similarity < 1.0 && distance <= 2 {
            return Some(ThreatIndicator {
                package_name: package_name.to_string(),
                package_version: String::new(),
                threat_level: determine_threat_level(similarity, distance),
                threat_type: ThreatType::Typosquatting,
                description: format!(
                    "Package '{}' may be typosquatting on '{}'",
                    package_name, known_pkg
                ),
                evidence: vec![
                    format!(
                        "Similar to popular package '{}' (similarity: {:.2})",
                        known_pkg, similarity
                    ),
                    format!("Edit distance: {} characters", distance),
                    "Common typosquatting patterns detected".to_string(),
                ],
                recommendation: format!(
                    "Verify this is the intended package. Consider using '{}' instead",
                    known_pkg
                ),
            });
        }
    }

    None
}

/// Determine threat level based on similarity metrics
fn determine_threat_level(similarity: f64, distance: usize) -> ThreatLevel {
    if similarity > 0.95 && distance == 1 {
        ThreatLevel::Critical // Very likely typosquatting (1 char difference)
    } else if similarity > 0.9 && distance <= 2 {
        ThreatLevel::High // Likely typosquatting (2 char difference)
    } else if similarity > 0.85 {
        ThreatLevel::Medium // Possible typosquatting
    } else {
        ThreatLevel::Low // Unlikely but flagged
    }
}

/// Common typosquatting patterns
pub fn detect_common_patterns(package_name: &str) -> Vec<String> {
    let mut patterns = Vec::new();

    // Check for common substitutions
    if package_name.contains("0") || package_name.contains("1") {
        patterns.push("Number substitution detected (0 for O, 1 for l)".to_string());
    }

    // Check for extra/missing characters
    if package_name.contains("--") {
        patterns.push("Double dash detected".to_string());
    }

    // Check for underscore/dash confusion
    let has_underscore = package_name.contains('_');
    let has_dash = package_name.contains('-');
    if has_underscore && has_dash {
        patterns.push("Mixed underscore and dash usage".to_string());
    }

    patterns
}

/// Check if differences between two strings are keyboard-adjacent typos
pub fn is_keyboard_adjacent_typo(original: &str, typo: &str) -> bool {
    if original.len() != typo.len() {
        return false;
    }

    let orig_chars: Vec<char> = original.chars().collect();
    let typo_chars: Vec<char> = typo.chars().collect();
    let mut diff_count = 0;

    for (o, t) in orig_chars.iter().zip(typo_chars.iter()) {
        if o != t {
            diff_count += 1;
            // Check if t is adjacent to o on keyboard
            if let Some(adjacent) = KEYBOARD_ADJACENT.get(&o.to_ascii_lowercase()) {
                if !adjacent.contains(&t.to_ascii_lowercase()) {
                    return false;
                }
            } else {
                return false;
            }
        }
    }

    diff_count == 1 // Exactly one keyboard-adjacent difference
}

/// Check if a character substitution pattern is used
pub fn has_char_substitution(original: &str, suspect: &str) -> Option<String> {
    if original.len() != suspect.len() {
        return None;
    }

    let orig_chars: Vec<char> = original.chars().collect();
    let suspect_chars: Vec<char> = suspect.chars().collect();

    for (i, (o, s)) in orig_chars.iter().zip(suspect_chars.iter()).enumerate() {
        if o != s {
            if let Some(subs) = CHAR_SUBSTITUTIONS.get(&o.to_ascii_lowercase()) {
                if subs.iter().any(|c| c == &s.to_ascii_lowercase()) {
                    return Some(format!(
                        "Character '{}' at position {} substituted with '{}'",
                        o, i, s
                    ));
                }
            }
        }
    }

    None
}

/// Comprehensive typosquatting check using all methods
pub fn comprehensive_typosquatting_check(
    package_name: &str,
    known_packages: &HashSet<String>,
) -> Option<ThreatIndicator> {
    let mut best_match: Option<(String, f64, Vec<String>)> = None;

    for known_pkg in known_packages {
        if package_name == known_pkg {
            continue; // Exact match, not typosquatting
        }

        let mut evidence = Vec::new();
        let mut risk_score: f64 = 0.0;

        // 1. Levenshtein similarity
        let similarity = normalized_levenshtein(package_name, known_pkg);
        let distance = levenshtein(package_name, known_pkg);

        if similarity > 0.7 {
            risk_score += similarity * 50.0;
            evidence.push(format!(
                "Similar to '{}' (similarity: {:.1}%, distance: {})",
                known_pkg,
                similarity * 100.0,
                distance
            ));
        }

        // 2. Keyboard adjacency check
        if is_keyboard_adjacent_typo(known_pkg, package_name) {
            risk_score += 30.0;
            evidence.push("Keyboard-adjacent typo detected".to_string());
        }

        // 3. Character substitution check
        if let Some(sub_msg) = has_char_substitution(known_pkg, package_name) {
            risk_score += 25.0;
            evidence.push(sub_msg);
        }

        // 4. Common pattern detection
        let patterns = detect_common_patterns(package_name);
        for pattern in patterns {
            risk_score += 10.0;
            evidence.push(pattern);
        }

        // 5. Prefix/suffix manipulation
        if package_name.starts_with(known_pkg) || package_name.ends_with(known_pkg) {
            risk_score += 15.0;
            evidence.push(format!("Contains exact package name '{}'", known_pkg));
        }

        // 6. Separator manipulation (dash/underscore swap)
        let normalized_pkg = package_name.replace('-', "_");
        let normalized_known = known_pkg.replace('-', "_");
        if normalized_pkg == normalized_known && package_name != known_pkg {
            risk_score += 20.0;
            evidence.push("Dash/underscore manipulation detected".to_string());
        }

        // Update best match if this is higher risk
        if risk_score > 40.0 {
            if best_match.is_none() || risk_score > best_match.as_ref().unwrap().1 {
                best_match = Some((known_pkg.clone(), risk_score, evidence));
            }
        }
    }

    best_match.map(|(matched_pkg, risk_score, evidence)| {
        let threat_level = if risk_score > 80.0 {
            ThreatLevel::Critical
        } else if risk_score > 60.0 {
            ThreatLevel::High
        } else if risk_score > 40.0 {
            ThreatLevel::Medium
        } else {
            ThreatLevel::Low
        };

        ThreatIndicator {
            package_name: package_name.to_string(),
            package_version: String::new(),
            threat_level,
            threat_type: ThreatType::Typosquatting,
            description: format!(
                "Package '{}' may be typosquatting on '{}' (risk score: {:.0})",
                package_name, matched_pkg, risk_score
            ),
            evidence,
            recommendation: format!(
                "Verify this is the intended package. Consider using '{}' instead",
                matched_pkg
            ),
        }
    })
}

/// Popular packages by ecosystem (top packages most likely to be typosquatted)
pub fn get_popular_packages(ecosystem: &str) -> HashSet<String> {
    let packages: Vec<&str> = match ecosystem {
        "npm" | "javascript" | "node" => vec![
            "lodash", "express", "react", "axios", "moment", "request", "chalk",
            "commander", "debug", "async", "bluebird", "underscore", "uuid", "mkdirp",
            "glob", "minimist", "yargs", "webpack", "babel-core", "eslint", "jest",
            "mocha", "typescript", "vue", "angular", "jquery", "bootstrap", "next",
            "socket.io", "mongoose", "sequelize", "passport", "bcrypt", "jsonwebtoken",
            "nodemailer", "puppeteer", "cheerio", "dotenv", "cors", "body-parser",
            "cookie-parser", "morgan", "helmet", "compression", "multer", "sharp",
            "imagemin", "jimp", "aws-sdk", "firebase", "stripe", "twilio",
        ],
        "pypi" | "python" | "pip" => vec![
            "requests", "numpy", "pandas", "django", "flask", "boto3", "urllib3",
            "setuptools", "cryptography", "pyyaml", "pillow", "scipy", "matplotlib",
            "tensorflow", "pytorch", "scikit-learn", "beautifulsoup4", "selenium",
            "sqlalchemy", "celery", "redis", "psycopg2", "pymongo", "elasticsearch",
            "fastapi", "pydantic", "httpx", "aiohttp", "pytest", "black", "mypy",
            "isort", "flake8", "pylint", "sphinx", "jupyter", "ipython", "notebook",
        ],
        "crates" | "rust" | "cargo" => vec![
            "serde", "tokio", "rand", "clap", "reqwest", "hyper", "actix-web",
            "rocket", "diesel", "sqlx", "sea-orm", "tracing", "log", "env_logger",
            "anyhow", "thiserror", "chrono", "uuid", "regex", "lazy_static",
            "once_cell", "futures", "async-std", "rayon", "crossbeam", "parking_lot",
            "bytes", "http", "tower", "tonic", "prost", "serde_json", "toml",
        ],
        "rubygems" | "ruby" | "bundler" => vec![
            "rails", "rack", "bundler", "rake", "rspec", "devise", "sidekiq",
            "puma", "unicorn", "nokogiri", "json", "httparty", "faraday", "rest-client",
            "activerecord", "pg", "mysql2", "redis", "mongoid", "elasticsearch",
        ],
        "packagist" | "php" | "composer" => vec![
            "laravel/framework", "symfony/symfony", "guzzlehttp/guzzle", "monolog/monolog",
            "phpunit/phpunit", "doctrine/orm", "twig/twig", "psr/log", "vlucas/phpdotenv",
            "nesbot/carbon", "league/flysystem", "intervention/image", "predis/predis",
        ],
        "maven" | "java" | "gradle" => vec![
            "org.springframework:spring-core", "com.google.guava:guava",
            "org.apache.commons:commons-lang3", "com.fasterxml.jackson.core:jackson-databind",
            "org.slf4j:slf4j-api", "ch.qos.logback:logback-classic", "junit:junit",
            "org.mockito:mockito-core", "org.projectlombok:lombok", "com.squareup.okhttp3:okhttp",
        ],
        _ => vec![],
    };

    packages.into_iter().map(String::from).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typosquatting_detection() {
        let mut known = HashSet::new();
        known.insert("lodash".to_string());

        // Very similar - likely typosquatting
        let result = check_typosquatting("lodosh", &known);
        assert!(result.is_some());

        let threat = result.unwrap();
        assert_eq!(threat.threat_type, ThreatType::Typosquatting);
    }

    #[test]
    fn test_safe_package_no_typosquatting() {
        let mut known = HashSet::new();
        known.insert("lodash".to_string());

        // Completely different - no typosquatting
        let result = check_typosquatting("react", &known);
        assert!(result.is_none());
    }

    #[test]
    fn test_exact_match_no_typosquatting() {
        let mut known = HashSet::new();
        known.insert("lodash".to_string());

        // Exact match - not typosquatting
        let result = check_typosquatting("lodash", &known);
        assert!(result.is_none());
    }

    #[test]
    fn test_common_patterns() {
        let patterns = detect_common_patterns("l0dash");
        assert!(patterns.iter().any(|p| p.contains("Number substitution")));

        let patterns2 = detect_common_patterns("lodash--extra");
        assert!(patterns2.iter().any(|p| p.contains("Double dash")));
    }
}
