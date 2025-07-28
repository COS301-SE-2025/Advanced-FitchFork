use crate::seed::Seeder;
use db::models::assignment;
use db::models::assignment_file::{FileType, Model};
use sea_orm::{DatabaseConnection, EntityTrait};
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;

pub struct AssignmentFileSeeder;

#[async_trait::async_trait]
impl Seeder for AssignmentFileSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        let assignments = assignment::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch assignments");

        let file_types: Vec<(FileType, fn(i32) -> String)> = vec![
            (FileType::Spec, |id| format!("spec_{}.txt", id)),
            (FileType::Memo, |id| format!("memo_{}.txt", id)),
            (FileType::Main, |id| format!("main_{}.txt", id)),
            (FileType::Makefile, |id| format!("makefile_{}.txt", id)),
            (FileType::MarkAllocator, |id| {
                format!("mark_allocator_{}.txt", id)
            }),
            (FileType::Config, |id| format!("config_{}.txt", id)),
            (FileType::Interpreter, |id| {
                format!("interpreter_{}.txt", id)
            }),
        ];

        for a in &assignments {
            if a.module_id == 9999 || a.module_id == 9998 {
                continue;
            }
            for &(ref file_type, filename_fn) in &file_types {
                let filename = filename_fn(a.id.try_into().unwrap());
                let content = format!("This is the content of assignment file {}", a.id);

                let _ = Model::save_file(
                    db,
                    a.id,
                    a.module_id,
                    file_type.clone(),
                    &filename,
                    content.as_bytes(),
                )
                .await;
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
        System.out.println(HelperThree.subtaskAlpha());
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
  "timeout_secs": 15,
  "max_memory": "768m",
  "max_cpus": "2",
  "max_processes": 256,
  "max_uncompressed_size": 50000000,
  "marking_scheme": "exact",
  "feedback_scheme": "auto"
}
"#;

        let zipped_files = vec![
            (FileType::Main, "main.zip", create_main_zip()),
            (FileType::Memo, "memo.zip", create_memo_zip()),
            (FileType::Makefile, "makefile.zip", create_makefile_zip()),
            (
                FileType::Config,
                "config.json",
                config_json.as_bytes().to_vec(),
            ),
        ];

        for (file_type, filename, content) in zipped_files {
            let _ = Model::save_file(
                db,
                special_assignment_id,
                special_module_id,
                file_type,
                filename,
                &content,
            )
            .await;
        }

        let cpp_module_id = 9998;
        let cpp_assignment_id = 9998;

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

                zip.start_file("Main.cpp", options).unwrap();
                zip.write_all(main_cpp.as_bytes()).unwrap();

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

                zip.start_file("HelperOne.cpp", options).unwrap();
                zip.write_all(helper_one_cpp.as_bytes()).unwrap();

                zip.start_file("HelperTwo.cpp", options).unwrap();
                zip.write_all(helper_two_cpp.as_bytes()).unwrap();

                zip.start_file("HelperThree.cpp", options).unwrap();
                zip.write_all(helper_three_cpp.as_bytes()).unwrap();

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
  "timeout_secs": 15,
  "max_memory": "768m",
  "max_cpus": "2",
  "max_processes": 256,
  "max_uncompressed_size": 50000000,
  "marking_scheme": "exact",
  "feedback_scheme": "auto"
}
"#;

        let zipped_files_cpp = vec![
            (FileType::Main, "main.zip", create_main_zip_cpp()),
            (FileType::Memo, "memo.zip", create_memo_zip_cpp()),
            (
                FileType::Makefile,
                "makefile.zip",
                create_makefile_zip_cpp(),
            ),
            (
                FileType::Config,
                "config.json",
                config_json_cpp.as_bytes().to_vec(),
            ),
        ];

        for (file_type, filename, content) in zipped_files_cpp {
            let _ = Model::save_file(
                db,
                cpp_assignment_id,
                cpp_module_id,
                file_type,
                filename,
                &content,
            )
            .await;
        }
    }
}
