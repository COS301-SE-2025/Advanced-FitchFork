use crate::models::module::Module;

pub fn make_module(name: &str, description: Option<&str>) -> Module {
    Module {
        id: 0,
        name: name.to_string(),
        description: description.map(|d| d.to_string()),
        created_at: chrono::Utc::now().to_rfc3339(),
    }
}
