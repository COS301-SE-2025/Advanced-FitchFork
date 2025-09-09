use crate::execution_config::execution_config::Language;
use chrono::Utc;
use regex::Regex;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct CoverageSummary {
    total_files: u64,
    total_lines: u64,
    covered_lines: u64,
    coverage_percent: f64,
}

#[derive(Debug, Serialize)]
pub struct CoverageFile {
    path: String,
    total_lines: u64,
    covered_lines: u64,
    coverage_percent: f64,
}

#[derive(Debug, Serialize)]
pub struct CoverageReport {
    generated_at: String,
    summary: CoverageSummary,
    files: Vec<CoverageFile>,
}

pub struct CoverageProcessor;

impl CoverageProcessor {
    pub fn process_report(language: Language, report_path: &Path) -> Result<String, String> {
        match language {
            Language::Cpp => Self::parse_cpp_report(report_path),
            Language::Java => Self::parse_java_report(report_path),
        }
    }

    fn parse_cpp_report(report_path: &Path) -> Result<String, String> {
        let content = fs::read_to_string(report_path)
            .map_err(|e| format!("Failed to read report file: {}", e))?;

        let re_file = Regex::new(r"File '([^']+)'").unwrap();
        let re_lines = Regex::new(r"Lines executed:([0-9.]+)% of (\d+)").unwrap();

        let mut files = Vec::new();
        let mut total_lines = 0;
        let mut total_covered = 0;
        let mut current_file: Option<String> = None;

        for line in content.lines() {
            if let Some(cap) = re_file.captures(line) {
                current_file = Some(cap[1].to_string());
            } else if let Some(cap) = re_lines.captures(line) {
                if let Some(file) = &current_file {
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

    fn parse_java_report(_report_path: &Path) -> Result<String, String> {
        Err("Java code coverage parsing not yet implemented".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    #[ignore]
    fn test_process_cpp_report_and_print() {
        // change this to point to your local gcov text report
        let report_path = Path::new("absolue_path_here_if_you_want_to_test");

        match CoverageProcessor::process_report(Language::Cpp, report_path) {
            Ok(json) => {
                println!("{}", json);
            }
            Err(e) => {
                panic!("Processing failed: {}", e);
            }
        }
    }
}
