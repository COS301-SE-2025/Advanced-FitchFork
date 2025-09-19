use validator::ValidationErrors;

pub fn format_validation_errors(errors: &ValidationErrors) -> String {
    errors
        .field_errors()
        .values()
        .flat_map(|errs| {
            errs.iter()
                .filter_map(|e| e.message.as_ref().map(|m| m.to_string()))
        })
        .collect::<Vec<_>>()
        .join("; ")
}
