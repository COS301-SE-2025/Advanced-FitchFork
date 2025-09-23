use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ValgrindTask {
    pub task_number: i64,
    pub leaked: bool,
    pub bytes_leaked: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValgrindReport {
    pub generated_at: String,
    pub total_tasks: usize,
    pub total_leaks: u64,
    pub tasks: Vec<ValgrindTask>,
}

pub struct ValgrindProcessor;

impl ValgrindProcessor {
    /// Takes an array of full task outputs, each associated with a task number
    pub fn process_report(task_contents: &[(i64, String)]) -> Result<String, String> {
        let mut tasks: Vec<ValgrindTask> = Vec::new();
        let mut total_leaks: u64 = 0;

        let re_definitely_lost =
            Regex::new(r"definitely lost:\s*([0-9,]+)\s*bytes").map_err(|e| e.to_string())?;

        for (task_number, content) in task_contents.iter() {
            let mut leaked_bytes: u64 = 0;

            for cap in re_definitely_lost.captures_iter(content) {
                let s = cap[1].replace(',', "");
                if let Ok(val) = s.parse::<u64>() {
                    leaked_bytes = val;
                    break; // take the first match
                }
            }

            total_leaks += leaked_bytes;

            tasks.push(ValgrindTask {
                task_number: *task_number,
                leaked: leaked_bytes > 0,
                bytes_leaked: leaked_bytes,
            });
        }

        let report = ValgrindReport {
            generated_at: Utc::now().to_rfc3339(),
            total_tasks: tasks.len(),
            total_leaks,
            tasks,
        };

        serde_json::to_string_pretty(&report)
            .map_err(|e| format!("Failed to serialize Valgrind report: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valgrind_processor() {
        let valgrind_outputs = vec![
            (
                2,
                r#"
                g++ -fprofile-arcs -ftest-coverage -O0 -std=c++17 -c Main.cpp -o Main.o
g++ -fprofile-arcs -ftest-coverage -O0 -std=c++17 -c HelperOne.cpp -o HelperOne.o
g++ -fprofile-arcs -ftest-coverage -O0 -std=c++17 -c HelperTwo.cpp -o HelperTwo.o
g++ -fprofile-arcs -ftest-coverage -O0 -std=c++17 -c HelperThree.cpp -o HelperThree.o
g++ -fprofile-arcs -ftest-coverage -O0 -std=c++17 Main.o HelperOne.o HelperTwo.o HelperThree.o -lgcov -o main
valgrind --leak-check=full ./main task3
&-=-&Task3Subtask1
HelperThree: Subtask for Task3
This as well
&-=-&Task3Subtask2
HelperOne: Subtask for Task3
This as well
And this
&-=-&Task3Subtask3

&FITCHFORK&StandardError

==22== Memcheck, a memory error detector
==22== Copyright (C) 2002-2024, and GNU GPL'd, by Julian Seward et al.
==22== Using Valgrind-3.23.0 and LibVEX; rerun with -h for copyright info
==22== Command: ./main task3
==22== 
==22== 
==22== HEAP SUMMARY:
==22==     in use at exit: 100 bytes in 1 blocks
==22==   total heap usage: 13 allocs, 12 frees, 79,320 bytes allocated
==22== 
==22== 100 bytes in 1 blocks are definitely lost in loss record 1 of 1
==22==    at 0x48AEF8F: operator new[](unsigned long) (in /usr/libexec/valgrind/vgpreload_memcheck-amd64-linux.so)
==22==    by 0x10D44F: HelperThree::subtaskAlpha[abi:cxx11]() (in /code/main)
==22==    by 0x10BBBA: runTask3() (in /code/main)
==22==    by 0x10C1A2: main (in /code/main)
==22== 
==22== LEAK SUMMARY:
==22==    definitely lost: 100 bytes in 1 blocks
==22==    indirectly lost: 0 bytes in 0 blocks
==22==      possibly lost: 0 bytes in 0 blocks
==22==    still reachable: 0 bytes in 0 blocks
==22==         suppressed: 0 bytes in 0 blocks
==22== 
==22== For lists of detected and suppressed errors, rerun with: -s
==22== ERROR SUMMARY: 1 errors from 1 contexts (suppressed: 0 from 0)
&FITCHFORK&ReturnCode

Retcode: 0
                "#
                    .to_string(),
            ),
            (
                3,
                r#"
                g++ -fprofile-arcs -ftest-coverage -O0 -std=c++17 -c Main.cpp -o Main.o
g++ -fprofile-arcs -ftest-coverage -O0 -std=c++17 -c HelperOne.cpp -o HelperOne.o
g++ -fprofile-arcs -ftest-coverage -O0 -std=c++17 -c HelperTwo.cpp -o HelperTwo.o
g++ -fprofile-arcs -ftest-coverage -O0 -std=c++17 -c HelperThree.cpp -o HelperThree.o
g++ -fprofile-arcs -ftest-coverage -O0 -std=c++17 Main.o HelperOne.o HelperTwo.o HelperThree.o -lgcov -o main
valgrind --leak-check=full ./main task2
&-=-&Task2Subtask1
HelperTwo: Subtask for Task2
This as well
And this
&-=-&Task2Subtask2
HelperThree: Subtask for Task2
This as well
And this
Additional wrong line
&-=-&Task2Subtask3
HelperOne: Subtask for Task2
This as well
And this
&FITCHFORK&StandardError

==22== Memcheck, a memory error detector
==22== Copyright (C) 2002-2024, and GNU GPL'd, by Julian Seward et al.
==22== Using Valgrind-3.23.0 and LibVEX; rerun with -h for copyright info
==22== Command: ./main task2
==22== 
==22== 
==22== HEAP SUMMARY:
==22==     in use at exit: 0 bytes in 0 blocks
==22==   total heap usage: 13 allocs, 13 frees, 79,302 bytes allocated
==22== 
==22== All heap blocks were freed -- no leaks are possible
==22== 
==22== For lists of detected and suppressed errors, rerun with: -s
==22== ERROR SUMMARY: 0 errors from 0 contexts (suppressed: 0 from 0)
&FITCHFORK&ReturnCode

Retcode: 0
                "#
                    .to_string(),
            ),
        ];

        let json_report = ValgrindProcessor::process_report(&valgrind_outputs)
            .expect("Failed to process Valgrind outputs");

        let report: ValgrindReport =
            serde_json::from_str(&json_report).expect("Failed to parse JSON");

        assert_eq!(report.total_tasks, 2);
        assert_eq!(report.total_leaks, 100);

        let task2 = &report.tasks[0];
        assert_eq!(task2.task_number, 2);
        assert!(task2.leaked);
        assert_eq!(task2.bytes_leaked, 100);

        let task3 = &report.tasks[1];
        assert_eq!(task3.task_number, 3);
        assert!(!task3.leaked);
        assert_eq!(task3.bytes_leaked, 0);
    }
}
