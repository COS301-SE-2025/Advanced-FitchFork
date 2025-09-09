use crate::seed::Seeder;
use db::models::{assignment, assignment_submission::Model as AssignmentSubmissionModel, user};
use rand::seq::SliceRandom;
use rand::{Rng, distributions::Alphanumeric};
use sea_orm::{DatabaseConnection, EntityTrait};
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use std::pin::Pin;

pub struct AssignmentSubmissionSeeder;

impl Seeder for AssignmentSubmissionSeeder {
    fn seed<'a>(&'a self, db: &'a DatabaseConnection) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            // Fetch all assignments and users
            let assignments = assignment::Entity::find()
                .all(db)
                .await
                .expect("Failed to fetch assignments");

            let mut users = user::Entity::find()
                .all(db)
                .await
                .expect("Failed to fetch users");

            if users.is_empty() {
                panic!("No users found — at least one user must exist to seed assignment_submissions");
            }

            for user in &users {
                let assignment_id = 10003;
                let attempt_number = 1;
                let filename = format!("studentSubmission_user{}.zip", user.id);
                let content = create_student_submission_plagiarism(user.id);

                match AssignmentSubmissionModel::save_file(
                    db,
                    assignment_id,
                    user.id,
                    attempt_number,
                    80,
                    100,
                    false,
                    &filename,
                    "hash123#",
                    &content,
                )
                .await
                {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!(
                            "Failed to seed plagiarism submission for assignment {} user {}: {}",
                            assignment_id, user.id, e
                        );
                    }
                }
            }

            users.truncate(2);

            for assignment in &assignments {
                if assignment.module_id == 9999
                    || assignment.module_id == 9998
                    || assignment.module_id == 10003
                {
                    continue;
                }
                for user in &users {
                    for counter in 1..=2 {
                        if assignment.id == 9999 && user.id == 1 && counter == 1 {
                            continue;
                        }

                            let dummy_filename = "submission.txt";
                            let dummy_content = format!(
                                "Dummy submission content for assignment {} by user {}",
                                assignment.id, user.id
                            );

                        let _ = AssignmentSubmissionModel::save_file(
                            db,
                            assignment.id,
                            user.id,
                            counter,
                            rand::random::<i64>() % 100,
                            100,
                            false,
                            dummy_filename,
                            "hash123#",
                            dummy_content.as_bytes(),
                        ).await;
                    }
                }
            }

                fn create_memo_zip_as_submission_java() -> Vec<u8> {
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
                return "HelperTwo: Subtask for Task1\nThis as well\nWrong output here";
            }
            public static String subtaskX() {
                return "HelperTwo: Subtask for Task2\nThis as well\nAnd this";
            }
            public static String subtaskGamma() {
                return "";
            }
        }
        "#;

                        let helper_three = r#"
        public class HelperThree {
            public static String subtaskC() {
                return "HelperThree: Subtask for Task1\nThis as well\nAnd this";
            }
            public static String subtaskY() {
                return "HelperThree: Subtask for Task2\nThis as well\nAnd this\nAdditional wrong line";
            }
            public static String subtaskAlpha() {
                return "HelperThree: Subtask for Task3\nThis as well";
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

                fn create_cpp_submission_zip() -> Vec<u8> {
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
            return "HelperTwo: Subtask for Task1\nThis as well\nWrong output here";
        }
        std::string HelperTwo::subtaskX() {
            return "HelperTwo: Subtask for Task2\nThis as well\nAnd this";
        }
        std::string HelperTwo::subtaskGamma() {
            return "";
        }
        "#;

                        let helper_three_cpp = r#"
        #include "HelperThree.h"
        std::string HelperThree::subtaskC() {
            return "HelperThree: Subtask for Task1\nThis as well\nAnd this";
        }
        std::string HelperThree::subtaskY() {
            return "HelperThree: Subtask for Task2\nThis as well\nAnd this\nAdditional wrong line";
        }
        std::string HelperThree::subtaskAlpha() {
            return "HelperThree: Subtask for Task3\nThis as well";
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

                let assignment_id_java = 9999;
                let user_id = 1;
                let attempt_number = 1;
                let filename_java = "submission_memo_clone.zip";
                let content_java = create_memo_zip_as_submission_java();

            match AssignmentSubmissionModel::save_file(
                db,
                assignment_id_java,
                user_id,
                attempt_number,
                80,
                100,
                false,
                filename_java,
                "hash123#",
                &content_java,
            ).await
            {
                Ok(_) => {}
                Err(e) => {
                    eprintln!(
                        "Failed to seed hardcoded submission for assignment {} user {}: {}",
                        assignment_id_java, user_id, e
                    );
                }
            }

                let assignment_id_cpp = 9998;
                let filename_cpp = "submission_cpp_clone.zip";
                let content_cpp = create_cpp_submission_zip();

            match AssignmentSubmissionModel::save_file(
                db,
                assignment_id_cpp,
                user_id,
                attempt_number,
                90,
                100,
                false,
                filename_cpp,
                "hash123#",
                &content_cpp,
            ).await
            {
                Ok(_) => {}
                Err(e) => {
                    eprintln!(
                        "Failed to seed hardcoded C++ submission for assignment {} user {}: {}",
                        assignment_id_cpp, user_id, e
                    );
                }
            }

            // Plagiarism Submissions

            fn create_student_submission_plagiarism(user_id: i64) -> Vec<u8> {
                let mut buf = Cursor::new(Vec::new());
                let mut zip = zip::ZipWriter::new(&mut buf);
                let options = SimpleFileOptions::default().unix_permissions(0o644);

                // 0 = identical
                // 1 = partially similar
                // 2 = mostly unique
                let similarity_group =
                    if user_id == 10 || user_id == 11 || user_id == 27 || user_id == 41 {
                        0
                    } else if user_id == 15 {
                        2
                    } else {
                        1
                    };

                let java_code = generate_convoluted_student_code(user_id, similarity_group);

                zip.start_file("StudentSolution.java", options).unwrap();
                zip.write_all(java_code.as_bytes()).unwrap();
                zip.finish().unwrap();

                buf.into_inner()
            }

            fn generate_convoluted_student_code(user_id: i64, group: usize) -> String {
                let mut rng = rand::thread_rng();

                // Pool of base methods with multiple lines
                let base_methods = vec![
                    (
                        "sumArray",
                        "int[] arr",
                        vec!["int sum = 0;", "for(int n: arr) sum += n;", "return sum;"],
                    ),
                    (
                        "factorial",
                        "int n",
                        vec!["int f = 1;", "for(int i=1;i<=n;i++) f*=i;", "return f;"],
                    ),
                    (
                        "isPrime",
                        "int n",
                        vec![
                            "if(n<=1) return false;",
                            "for(int i=2;i*i<=n;i++) if(n%i==0) return false;",
                            "return true;",
                        ],
                    ),
                    (
                        "printPattern",
                        "int n",
                        vec![
                            "for(int i=1;i<=n;i++){",
                            "for(int j=0;j<i;j++) System.out.print('*');",
                            "System.out.println();",
                            "}",
                        ],
                    ),
                    (
                        "fibonacci",
                        "int n",
                        vec![
                            "int a=0,b=1;",
                            "for(int i=2;i<=n;i++){int tmp=b;b=a+b;a=tmp;}",
                            "return b;",
                        ],
                    ),
                    (
                        "reverseString",
                        "String s",
                        vec![
                            "String rev=\"\";",
                            "for(int i=s.length()-1;i>=0;i--)",
                            "rev+=s.charAt(i);",
                            "return rev;",
                        ],
                    ),
                ];

                // Pick methods according to group
                let method_count = match group {
                    0 => base_methods.len(),                    // identical -> use all
                    1 => rng.gen_range(3..=base_methods.len()), // partial
                    2 => rng.gen_range(2..=3),                  // mostly unique
                    _ => 3,
                };

                let mut chosen_methods = base_methods.clone();
                chosen_methods.shuffle(&mut rng);
                let chosen_methods = &chosen_methods[..method_count];

                let mut methods_code = Vec::new();
                for (idx, (name, args, body_lines)) in chosen_methods.iter().enumerate() {
                    let method_code = match group {
                        0 => format!(
                            "public {} {}({}) {{\n{}\n}}",
                            if name == &"isPrime" {
                                "boolean"
                            } else if name == &"reverseString" {
                                "String"
                            } else {
                                "int"
                            },
                            name,
                            args,
                            body_lines.join("\n")
                        ),
                        1 => {
                            // partially similar: tweak variable names, add extra comments
                            let name_mod = format!("{}_U{}", name, user_id);
                            let body_mod: Vec<String> = body_lines
                                .iter()
                                .map(|l| {
                                    if rng.gen_bool(0.3) {
                                        format!("// U{} tweak\n{}", user_id, l)
                                    } else {
                                        l.to_string()
                                    }
                                })
                                .collect();
                            format!(
                                "public int {}({}) {{\n{}\n}}",
                                name_mod,
                                args,
                                body_mod.join("\n")
                            )
                        }
                        2 => {
                            // mostly unique: totally random methods
                            let random_lines: Vec<String> = (0..rng.gen_range(3..6))
                                .map(|_| {
                                    format!(
                                        "System.out.println(\"UNIQUE_{}:{}\");",
                                        random_token(6),
                                        random_token(12)
                                    )
                                })
                                .collect();
                            format!(
                                "public void uniqueMethod{}() {{\n{}\n}}",
                                idx,
                                random_lines.join("\n")
                            )
                        }
                        _ => format!("public void dummy() {{}}"),
                    };
                    methods_code.push(method_code);
                }

                // Helper methods added
                if group != 2 {
                    methods_code.push(format!(
                        "private int helperMultiply{}(int a,int b){{return a*b + {};}}",
                        user_id,
                        rng.gen_range(0..10)
                    ));
                    methods_code.push(format!(
                        "private String helperComment{}(){{return \"Extra comment {}\";}}",
                        user_id,
                        random_token(8)
                    ));
                }

                // Increase complexity hurray
                if group != 2 {
                    methods_code.push(
                        r#"
    public String gradeStudent(int score){
        switch(score/10){
            case 10: case 9: return "A";
            case 8: return "B";
            case 7: return "C";
            default: return "F";
        }
    }"#
                        .to_string(),
                    );
                }

                // Shuffle em
                methods_code.shuffle(&mut rng);

                format!(
                    "public class StudentSolution {{\n{}\n}}",
                    methods_code.join("\n\n")
                )
            }

            fn random_token(len: usize) -> String {
                rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(len)
                    .map(char::from)
                    .collect()
            }
        })
    }
}