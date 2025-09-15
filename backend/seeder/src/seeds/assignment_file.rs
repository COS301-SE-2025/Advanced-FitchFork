use crate::seed::Seeder;
use services::service::{Service, AppError};
use services::assignment::AssignmentService;
use services::assignment_file::{AssignmentFileService, CreateAssignmentFile};
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use std::pin::Pin;

pub struct AssignmentFileSeeder;

impl Seeder for AssignmentFileSeeder {
    fn seed<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            let assignments = AssignmentService::find_all(
                &vec![],
                &vec![],
                None,
            ).await?;

            let file_types: Vec<(&str, fn(i32) -> String)> = vec![
                ("spec", |id| format!("spec_{}.txt", id)),
                ("memo", |id| format!("memo_{}.txt", id)),
                ("main", |id| format!("main_{}.txt", id)),
                ("makefile", |id| format!("makefile_{}.txt", id)),
                ("mark_allocator", |id| {
                    format!("mark_allocator_{}.txt", id)
                }),
                ("config", |id| format!("config_{}.txt", id)),
            ];

            for a in &assignments {
                if a.module_id == 9999 || a.module_id == 9998 || a.module_id == 10003 {
                    continue;
                }

                for &(ref file_type, filename_fn) in &file_types {
                    let filename = filename_fn(a.id.try_into().unwrap());
                    let content = format!("This is the content of assignment file {}", a.id);

                    AssignmentFileService::create(
                        CreateAssignmentFile{
                            assignment_id: a.id,
                            module_id: a.module_id,
                            file_type: file_type.to_string(),
                            filename: filename,
                            bytes: content.as_bytes().to_vec(),
                        }
                    ).await?;
                }
            }

            let special_module_id: i64 = 9999;
            let special_assignment_id: i64 = 9999;

            fn create_main_zip() -> Vec<u8> {
                let mut buf = Cursor::new(Vec::new());
                {
                    let mut zip = zip::ZipWriter::new(&mut buf);
                    let options = SimpleFileOptions::default().unix_permissions(0o644);

                    let main_java = r#"
        public class Main {
            public static void main(String[] args) {
                String task = args.length > 0 ? args[0] : "task1";

                switch (task) {
                    case "task1":
                        runTask1();
                        break;
                    case "task2":
                        runTask2();
                        break;
                    case "task3":
                        runTask3();
                        break;
                    default:
                        System.out.println("" + task + " is not a valid task");
                }
            }

            static void runTask1() {
                System.out.println("" + "&-=-&Task1Subtask1");
                System.out.println(HelperOne.subtaskA());
                System.out.println("&-=-&Task1Subtask2");
                System.out.println(HelperTwo.subtaskB());
                System.out.println("&-=-&Task1Subtask3");
                System.out.println(HelperThree.subtaskC());
            }

            static void runTask2() {
                System.out.println("&-=-&Task2Subtask1");
                System.out.println(HelperTwo.subtaskX());
                System.out.println("&-=-&Task2Subtask2");
                System.out.println(HelperThree.subtaskY());
                System.out.println("&-=-&Task2Subtask3");
                System.out.println(HelperOne.subtaskZ());
            }

        static void runTask3() {
            System.out.println("&-=-&Task3Subtask1");
            System.out.println(HelperThree.subtaskAlpha())
            System.out.println("&-=-&Task3Subtask2");
            System.out.println(HelperOne.subtaskBeta());
            System.out.println("&-=-&Task3Subtask3");
            System.out.println(HelperTwo.subtaskGamma());
        }
    }
    "#;

                    zip.start_file("Main.java", options).unwrap();
                    zip.write_all(main_java.as_bytes()).unwrap();
                    zip.finish().unwrap();
                }
                buf.into_inner()
            }

