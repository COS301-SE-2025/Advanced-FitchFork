use crate::languages::Language;
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CoverageSummary {
    pub total_files: u64,
    pub total_lines: u64,
    pub covered_lines: u64,
    pub coverage_percent: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoverageFile {
    pub path: String,
    pub total_lines: u64,
    pub covered_lines: u64,
    pub coverage_percent: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoverageReport {
    pub generated_at: String,
    pub summary: CoverageSummary,
    pub files: Vec<CoverageFile>,
}

pub struct CoverageProcessor;

impl CoverageProcessor {
    pub fn process_report(
        language: Language,
        content: &str,
        whitelist: &[String],
    ) -> Result<String, String> {
        match language {
            Language::Cpp => Self::parse_cpp_report(content, whitelist),
            Language::Java => Self::parse_java_report(content, whitelist),
            other => Err(format!(
                "Code coverage parsing not supported for {:?}",
                other
            )),
        }
    }

    fn parse_cpp_report(content: &str, whitelist: &[String]) -> Result<String, String> {
        let re_file = Regex::new(r"File '([^']+)'").unwrap();
        let re_lines = Regex::new(r"Lines executed:([0-9.]+)% of (\d+)").unwrap();

        let mut files = Vec::new();
        let mut total_lines: u64 = 0;
        let mut total_covered: u64 = 0;
        let mut current_file: Option<String> = None;

        for line in content.lines() {
            if let Some(cap) = re_file.captures(line) {
                current_file = Some(cap[1].to_string());
            } else if let Some(cap) = re_lines.captures(line) {
                if let Some(file) = &current_file {
                    // Only include file if it's in the whitelist or whitelist is empty
                    if whitelist.is_empty() || whitelist.contains(file) {
                        let percent: f64 = cap[1].parse().unwrap_or(0.0);
                        let lines: u64 = cap[2].parse().unwrap_or(0);
                        let covered = ((percent / 100.0) * (lines as f64)).round() as u64;

                        total_lines += lines;
                        total_covered += covered;

                        files.push(CoverageFile {
                            path: file.clone(),
                            total_lines: lines,
                            covered_lines: covered,
                            coverage_percent: percent,
                        });
                    }
                    current_file = None;
                }
            }
        }

        let summary = CoverageSummary {
            total_files: files.len() as u64,
            total_lines,
            covered_lines: total_covered,
            coverage_percent: if total_lines > 0 {
                (total_covered as f64 / total_lines as f64) * 100.0
            } else {
                0.0
            },
        };

        let report = CoverageReport {
            generated_at: Utc::now().to_rfc3339(),
            summary,
            files,
        };

        serde_json::to_string_pretty(&report)
            .map_err(|e| format!("Failed to serialize coverage report: {}", e))
    }

    fn parse_java_report(content: &str, whitelist: &[String]) -> Result<String, String> {
        let mut files = Vec::new();
        let mut total_lines: u64 = 0;
        let mut total_covered: u64 = 0;

        for line in content.lines() {
            if line.starts_with("GROUP,") || line.trim().is_empty() {
                continue;
            }

            let cols: Vec<&str> = line.split(',').collect();
            if cols.len() < 9 {
                continue;
            }

            let class_name = cols[2].trim().to_string();
            let file_name = format!("{}.java", class_name);

            if !whitelist.is_empty() && !whitelist.contains(&file_name) {
                continue;
            }

            let line_missed: u64 = cols[7].trim().parse().unwrap_or(0);
            let line_covered: u64 = cols[8].trim().parse().unwrap_or(0);
            let total = line_missed + line_covered;
            let percent = if total > 0 {
                (line_covered as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            total_lines += total;
            total_covered += line_covered;

            files.push(CoverageFile {
                path: file_name, // now includes .java
                total_lines: total,
                covered_lines: line_covered,
                coverage_percent: percent,
            });
        }

        let summary = CoverageSummary {
            total_files: files.len() as u64,
            total_lines,
            covered_lines: total_covered,
            coverage_percent: if total_lines > 0 {
                (total_covered as f64 / total_lines as f64) * 100.0
            } else {
                0.0
            },
        };

        let report = CoverageReport {
            generated_at: Utc::now().to_rfc3339(),
            summary,
            files,
        };

        serde_json::to_string_pretty(&report)
            .map_err(|e| format!("Failed to serialize Java coverage report: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_process_cpp_report_and_print() {
        let content = "dummy content here";

        match CoverageProcessor::process_report(Language::Cpp, content, &Vec::new()) {
            Ok(json) => {
                println!("{}", json);
            }
            Err(e) => {
                panic!("Processing failed: {}", e);
            }
        }
    }
}
