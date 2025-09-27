use std::collections::HashMap;

fn key(s: &str) -> String {
    s.trim_end().to_string()
}

pub fn reorder_student_by_memo(student: Vec<String>, memo: &[String]) -> Vec<String> {
    let mut need: HashMap<String, i32> = HashMap::new();
    for m in memo {
        *need.entry(key(m)).or_default() += 1;
    }
    let mut matched = Vec::with_capacity(student.len());
    let mut rest = Vec::new();

    for s in student {
        let k = key(&s);
        if let Some(cnt) = need.get_mut(&k) {
            if *cnt > 0 {
                matched.push(s);
                *cnt -= 1;
                continue;
            }
        }
        rest.push(s);
    }
    matched.into_iter().chain(rest.into_iter()).collect()
}