            fn create_memo_zip() -> Vec<u8> {
                let mut buf = Cursor::new(Vec::new());
                {
                    let mut zip = zip::ZipWriter::new(&mut buf);
                    let options = SimpleFileOptions::default().unix_permissions(0o644);

                    let helper_one = r#"
        public class HelperOne {
            public static String subtaskA() {
                return "" + "HelperOne: Subtask for Task1\nThis as well\nAnd this";
            }
            public static String subtaskZ() {
                return "HelperOne: Subtask for Task2\nThis as well\nAnd this";
            }
            public static String subtaskBeta() {
                return "HelperOne: Subtask for Task3\nThis as well\nAnd this";
            }
        }
        "#;

                        let helper_two = r#"
        public class HelperTwo {
            public static String subtaskB() {
                return "HelperTwo: Subtask for Task1\nThis as well\nAnd this";
            }
            public static String subtaskX() {
                return "HelperTwo: Subtask for Task2\nThis as well\nAnd this";
            }
            public static String subtaskGamma() {
                return "HelperTwo: Subtask for Task3\nThis as well\nAnd this";
            }
        }
        "#;

                        let helper_three = r#"
        public class HelperThree {
            public static String subtaskC() {
                return "HelperThree: Subtask for Task1\nThis as well\nAnd this";
            }
            public static String subtaskY() {
                return "HelperThree: Subtask for Task2\nThis as well\nAnd this";
            }
            public static String subtaskAlpha() {
                return "HelperThree: Subtask for Task3\nThis as well\nAnd this";
            }
        }
        "#;

                    zip.start_file("HelperOne.java", options).unwrap();
                    zip.write_all(helper_one.as_bytes()).unwrap();

                    zip.start_file("HelperTwo.java", options).unwrap();
                    zip.write_all(helper_two.as_bytes()).unwrap();

                    zip.start_file("HelperThree.java", options).unwrap();
                    zip.write_all(helper_three.as_bytes()).unwrap();

                    zip.finish().unwrap();
                }
                buf.into_inner()
            }

            fn create_makefile_zip() -> Vec<u8> {
                let mut buf = Cursor::new(Vec::new());
                {
                    let mut zip = zip::ZipWriter::new(&mut buf);
                    let options = SimpleFileOptions::default().unix_permissions(0o644);

                    let makefile_content = r#"
        task1:
            javac -d /output Main.java HelperOne.java HelperTwo.java HelperThree.java && java -cp /output Main task1

        task2:
            javac -d /output Main.java HelperOne.java HelperTwo.java HelperThree.java && java -cp /output Main task2

        task3:
            javac -d /output Main.java HelperOne.java HelperTwo.java HelperThree.java && java -cp /output Main task3
        "#;

                    zip.start_file("Makefile", options).unwrap();
                    zip.write_all(makefile_content.as_bytes()).unwrap();
                    zip.finish().unwrap();
                }
                buf.into_inner()
            }

        // New config file content
        let config_json = r#"
    {
    "execution": {
        "timeout_secs": 10,
        "max_memory": 8589934592,
        "max_cpus": 2,
        "max_uncompressed_size": 100000000,
        "max_processes": 256
    },
    "marking": {
        "marking_scheme": "exact",
        "feedback_scheme": "auto",
        "deliminator": "&-=-&"
    },
    "project": {
        "language": "cpp"
    },
    "output": {
        "stdout": true,
        "stderr": true,
        "retcode": true
    }
    }
    "#;

            let zipped_files = vec![
                ("main", "main.zip", create_main_zip()),
                ("memo", "memo.zip", create_memo_zip()),
                ("makefile", "makefile.zip", create_makefile_zip()),
                (
                    "config",
                    "config.json",
                    config_json.as_bytes().to_vec(),
                ),
            ];

            for (file_type, filename, content) in zipped_files {
                AssignmentFileService::create(
                    CreateAssignmentFile{
                        assignment_id: special_assignment_id,
                        module_id: special_module_id,
                        file_type: file_type.to_string(),
                        filename: filename.to_string(),
                        bytes: content,
                    }
                ).await?;
            }

            let cpp_module_id = 9998;
            let cpp_assignment_id = 9998;

