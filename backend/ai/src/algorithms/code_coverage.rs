use crate::utils::output::Output;
use sea_orm::DatabaseConnection;
use serde_json::Value;

pub async fn coverage_percent_for_attempt(
    db: &DatabaseConnection,
    module_id: i64,
    assignment_id: i64,
    user_id: i64,
    attempt_number: i64,
) -> Result<f64, String> {
    let cov: Vec<(i64, String)> = Output::get_submission_output_code_coverage(
        db,
        module_id,
        assignment_id,
        user_id,
        attempt_number,
    )
    .await
    .map_err(|e| e.to_string())?;

    if let Some((_task_id, json)) = cov.first() {
        let pct = coverage_percent_from_json(json)?;
        Ok(pct.clamp(0.0, 100.0)) // optional clamp here
    } else {
        Ok(0.0)
    }
}

#[inline]
pub fn coverage_fitness(percent: f64) -> f64 {
    (percent / 100.0).clamp(0.0, 1.0)
}

pub fn coverage_percent_from_json(json: &str) -> Result<f64, String> {
    let v: Value =
        serde_json::from_str(json).map_err(|e| format!("Failed to parse coverage JSON: {}", e))?;
    if let Some(n) = v.get("summary").and_then(|s| s.get("coverage_percent")) {
        if let Some(x) = n.as_f64() {
            return Ok(x);
        }
        if let Some(s) = n.as_str() {
            if let Ok(x) = s.parse::<f64>() {
                return Ok(x);
            }
        }
    }
    Ok(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_fitness_bounds_and_scale() {
        assert_eq!(coverage_fitness(0.0), 0.0);
        assert_eq!(coverage_fitness(50.0), 0.5);
        assert_eq!(coverage_fitness(100.0), 1.0);
        // clamp beyond range
        assert_eq!(coverage_fitness(-10.0), 0.0);
        assert_eq!(coverage_fitness(150.0), 1.0);
    }

    #[test]
    fn parse_json_happy_path() {
        let json = r#"{ "summary": { "coverage_percent": 73.25 } }"#;
        let v = coverage_percent_from_json(json).unwrap();
        assert!((v - 73.25).abs() < 1e-9);
    }

    #[test]
    fn parse_json_missing_returns_zero() {
        let json1 = r#"{}"#;
        let json2 = r#"{ "summary": {} }"#;
        let json3 = r#"{ "summary": { "coverage_percent": null } }"#;
        assert_eq!(coverage_percent_from_json(json1).unwrap(), 0.0);
        assert_eq!(coverage_percent_from_json(json2).unwrap(), 0.0);
        assert_eq!(coverage_percent_from_json(json3).unwrap(), 0.0);
    }

    #[test]
    fn parse_json_garbage_errors() {
        let bad = "not-json at all";
        let err = coverage_percent_from_json(bad).unwrap_err();
        assert!(err.contains("Failed to parse coverage JSON"));
    }
}
