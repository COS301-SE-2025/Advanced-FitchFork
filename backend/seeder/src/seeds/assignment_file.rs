use crate::seed::Seeder;
use db::models::assignment;
use db::models::assignment_file::{FileType, Model};
use sea_orm::{DatabaseConnection, EntityTrait};
use std::io::{Cursor, Write};
use zip::write::FileOptions;

pub struct AssignmentFileSeeder;

#[async_trait::async_trait]
impl Seeder for AssignmentFileSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
        // Existing seed logic
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
        ];

        for a in &assignments {
            if a.module_id == 9999 {
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
                let options = FileOptions::default().unix_permissions(0o644);

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
        System.out.println("" + "&-=-&Task1");
        System.out.println(HelperOne.subtaskA());
        System.out.println("&-=-&Task1Subtask2");
        System.out.println(HelperTwo.subtaskB());
        System.out.println("&-=-&Task1Subtask3");
        System.out.println(HelperThree.subtaskC());
    }

    static void runTask2() {
        System.out.println("&-=-&Task2");
        System.out.println(HelperTwo.subtaskX());
        System.out.println("&-=-&Task2Subtask2");
        System.out.println(HelperThree.subtaskY());
        System.out.println("&-=-&Task2Subtask3");
        System.out.println(HelperOne.subtaskZ());
    }

    static void runTask3() {
        System.out.println("&-=-&Task3");
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
                let options = FileOptions::default().unix_permissions(0o644);

                let helper_one = r#"
public class HelperOne {
    public static String subtaskA() {
        return "" + "HelperOne: Running subtask A for Task1";
    }
    public static String subtaskZ() {
        return "HelperOne: Running subtask Z for Task2";
    }
    public static String subtaskBeta() {
        return "HelperOne: Running subtask Beta for Task3";
    }
}
"#;

                let helper_two = r#"
public class HelperTwo {
    public static String subtaskB() {
        return "HelperTwo: Running subtask B for Task1";
    }
    public static String subtaskX() {
        return "HelperTwo: Running subtask X for Task2";
    }
    public static String subtaskGamma() {
        return "HelperTwo: Running subtask Gamma for Task3";
    }
}
"#;

                let helper_three = r#"
public class HelperThree {
    public static String subtaskC() {
        return "HelperThree: Running subtask C for Task1";
    }
    public static String subtaskY() {
        return "HelperThree: Running subtask Y for Task2";
    }
    public static String subtaskAlpha() {
        return "HelperThree: Running subtask Alpha for Task3";
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
                let options = FileOptions::default().unix_permissions(0o644);

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
  "language": "java"
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
    }
}