        //Original main file that was created
        fn create_main_zip_cpp() -> Vec<u8> {
            let mut buf = Cursor::new(Vec::new());
            {
                let mut zip = zip::ZipWriter::new(&mut buf);
                let options = SimpleFileOptions::default().unix_permissions(0o644);

                    let main_cpp = r#"
        #include <iostream>
        #include <string>
        #include "HelperOne.h"
        #include "HelperTwo.h"
        #include "HelperThree.h"

        void runTask1() {
            std::cout << "&-=-&Task1Subtask1\n" << HelperOne::subtaskA() << std::endl;
            std::cout << "&-=-&Task1Subtask2\n" << HelperTwo::subtaskB() << std::endl;
            std::cout << "&-=-&Task1Subtask3\n" << HelperThree::subtaskC() << std::endl;
        }

        void runTask2() {
            std::cout << "&-=-&Task2Subtask1\n" << HelperTwo::subtaskX() << std::endl;
            std::cout << "&-=-&Task2Subtask2\n" << HelperThree::subtaskY() << std::endl;
            std::cout << "&-=-&Task2Subtask3\n" << HelperOne::subtaskZ() << std::endl;
        }

        void runTask3() {
            std::cout << "&-=-&Task3Subtask1\n" << HelperThree::subtaskAlpha() << std::endl;
            std::cout << "&-=-&Task3Subtask2\n" << HelperOne::subtaskBeta() << std::endl;
            std::cout << "&-=-&Task3Subtask3\n" << HelperTwo::subtaskGamma() << std::endl;
        }

        int main(int argc, char* argv[]) {
            std::string task = argc > 1 ? argv[1] : "task1";

            if (task == "task1") runTask1();
            else if (task == "task2") runTask2();
            else if (task == "task3") runTask3();
            else std::cout << task << " is not a valid task" << std::endl;

            return 0;
        }
        "#;

                    zip.start_file("Main.cpp", options).unwrap();
                    zip.write_all(main_cpp.as_bytes()).unwrap();

                    zip.finish().unwrap();
                }
                buf.into_inner()
            }

            fn create_memo_zip_cpp() -> Vec<u8> {
                let mut buf = Cursor::new(Vec::new());
                {
                    let mut zip = zip::ZipWriter::new(&mut buf);
                    let options = SimpleFileOptions::default().unix_permissions(0o644);

                    let helper_one_cpp = r#"
        #include "HelperOne.h"
        std::string HelperOne::subtaskA() {
            return "HelperOne: Subtask for Task1\nThis as well\nAnd this";
        }
        std::string HelperOne::subtaskZ() {
            return "HelperOne: Subtask for Task2\nThis as well\nAnd this";
        }
        std::string HelperOne::subtaskBeta() {
            return "HelperOne: Subtask for Task3\nThis as well\nAnd this";
        }
        "#;

                        let helper_two_cpp = r#"
        #include "HelperTwo.h"
        std::string HelperTwo::subtaskB() {
            return "HelperTwo: Subtask for Task1\nThis as well\nAnd this";
        }
        std::string HelperTwo::subtaskX() {
            return "HelperTwo: Subtask for Task2\nThis as well\nAnd this";
        }
        std::string HelperTwo::subtaskGamma() {
            return "HelperTwo: Subtask for Task3\nThis as well\nAnd this";
        }
        "#;

                        let helper_three_cpp = r#"
        #include "HelperThree.h"
        std::string HelperThree::subtaskC() {
            return "HelperThree: Subtask for Task1\nThis as well\nAnd this";
        }
        std::string HelperThree::subtaskY() {
            return "HelperThree: Subtask for Task2\nThis as well\nAnd this";
        }
        std::string HelperThree::subtaskAlpha() {
            return "HelperThree: Subtask for Task3\nThis as well\nAnd this";
        }
        "#;

                        let helper_one_h = r#"
        #ifndef HELPERONE_H
        #define HELPERONE_H
        #include <string>
        struct HelperOne {
            static std::string subtaskA();
            static std::string subtaskZ();
            static std::string subtaskBeta();
        };
        #endif
        "#;

                        let helper_two_h = r#"
        #ifndef HELPERTWO_H
        #define HELPERTWO_H
        #include <string>
        struct HelperTwo {
            static std::string subtaskB();
            static std::string subtaskX();
            static std::string subtaskGamma();
        };
        #endif
        "#;

                        let helper_three_h = r#"
        #ifndef HELPERTHREE_H
        #define HELPERTHREE_H
        #include <string>
        struct HelperThree {
            static std::string subtaskC();
            static std::string subtaskY();
            static std::string subtaskAlpha();
        };
        #endif
        "#;

                    zip.start_file("HelperOne.cpp", options).unwrap();
                    zip.write_all(helper_one_cpp.as_bytes()).unwrap();

                    zip.start_file("HelperTwo.cpp", options).unwrap();
                    zip.write_all(helper_two_cpp.as_bytes()).unwrap();

                    zip.start_file("HelperThree.cpp", options).unwrap();
                    zip.write_all(helper_three_cpp.as_bytes()).unwrap();

                    zip.start_file("HelperOne.h", options).unwrap();
                    zip.write_all(helper_one_h.as_bytes()).unwrap();

                    zip.start_file("HelperTwo.h", options).unwrap();
                    zip.write_all(helper_two_h.as_bytes()).unwrap();

                    zip.start_file("HelperThree.h", options).unwrap();
                    zip.write_all(helper_three_h.as_bytes()).unwrap();

                    zip.finish().unwrap();
                }
                buf.into_inner()
            }

