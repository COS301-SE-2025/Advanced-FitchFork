use std::fs::{self, File};
use std::io::{Error, Write};

use serde_json::{Value, from_str, json};
#[allow(dead_code)]
const SEPARATOR: &str = "&-=-& ";

pub async fn generate_allocator(module: i64, assignment: i64) -> Value {
    let path = format!(
        "../data/assignment_files/module_{}/assignment_{}/memo_output/",
        module, assignment
    );
    let allocator_path = format!(
        "../data/assignment_files/module_{}/assignment_{}/mark_allocator/",
        module, assignment
    );

    let paths = fs::read_dir(&path).unwrap();
    let mut task_index = 1;
    let mut tasks_json: Vec<Value> = vec![];

    for p in paths {
        let file_path = format!("{}{}", &path, p.unwrap().file_name().into_string().unwrap());
        let mut subsections = vec![];
        let mut current_section = String::new();
        let mut mark_counter = 0;
        let mut task_value = 0;

        for line in fs::read_to_string(&file_path).unwrap().lines() {
            let split: Vec<_> = line.split(SEPARATOR).collect();

            if split.len() > 1 {
                if !current_section.is_empty() {
                    subsections.push(json!({
                        "name": current_section,
                        "value": mark_counter
                    }));
                    task_value += mark_counter;
                }
                current_section = split.last().unwrap().trim().to_string();
                mark_counter = 0;
            } else if !line.trim().is_empty() {
                mark_counter += 1;
            }
        }

        if !current_section.is_empty() {
            subsections.push(json!({
                "name": current_section,
                "value": mark_counter
            }));
            task_value += mark_counter;
        }

        let task_name = format!("task{}", task_index);
        let task_entry = json!({
            task_name: {
                "name": format!("Task {}", task_index),
                "value": task_value,
                "subsections": subsections
            }
        });

        tasks_json.push(task_entry);
        task_index += 1;
    }

    let final_json = json!({
        "tasks": tasks_json
    });

    fs::create_dir_all(&allocator_path).unwrap();

    let output_path = format!("{}allocator.json", allocator_path);
    let mut file = File::create(&output_path).unwrap();
    write!(
        file,
        "{}",
        serde_json::to_string_pretty(&final_json).unwrap()
    )
    .unwrap();

    final_json
}

pub async fn load_allocator(module: i64, assignment: i64) -> Option<Value> {
    let path = format!(
        "../data/assignment_files/module_{}/assignment_{}/mark_allocator/allocator.json",
        module, assignment
    );

    match fs::read_to_string(&path) {
        Ok(json_str) => match from_str::<Value>(&json_str) {
            Ok(json_value) => Some(json_value),
            Err(e) => {
                eprintln!("Failed to parse JSON: {}", e);
                None
            }
        },
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
            None
        }
    }
}

pub async fn save_allocator(module: i64, assignment: i64, json: Value) -> Result<(), Error> {
    let allocator_path = format!(
        "../data/assignment_files/module_{}/assignment_{}/mark_allocator/allocator.json",
        module, assignment
    );
    let mut file = File::open(allocator_path).unwrap();
    write!(file, "{}", serde_json::to_string_pretty(&json).unwrap())
}
