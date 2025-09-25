use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf};

use crate::execution_config::{ExecutionConfig, MarkingScheme};
use crate::paths::{mark_allocator_dir, mark_allocator_path};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarkAllocator {
    pub generated_at: DateTime<Utc>,
    pub tasks: Vec<Task>,
    pub total_value: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub task_number: i64,
    pub name: String,
    pub value: i64,
    #[serde(default)]
    pub code_coverage: Option<bool>,
    pub valgrind: Option<bool>,
    pub subsections: Vec<Subsection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subsection {
    pub name: String,
    pub value: i64,
    #[serde(default)]
    pub regex: Option<Vec<String>>,
    #[serde(default)]
    pub feedback: Option<String>,
}

impl MarkAllocator {
    pub fn recompute_total(&mut self) -> i64 {
        self.total_value = self.tasks.iter().map(|t| t.value).sum();
        self.total_value
    }
    pub fn new_now(tasks: Vec<Task>) -> Self {
        let mut me = MarkAllocator {
            generated_at: Utc::now(),
            total_value: tasks.iter().map(|t| t.value).sum(),
            tasks,
        };
        me.recompute_total();
        me
    }
}

/// Read allocator.json as **normalized**.
pub fn load_allocator(module_id: i64, assignment_id: i64) -> Result<MarkAllocator, String> {
    use std::io::ErrorKind;

    let path = mark_allocator_path(module_id, assignment_id);

    // Short, standardized I/O errors
    let s = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            let msg = match e.kind() {
                ErrorKind::NotFound => "File not found".to_string(),
                ErrorKind::PermissionDenied => {
                    "Permission denied reading mark allocator".to_string()
                }
                ErrorKind::InvalidData => "Allocator file is not valid UTF-8".to_string(),
                _ => format!("Failed to read mark allocator ({})", e.kind()),
            };
            return Err(msg);
        }
    };

    // Short parse error
    serde_json::from_str::<MarkAllocator>(&s)
        .map_err(|_| "Invalid allocator JSON (normalized expected)".to_string())
}

/// Save allocator.json as **normalized** (atomic-ish write).
pub fn save_allocator(
    module_id: i64,
    assignment_id: i64,
    alloc: &MarkAllocator,
) -> Result<(), String> {
    use std::io::ErrorKind;

    let dir = mark_allocator_dir(module_id, assignment_id);
    fs::create_dir_all(&dir).map_err(|e| match e.kind() {
        ErrorKind::PermissionDenied => "Permission denied creating allocator directory".to_string(),
        _ => "Failed to prepare allocator directory".to_string(),
    })?;

    let path = mark_allocator_path(module_id, assignment_id);
    let pretty = serde_json::to_string_pretty(alloc)
        .map_err(|_| "Failed to serialize allocator".to_string())?;

    let tmp = temp_path(&path);
    {
        let mut f = fs::File::create(&tmp).map_err(|e| match e.kind() {
            ErrorKind::PermissionDenied => "Permission denied creating temp file".to_string(),
            _ => "Failed to create temp file".to_string(),
        })?;
        f.write_all(pretty.as_bytes())
            .map_err(|_| "Failed to write temp file".to_string())?;
        f.flush()
            .map_err(|_| "Failed to flush temp file".to_string())?;
    }
    fs::rename(&tmp, &path).map_err(|_| "Failed to move temp file into place".to_string())?;
    Ok(())
}

fn temp_path(final_path: &PathBuf) -> PathBuf {
    let mut tmp = final_path.clone();
    let fname = final_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("allocator.json");
    tmp.set_file_name(format!("{fname}.tmp"));
    tmp
}

// Carry-over type used by generators
#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub id: i64,
    pub task_number: i64,
    pub code_coverage: bool,
    pub valgrind: bool,
    pub name: String,
}