            fn create_makefile_zip_cpp() -> Vec<u8> {
                let mut buf = Cursor::new(Vec::new());
                {
                    let mut zip = zip::ZipWriter::new(&mut buf);
                    let options = SimpleFileOptions::default().unix_permissions(0o644);

                    let makefile_content = r#"
        CXX = g++
        CXXFLAGS = -fprofile-arcs -ftest-coverage -O0 -std=c++17
        LDFLAGS = -lgcov

        SRC = Main.cpp HelperOne.cpp HelperTwo.cpp HelperThree.cpp
        OBJ = Main.o HelperOne.o HelperTwo.o HelperThree.o

        OUTPUT = main

        main: $(OBJ)
            $(CXX) $(CXXFLAGS) $^ $(LDFLAGS) -o $@

        %.o: %.cpp
            $(CXX) $(CXXFLAGS) -c $< -o $@

        task1: main
            ./main task1

        task2: main
            ./main task2

        task3: main
            ./main task3

        task4: main
            ./main task1
            ./main task2
            ./main task3
            gcov $(SRC)
        "#;

                    zip.start_file("Makefile", options).unwrap();
                    zip.write_all(makefile_content.as_bytes()).unwrap();
                    zip.finish().unwrap();
                }
                buf.into_inner()
            }

            let config_json_cpp = r#"
    {
    "execution": {
        "timeout_secs": 10,
        "max_memory": 8589934592,
        "max_cpus": 2,
        "max_uncompressed_size": 100000000,
        "max_processes": 256
    },
    "marking": {
        "marking_scheme": "exact",
        "feedback_scheme": "auto",
        "deliminator": "&-=-&"
    },
    "project": {
        "language": "cpp"
    },
    "output": {
        "stdout": true,
        "stderr": true,
        "retcode": true
    }
    }
    "#;

            let zipped_files_cpp = vec![
                ("main", "main.zip", create_main_zip_cpp()),
                ("memo", "memo.zip", create_memo_zip_cpp()),
                (
                    "makefile",
                    "makefile.zip",
                    create_makefile_zip_cpp(),
                ),
                (
                    "config",
                    "config.json",
                    config_json_cpp.as_bytes().to_vec(),
                ),
            ];

            for (file_type, filename, content) in zipped_files_cpp {
                AssignmentFileService::create(
                    CreateAssignmentFile{
                        assignment_id: cpp_assignment_id,
                        module_id: cpp_module_id,
                        file_type: file_type.to_string(),
                        filename: filename.to_string(),
                        bytes: content,
                    }
                ).await?;
            }

            //Plagerism Assignment
            let plag_module: i64 = 10003;
            let plag_assignment: i64 = 10003;

            fn create_plag_main_zip() -> Vec<u8> {
                let mut buf = Cursor::new(Vec::new());
                {
                    let mut zip = zip::ZipWriter::new(&mut buf);
                    let options = SimpleFileOptions::default().unix_permissions(0o644);

                    let main_java = r#"
    public class Main {
        public static void main(String[] args) {
            String task = args.length > 0 ? args[0] : "task1";

            switch (task) {
                case "task1":
                    System.out.println("Hello World!");
                    break;
                default:
                    System.out.println("" + task + " is not a valid task");
            }
        }
    }
    "#;

                    zip.start_file("Main.java", options).unwrap();
                    zip.write_all(main_java.as_bytes()).unwrap();
                    zip.finish().unwrap();
                }
                buf.into_inner()
            }

