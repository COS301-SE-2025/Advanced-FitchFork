pub mod scorer;
pub mod feedback;
pub mod report;
pub mod error;
pub mod traits;
pub mod parsers;
pub mod utilities;
pub mod types;
pub mod comparators;

use std::path::PathBuf;
use crate::comparators::percentage_comparator::PercentageComparator;
use crate::error::MarkerError;
use crate::traits::parser::Parser;
use crate::utilities::file_loader::load_files;
use crate::traits::feedback::Feedback;
use crate::traits::comparator::OutputComparator;
use crate::report::MarkReportResponse;
use crate::scorer::compute_overall_score;
use crate::feedback::auto_feedback::AutoFeedback;
use crate::report::generate_mark_report;
use crate::types::{AllocatorSchema, TaskResult};
use crate::parsers::coverage_parser::CoverageReport;
use crate::parsers::complexity_parser::ComplexityReport;

pub struct MarkingJob<'a> {
    submission_id: &'a str,
    memo_outputs: Vec<PathBuf>,
    student_outputs: Vec<PathBuf>,
    allocation_json: PathBuf,
    coverage_report: Option<PathBuf>,
    complexity_report: Option<PathBuf>,
    comparator: Box<dyn OutputComparator + 'a>,
}

impl<'a> MarkingJob<'a> {
    pub fn new(
        submission_id: &'a str,
        memo_outputs: Vec<PathBuf>,
        student_outputs: Vec<PathBuf>,
        allocation_json: PathBuf,
    ) -> Self {
        Self {
            submission_id,
            memo_outputs,
            student_outputs,
            allocation_json,
            coverage_report: None,
            complexity_report: None,
            comparator: Box::new(PercentageComparator),
        }
    }

    pub fn with_coverage(mut self, report: PathBuf) -> Self {
        self.coverage_report = Some(report);
        self
    }

    pub fn with_complexity(mut self, report: PathBuf) -> Self {
        self.complexity_report = Some(report);
        self
    }

