use sea_orm::DatabaseConnection;
use serde_json::Value;
use crate::utils::output::Output;

pub async fn coverage_percent_for_attempt(
    db: &DatabaseConnection,
    module_id: i64,
    assignment_id: i64,
    user_id: i64,
    attempt_number: i64,
) -> Result<f64, String> {
    let cov: Vec<(i64, String)> = Output::get_submission_output_code_coverage(
        db, module_id, assignment_id, user_id, attempt_number
    ).await.map_err(|e| e.to_string())?;

    if let Some((_task_id, json)) = cov.first() {
        let v: Value = serde_json::from_str(json)
            .map_err(|e| format!("Failed to parse coverage JSON: {}", e))?;
        Ok(v.get("summary")
            .and_then(|s| s.get("coverage_percent"))
            .and_then(|p| p.as_f64())
            .unwrap_or(0.0))
    } else {
        Ok(0.0)
    }
}

#[inline]
pub fn coverage_fitness(percent: f64) -> f64 {
    (percent / 100.0).clamp(0.0, 1.0)
}