            fn create_plag_memo_zip() -> Vec<u8> {
                let mut buf = Cursor::new(Vec::new());
                {
                    let mut zip = zip::ZipWriter::new(&mut buf);
                    let options = SimpleFileOptions::default().unix_permissions(0o644);

                    let helper_one = r#"
    public class StudentSolution {
    private int helperMultiply1(int a,int b){return a*b + 9;}

    public int fibonacci_U1(int n) {
    int a=0,b=1;
    // U1 tweak
    for(int i=2;i<=n;i++){int tmp=b;b=a+b;a=tmp;}
    return b;
    }

    public int factorial_U1(int n) {
    int f = 1;
    for(int i=1;i<=n;i++) f*=i;
    return f;
    }

    private String helperComment1(){return "Extra comment TXgZpkUF";}

    public int sumArray_U1(int[] arr) {
    int sum = 0;
    for(int n: arr) sum += n;
    // U1 tweak
    return sum;
    }


    public String gradeStudent(int score){
        switch(score/10){
            case 10: case 9: return "A";
            case 8: return "B";
            case 7: return "C";
            default: return "F";
        }
    }

    public int reverseString_U1(String s) {
    String rev="";
    for(int i=s.length()-1;i>=0;i--)
    rev+=s.charAt(i);
    return rev;
    }
    }
    "#;
                    zip.start_file("StudentSolution.java", options).unwrap();
                    zip.write_all(helper_one.as_bytes()).unwrap();

                    zip.finish().unwrap();
                }
                buf.into_inner()
            }

            fn create_plag_makefile_zip() -> Vec<u8> {
                let mut buf = Cursor::new(Vec::new());
                {
                    let mut zip = zip::ZipWriter::new(&mut buf);
                    let options = SimpleFileOptions::default().unix_permissions(0o644);

                    let makefile_content = r#"
    task1:
        javac -d /output Main.java StudentSolution.java && java -cp /output Main task1
    "#;

                    zip.start_file("Makefile", options).unwrap();
                    zip.write_all(makefile_content.as_bytes()).unwrap();
                    zip.finish().unwrap();
                }
                buf.into_inner()
            }

            // New config file content
            let config_json = r#"
    {
    "execution": {
        "timeout_secs": 10,
        "max_memory": 8589934592,
        "max_cpus": 2,
        "max_uncompressed_size": 100000000,
        "max_processes": 256
    },
    "marking": {
        "marking_scheme": "exact",
        "feedback_scheme": "auto",
        "deliminator": "&-=-&"
    },
    "project": {
        "language": "java"
    },
    "output": {
        "stdout": true,
        "stderr": false,
        "retcode": false
    }
    }
    "#;

            let zipped_files = vec![
                ("main", "main.zip", create_plag_main_zip()),
                ("memo", "memo.zip", create_plag_memo_zip()),
                (
                    "makefile",
                    "makefile.zip",
                    create_plag_makefile_zip(),
                ),
                (
                    "config",
                    "config.json",
                    config_json.as_bytes().to_vec(),
                ),
            ];

            for (file_type, filename, content) in zipped_files {
                AssignmentFileService::create(
                    CreateAssignmentFile{
                        assignment_id: plag_assignment,
                        module_id: plag_module,
                        file_type: file_type.to_string(),
                        filename: filename.to_string(),
                        bytes: content,
                    }
                ).await?;
            }

            //GATLAM
            //Plagerism Assignment
            let plag_module: i64 = 10003;
            let plag_assignment: i64 = 10004;

            fn create_interpreter_main_zip() -> Vec<u8> {
                let mut buf = Cursor::new(Vec::new());
                {
                    let mut zip = zip::ZipWriter::new(&mut buf);
                    let options = SimpleFileOptions::default().unix_permissions(0o644);

                    let helper_one = r#"
    //Nothing
    "#;
                    zip.start_file("Main.java", options).unwrap();
                    zip.write_all(helper_one.as_bytes()).unwrap();

                    zip.finish().unwrap();
                }
                buf.into_inner()
            }