    pub fn with_comparator<C: OutputComparator + 'a>(mut self, comparator: C) -> Self {
        self.comparator = Box::new(comparator);
        self
    }

    pub fn mark(self) -> Result<MarkReportResponse, MarkerError> {
        // 1. Load & validate all files
        let files = load_files(
            self.submission_id,
            self.memo_outputs,
            self.student_outputs,
            self.allocation_json,
            self.coverage_report,
            self.complexity_report,
        )?;

        // 2. Parse JSON inputs
        let allocator: AllocatorSchema = parsers::allocator_parser::JsonAllocatorParser.parse(&files.allocator_raw)?;

        if let Some(coverage_raw) = files.coverage_raw {
            let _coverage: CoverageReport = parsers::coverage_parser::JsonCoverageParser.parse(&coverage_raw)?;
        }
        
        if let Some(complexity_raw) = files.complexity_raw {
            let _complexity: ComplexityReport = parsers::complexity_parser::JsonComplexityParser.parse(&complexity_raw)?;
        }

        // 3. Parse memo/student outputs into tasks & subtasks
        let submission = parsers::output_parser::OutputParser.parse((
            &files.memo_contents,
            &files.student_contents,
            allocator
                .0
                .iter()
                .map(|task| task.subsections.len())
                .collect(),
        ))?;

        // 4. For each task & each subsection, invoke the comparator strategy
        let mut all_results: Vec<TaskResult> = Vec::new();

        for task_entry in &allocator.0 {
            let submission_task = submission.tasks.iter().find(|t| t.task_id.eq_ignore_ascii_case(&task_entry.id));

            if let Some(task_output) = submission_task {
                for (i, subsection) in task_entry.subsections.iter().enumerate() {
                    if i >= task_output.memo_output.subtasks.len() {
                        return Err(MarkerError::InputMismatch(format!(
                            "Task '{}' has more subsections in allocator than in memo output",
                            task_entry.id
                        )));
                    }

                    let memo_lines = &task_output.memo_output.subtasks[i].lines;
                    let student_lines = &task_output.student_output.subtasks[i].lines;

                    let result: TaskResult = self.comparator.compare(
                        subsection,
                        memo_lines,
                        student_lines,
                    );
                    all_results.push(result);
                }
            } else {
                return Err(MarkerError::InputMismatch(format!(
                    "Task '{}' from allocator not found in submission outputs",
                    task_entry.id
                )));
            }
        }

        // 5. Compute overall score
        let overall = compute_overall_score(&all_results)?;

        // 6. Assemble feedback
        let feedback_entries = AutoFeedback
            .assemble_feedback(&all_results)?;

        // 7. Build the final MarkReport
        let report = generate_mark_report(
            self.submission_id,
            all_results,
            overall,
            feedback_entries,
        );

        // 8. Wrap and return the API response
        Ok(report.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::comparators::exact_comparator::ExactComparator;

    #[test]
    fn test_marker_happy_path() {
        let dir = "src/test_files/marker/case1";
        let memo_outputs = vec![PathBuf::from(format!("{}/memo1.txt", dir))];
        let student_outputs = vec![PathBuf::from(format!("{}/student1.txt", dir))];
        let allocation_json = PathBuf::from(format!("{}/allocator.json", dir));

        let job = MarkingJob::new(
            "test_submission",
            memo_outputs,
            student_outputs,
            allocation_json,
        );

        let result = job.mark();
        assert!(result.is_ok());

        let response = result.unwrap();
        println!("test_marker_happy_path:\n{}", serde_json::to_string_pretty(&response).unwrap());
        assert!(response.success);
        assert_eq!(response.data.submission_id, "test_submission");
        assert_eq!(response.data.overall_score, 100);
    }

    #[test]
    fn test_marker_happy_path_case2() {
        let dir = "src/test_files/marker/case2";
        let memo_outputs = vec![
            PathBuf::from(format!("{}/memo1.txt", dir)),
            PathBuf::from(format!("{}/memo2.txt", dir)),
        ];
        let student_outputs = vec![
            PathBuf::from(format!("{}/student1.txt", dir)),
            PathBuf::from(format!("{}/student2.txt", dir)),
        ];
        let allocation_json = PathBuf::from(format!("{}/allocator.json", dir));

        let job = MarkingJob::new(
            "test_submission_2",
            memo_outputs,
            student_outputs,
            allocation_json,
        );

        let result = job.mark();
        assert!(result.is_ok());

        let response = result.unwrap();
        println!("test_marker_happy_path_case2:\n{}", serde_json::to_string_pretty(&response).unwrap());
        assert!(response.success);
        assert_eq!(response.data.submission_id, "test_submission_2");
        assert_eq!(response.data.overall_score, 67);
    }

    #[test]
    fn test_marker_with_explicit_comparator() {
        let dir = "src/test_files/marker/case2";
        let memo_outputs = vec![
            PathBuf::from(format!("{}/memo1.txt", dir)),
            PathBuf::from(format!("{}/memo2.txt", dir)),
        ];
        let student_outputs = vec![
            PathBuf::from(format!("{}/student1.txt", dir)),
            PathBuf::from(format!("{}/student2.txt", dir)),
        ];
        let allocation_json = PathBuf::from(format!("{}/allocator.json", dir));

        let job = MarkingJob::new(
            "test_submission_3",
            memo_outputs,
            student_outputs,
            allocation_json,
        )
        .with_comparator(ExactComparator);

        let result = job.mark();
        assert!(result.is_ok());

        let response = result.unwrap();
        println!("test_marker_with_explicit_comparator:\n{}", serde_json::to_string_pretty(&response).unwrap());
        assert!(response.success);
        assert_eq!(response.data.submission_id, "test_submission_3");
        assert_eq!(response.data.overall_score, 67);
    }
} 