/// Generator: builds a **normalized** allocator (regex arrays only when scheme=Regex).
pub async fn generate_allocator(
    module_id: i64,
    assignment_id: i64,
    tasks_info: &[(TaskInfo, PathBuf)],
) -> Result<MarkAllocator, String> {
    let default_valgrind_mark_value = 5;

    // Read config once up-front
    let (separator, want_regex, cover_weight_frac) =
        match ExecutionConfig::get_execution_config(module_id, assignment_id) {
            Ok(cfg) => {
                let want_regex = matches!(cfg.marking.marking_scheme, MarkingScheme::Regex);

                // Normalize weight: treat >= 1.0 as percent; otherwise assume fraction.
                // e.g. 10.0 -> 0.10; 0.10 -> 0.10
                let mut w = cfg.code_coverage.code_coverage_weight as f64;
                if w >= 1.0 {
                    w /= 100.0;
                }
                // Clamp to sane range [0.0, 0.95] to avoid division blow-ups (you can choose a different cap)
                let w = w.clamp(0.0, 0.95);

                (cfg.marking.deliminator, want_regex, w)
            }
            Err(_) => ("&-=-&".to_string(), false, 0.10), // fallback: 10%
        };

    let dir = mark_allocator_dir(module_id, assignment_id);
    fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create allocator dir {:?}: {}", dir, e))?;

    // -------- Pass 1: build all tasks (with parsed values for non-coverage) --------
    let mut tasks: Vec<Task> = Vec::with_capacity(tasks_info.len());
    let mut base_total: i64 = 0; // sum of non-coverage tasks

    let mut coverage_indices: Vec<usize> = Vec::new();

    for (idx, (info, maybe_path)) in tasks_info.iter().enumerate() {
        let mut subsections = Vec::new();
        let mut task_value: i64 = 0;

        if !info.code_coverage && maybe_path.exists() {
            // Normal subsection parsing
            let content = fs::read_to_string(maybe_path)
                .map_err(|e| format!("Failed reading {:?}: {}", maybe_path, e))?;

            let mut current_section = String::new();
            let mut mark_counter: i64 = 0;

            let mut section_counter = 0;

            for line in content.lines() {
                let split: Vec<_> = line.split(&separator).collect();
                if split.len() > 1 {
                    if !current_section.is_empty() {
                        subsections.push(Subsection {
                            name: current_section.clone(),
                            value: mark_counter,
                            regex: if want_regex {
                                Some(vec![String::new(); mark_counter.max(0) as usize])
                            } else {
                                None
                            },
                            feedback: None,
                        });
                        task_value += mark_counter;
                    }
                    section_counter += 1;
                    let name = split.last().unwrap().trim();
                    current_section = if name.is_empty() {
                        format!("Section {}", section_counter)
                    } else {
                        name.to_string()
                    };
                    mark_counter = 0;
                } else if !line.trim().is_empty() {
                    mark_counter += 1;
                }
            }

            if !current_section.is_empty() {
                subsections.push(Subsection {
                    name: current_section,
                    value: mark_counter,
                    regex: if want_regex {
                        Some(
                            std::iter::repeat(String::new())
                                .take(mark_counter.max(0) as usize)
                                .collect(),
                        )
                    } else {
                        None
                    },
                    feedback: None,
                });
                task_value += mark_counter;
            }

            if info.valgrind {
                task_value += default_valgrind_mark_value;
            }

            base_total += task_value;
        } else if info.code_coverage {
            coverage_indices.push(idx);
        }

        // --- Append valgrind-specific subsection if this is a valgrind task ---
        if info.valgrind {
            subsections.push(Subsection {
                name: "Memory Leaks".to_string(),
                value: default_valgrind_mark_value,
                regex: None,
                feedback: Some("Check for memory leaks with Valgrind".to_string()),
            });
        }

        tasks.push(Task {
            task_number: info.task_number,
            name: if info.name.trim().is_empty() {
                format!("Task {}", info.task_number)
            } else {
                info.name.clone()
            },
            value: task_value,
            code_coverage: Some(info.code_coverage),
            valgrind: Some(info.valgrind),
            subsections,
        });
    }

    // -------- Pass 2: assign coverage marks based on base_total & weight --------
    if !coverage_indices.is_empty() && cover_weight_frac > 0.0 {
        // C = round(B * w / (1 - w))
        let c = if cover_weight_frac >= 1.0 {
            0 // (shouldn’t happen due to clamp) – but keep safe
        } else {
            let c_f = (base_total as f64) * (cover_weight_frac / (1.0 - cover_weight_frac));
            c_f.round() as i64
        };

        let n = coverage_indices.len() as i64;
        let per = c.div_euclid(n.max(1)); // even split
        let rem = c.rem_euclid(n.max(1)); // distribute remainder +1 to first 'rem' tasks

        for (j, &ti) in coverage_indices.iter().enumerate() {
            let mut v = per;
            if (j as i64) < rem {
                v += 1;
            }
            // Ensure non-negative
            if v < 0 {
                v = 0;
            }
            if let Some(t) = tasks.get_mut(ti) {
                t.value = v;
                // Keep subsections empty for coverage tasks
                t.subsections.clear();
            }
        }
    } else {
        // No coverage tasks or weight == 0: nothing to do.
    }

    let mut alloc = MarkAllocator::new_now(tasks);
    alloc.recompute_total();
    Ok(alloc)
}