            fn create_interpreter_memo_zip() -> Vec<u8> {
                let mut buf = Cursor::new(Vec::new());
                {
                    let mut zip = zip::ZipWriter::new(&mut buf);
                    let options = SimpleFileOptions::default().unix_permissions(0o644);

                    let helper_one = r#"
    public class StudentSolution {
    private int helperMultiply1(int a,int b){return a*b + 9;}

    public int fibonacci_U1(int n) {
    int a=0,b=1;
    // U1 tweak
    for(int i=2;i<=n;i++){int tmp=b;b=a+b;a=tmp;}
    return b;
    }

    public int factorial_U1(int n) {
    int f = 1;
    for(int i=1;i<=n;i++) f*=i;
    return f;
    }

    private String helperComment1(){return "Extra comment TXgZpkUF";}

    public int sumArray_U1(int[] arr) {
    int sum = 0;
    for(int n: arr) sum += n;
    // U1 tweak
    return sum;
    }


    public String gradeStudent(int score){
        switch(score/10){
            case 10: case 9: return "A";
            case 8: return "B";
            case 7: return "C";
            default: return "F";
        }
    }

    public String reverseString_U1(String s) {
        String rev="";
        for(int i=s.length()-1;i>=0;i--)
            rev+=s.charAt(i);
        return rev;
    }
    }
    "#;
                    zip.start_file("StudentSolution.java", options).unwrap();
                    zip.write_all(helper_one.as_bytes()).unwrap();

                    zip.finish().unwrap();
                }
                buf.into_inner()
            }

            fn create_interpreter_makefile_zip() -> Vec<u8> {
                let mut buf = Cursor::new(Vec::new());
                {
                    let mut zip = zip::ZipWriter::new(&mut buf);
                    let options = SimpleFileOptions::default().unix_permissions(0o644);

                    let makefile_content = r#"
    task1:
        javac -d /output Main.java StudentSolution.java && java -cp /output Main task1
    "#;

                    zip.start_file("Makefile", options).unwrap();
                    zip.write_all(makefile_content.as_bytes()).unwrap();
                    zip.finish().unwrap();
                }
                buf.into_inner()
            }

            // New config file content
            let config_json = r#"
    {
    "execution": {
        "max_cpus": 2,
        "max_memory": 8589934592,
        "max_processes": 256,
        "max_uncompressed_size": 100000000,
        "timeout_secs": 10
    },
    "gatlam": {
        "crossover_probability": 0.9,
        "crossover_type": "onepoint",
        "genes": [
        {
            "max_value": 5,
            "min_value": -5
        },
        {
            "max_value": 9,
            "min_value": -4
        }
        ],
        "max_parallel_chromosomes": 4,
        "mutation_probability": 0.01,
        "mutation_type": "bitflip",
        "number_of_generations": 5,
        "omega1": 0.5,
        "omega2": 0.3,
        "omega3": 0.2,
        "population_size": 1,
        "reproduction_probability": 0.8,
        "selection_size": 20,
        "task_spec": {
        "forbidden_outputs": [],
        "max_runtime_ms": null,
        "valid_return_codes": [
            0
        ]
        },
        "verbose": false
    },
    "marking": {
        "deliminator": "&-=-&",
        "feedback_scheme": "auto",
        "marking_scheme": "exact"
    },
    "output": {
        "retcode": true,
        "stderr": true,
        "stdout": true
    },
    "project": {
        "language": "java",
        "submission_mode": "gatlam"
    }
    }
    "#;

            let zipped_files = vec![
                ("main", "main.zip", create_interpreter_main_zip()),
                ("memo", "memo.zip", create_interpreter_memo_zip()),
                (
                    "makefile",
                    "makefile.zip",
                    create_interpreter_makefile_zip(),
                ),
                (
                    "config",
                    "config.json",
                    config_json.as_bytes().to_vec(),
                ),
            ];

            for (file_type, filename, content) in zipped_files {
                AssignmentFileService::create(
                    CreateAssignmentFile{
                        assignment_id: plag_assignment,
                        module_id: plag_module,
                        file_type: file_type.to_string(),
                        filename: filename.to_string(),
                        bytes: content,
                    }
                ).await?;
            }

            Ok(())
        })
    }
}
