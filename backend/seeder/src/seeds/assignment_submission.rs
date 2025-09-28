use crate::seed::Seeder;
use db::models::assignment_submission::SubmissionStatus;
use db::models::{assignment, assignment_submission::Model as AssignmentSubmissionModel, user};
use rand::seq::SliceRandom;
use rand::{Rng, distributions::Alphanumeric};
use sea_orm::ActiveModelTrait;
use sea_orm::{DatabaseConnection, EntityTrait, IntoActiveModel, Set};
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;

pub struct AssignmentSubmissionSeeder;

#[async_trait::async_trait]
impl Seeder for AssignmentSubmissionSeeder {
    async fn seed(&self, db: &DatabaseConnection) {
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

            let plag_file = match AssignmentSubmissionModel::save_file(
                db,
                assignment_id,
                user.id,
                attempt_number,
                80.0,
                100.0,
                false,
                &filename,
                "hash123#",
                &content,
            )
            .await
            {
                Ok(plag_file) => plag_file,
                Err(e) => panic!(
                    "Failed to seed assignment {} user {}: {}",
                    assignment_id, user.id, e
                ),
            };

            let mut am = plag_file.into_active_model();
            am.status = Set(SubmissionStatus::Graded);
            let _updated_file = am.update(db).await.expect(&format!(
                "Failed to update submission {} for user {}",
                assignment_id, user.id
            ));
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
                        rand::random::<f64>() % 100.0,
                        100.0,
                        false,
                        dummy_filename,
                        "hash123#",
                        dummy_content.as_bytes(),
                    )
                    .await;
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

                let main_java = r#"
public class Main {
    public static void main(String[] args) {
        runTask1();
        runTask2();
        runTask3();
    }

    static void runTask1() {
        System.out.println(HelperOne.subtaskA());
        System.out.println(HelperTwo.subtaskB());
    }

    static void runTask2() {
        System.out.println(HelperOne.subtaskZ());
    }

    static void runTask3() {
        System.out.println(HelperOne.subtaskBeta());
        System.out.println(HelperTwo.subtaskGamma());
    }
}

"#;

                zip.start_file("HelperOne.java", options).unwrap();
                zip.write_all(helper_one.as_bytes()).unwrap();

                zip.start_file("HelperTwo.java", options).unwrap();
                zip.write_all(helper_two.as_bytes()).unwrap();

                zip.start_file("HelperThree.java", options).unwrap();
                zip.write_all(helper_three.as_bytes()).unwrap();

                zip.start_file("Main.java", options).unwrap();
                zip.write_all(main_java.as_bytes()).unwrap();

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
    char* leak = new char[100]; // memory leak
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

                let main_cpp = r####"
#include <iostream>
#include <string>
#include "HelperOne.h"
#include "HelperTwo.h"
#include "HelperThree.h"

void runTask1() {
    std::cout << "###Task1Subtask1\n" << HelperOne::subtaskA() << std::endl;
}

void runTask2() {
    std::cout << "###Task2Subtask1\n" << HelperTwo::subtaskX() << std::endl;
        std::cout << "###Task3Subtask2\n" << HelperOne::subtaskZ() << std::endl;
}

void runTask3() {
    std::cout << "###Task3Subtask2\n" << HelperOne::subtaskBeta() << std::endl;
    std::cout << "###Task3Subtask3\n" << HelperTwo::subtaskGamma() << std::endl;
}

int main() {
    runTask1();
    runTask2();
    runTask3();
    return 0;
}
"####;

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

                zip.start_file("Main.cpp", options).unwrap();
                zip.write_all(main_cpp.as_bytes()).unwrap();

                zip.finish().unwrap();
            }
            buf.into_inner()
        }

        let assignment_id_java = 9999;
        let user_id = 1;
        let attempt_number = 1;
        let filename_java = "submission_memo_clone.zip";
        let content_java = create_memo_zip_as_submission_java();

        let file1 = match AssignmentSubmissionModel::save_file(
            db,
            assignment_id_java,
            user_id,
            attempt_number,
            80.0,
            100.0,
            false,
            filename_java,
            "hash123#",
            &content_java,
        )
        .await
        {
            Ok(file1) => file1,
            Err(e) => panic!(
                "Failed to seed assignment {} user {}: {}",
                assignment_id_java, user_id, e
            ),
        };

        let mut am = file1.into_active_model();
        am.status = Set(SubmissionStatus::Graded);
        let _updated_file = am.update(db).await.expect(&format!(
            "Failed to update submission {} for user {}",
            assignment_id_java, user_id
        ));

        let assignment_id_cpp = 9998;
        let filename_cpp = "submission_cpp_clone.zip";
        let content_cpp = create_cpp_submission_zip();

        let file = match AssignmentSubmissionModel::save_file(
            db,
            assignment_id_cpp,
            user_id,
            attempt_number,
            90.0,
            100.0,
            false,
            filename_cpp,
            "hash123#",
            &content_cpp,
        )
        .await
        {
            Ok(file) => file,
            Err(e) => panic!(
                "Failed to seed assignment {} user {}: {}",
                assignment_id_cpp, user_id, e
            ),
        };

        let mut am = file.into_active_model();
        am.status = Set(SubmissionStatus::Graded);
        let _updated_file = am.update(db).await.expect(&format!(
            "Failed to update submission {} for user {}",
            assignment_id_cpp, user_id
        ));

        // Plagiarism Submissions

        fn create_student_submission_plagiarism(user_id: i64) -> Vec<u8> {
            let mut buf = Cursor::new(Vec::new());
            let mut zip = zip::ZipWriter::new(&mut buf);
            let options = SimpleFileOptions::default().unix_permissions(0o644);

            // 0 = identical, 1 = partially similar, 2 = mostly unique
            let similarity_group =
                if user_id == 10 || user_id == 11 || user_id == 27 || user_id == 41 {
                    0
                } else if user_id == 15 {
                    2
                } else {
                    1
                };

            // Generate C++: two headers + two impls
            let (sol_h, sol_cpp) = generate_cpp_primary(user_id, similarity_group);
            let (util_h, util_cpp) = generate_cpp_secondary(user_id, similarity_group);

            // --- file set: StudentSolution
            zip.start_file("StudentSolution.h", options).unwrap();
            zip.write_all(sol_h.as_bytes()).unwrap();

            zip.start_file("StudentSolution.cpp", options).unwrap();
            zip.write_all(sol_cpp.as_bytes()).unwrap();

            // --- file set: StudentUtils
            zip.start_file("StudentUtils.h", options).unwrap();
            zip.write_all(util_h.as_bytes()).unwrap();

            zip.start_file("StudentUtils.cpp", options).unwrap();
            zip.write_all(util_cpp.as_bytes()).unwrap();

            zip.finish().unwrap();
            buf.into_inner()
        }

        fn generate_cpp_primary(user_id: i64, group: usize) -> (String, String) {
            use rand::Rng;

            // (name, rett, args, body_lines)
            let base: Vec<(&str, &str, &str, Vec<&str>)> = vec![
                (
                    "sumArray",
                    "int",
                    "const std::vector<int>& arr",
                    vec!["int s = 0;", "for (int n : arr) s += n;", "return s;"],
                ),
                (
                    "factorial",
                    "int",
                    "int n",
                    vec![
                        "int f = 1;",
                        "for (int i = 1; i <= n; ++i) f *= i;",
                        "return f;",
                    ],
                ),
                (
                    "isPrime",
                    "bool",
                    "int n",
                    vec![
                        "if (n <= 1) return false;",
                        "for (int i = 2; i * i <= n; ++i) if (n % i == 0) return false;",
                        "return true;",
                    ],
                ),
                (
                    "fibonacci",
                    "int",
                    "int n",
                    vec![
                        "int a = 0, b = 1;",
                        "for (int i = 2; i <= n; ++i) { int t = b; b = a + b; a = t; }",
                        "return n == 0 ? 0 : b;",
                    ],
                ),
                (
                    "reverseString",
                    "std::string",
                    "const std::string& s",
                    vec!["std::string r(s.rbegin(), s.rend());", "return r;"],
                ),
                (
                    "printPattern",
                    "std::string",
                    "int n",
                    vec![
                        "std::ostringstream out;",
                        "for (int i = 1; i <= n; ++i) {",
                        "  for (int j = 0; j < i; ++j) out << '*';",
                        "  out << '\\n';",
                        "}",
                        "return out.str();",
                    ],
                ),
            ];

            let mut rng = rand::thread_rng();
            let method_count = match group {
                0 => base.len(), // identical: use all
                1 => rng.gen_range(3..=base.len()),
                2 => rng.gen_range(2..=3),
                _ => 3,
            };

            // pick subset
            let mut pool = base.clone();
            pool.shuffle(&mut rng);
            let chosen = &pool[..method_count];

            // Maybe rename for group 1 to introduce partial similarity
            let mut protos = Vec::new();
            let mut impls = Vec::new();

            let cls = "StudentSolution";
            let ns = "student";

            for (_, (name, rett, args, body)) in chosen.iter().enumerate() {
                let mut fn_name = name.to_string();
                if group == 1 {
                    fn_name = format!("{}_U{}", name, user_id);
                } else if group == 2 {
                    // keep base names but add noise inside
                }

                // header proto
                protos.push(format!("    static {} {}({});", rett, fn_name, args));

                // impl body
                let mut body_lines: Vec<String> = body.iter().map(|s| s.to_string()).collect();
                if group == 1 {
                    // sprinkle comments
                    if rng.gen_bool(0.35) {
                        body_lines.insert(0, format!("// U{} tweak", user_id));
                    }
                } else if group == 2 {
                    // add noise prints
                    if *rett == "void" {
                        // none here, but keeping the pattern
                    } else {
                        body_lines.insert(0, format!("// noise {}", random_token(6)));
                    }
                }

                impls.push(
                    format!(
                        "{} {}::{}::{}({}) {{\n    {}\n}}\n",
                        rett,
                        ns,
                        cls,
                        fn_name,
                        args,
                        body_lines.join("\n    ")
                    )
                    .replace(
                        &format!("{} {}::", rett, ns),
                        &format!(
                            "#include \"{}{}.h\"\nnamespace {} {{\n{} {}::",
                            cls, "", ns, rett, cls
                        ),
                    ),
                );
                // Fix include placement later (we'll build whole files below)
            }

            // Optional extra helpers for groups != 2
            if group != 2 {
                let helper_a = format!("    static int helperMultiply{}(int a, int b);", user_id);
                let helper_b = format!("    static std::string helperNote{}();", user_id);
                protos.push(helper_a.clone());
                protos.push(helper_b.clone());

                impls.push(format!(
                    r#"int {ns}::{cls}::helperMultiply{u}(int a, int b) {{
            return a * b + {k};
        }}

        std::string {ns}::{cls}::helperNote{u}() {{
            return std::string("Extra:") + "{tok}";
        }}
        "#,
                    ns = ns,
                    cls = cls,
                    u = user_id,
                    k = rng.gen_range(0..10),
                    tok = random_token(8)
                ));
            }

            // Build header + impl strings
            let header = format!(
                r#"#pragma once
        #include <string>
        #include <vector>
        #include <sstream>

        namespace {ns} {{
        struct {cls} {{
        {protos}
        }};
        }} // namespace {ns}
        "#,
                ns = ns,
                cls = cls,
                protos = protos.join("\n")
            );

            let impl_src = format!(
                r#"#include "{cls}.h"

        namespace {ns} {{

        {impls}

        }} // namespace {ns}
        "#,
                cls = cls,
                ns = ns,
                impls = impls.join("\n")
            );

            (header, impl_src)
        }

        fn generate_cpp_secondary(user_id: i64, group: usize) -> (String, String) {
            use rand::Rng;

            // (name, rett, args, body_lines)
            let base: Vec<(&str, &str, &str, Vec<&str>)> = vec![
                (
                    "minArray",
                    "int",
                    "const std::vector<int>& arr",
                    vec![
                        "if (arr.empty()) return 0;",
                        "int m = arr[0];",
                        "for (int v : arr) if (v < m) m = v;",
                        "return m;",
                    ],
                ),
                (
                    "maxArray",
                    "int",
                    "const std::vector<int>& arr",
                    vec![
                        "if (arr.empty()) return 0;",
                        "int m = arr[0];",
                        "for (int v : arr) if (v > m) m = v;",
                        "return m;",
                    ],
                ),
                (
                    "avgArray",
                    "int",
                    "const std::vector<int>& arr",
                    vec![
                        "if (arr.empty()) return 0;",
                        "long long s = 0;",
                        "for (int v : arr) s += v;",
                        "return static_cast<int>(s / (long long)arr.size());",
                    ],
                ),
                (
                    "countVowels",
                    "int",
                    "const std::string& s",
                    vec![
                        "int c = 0;",
                        "for (unsigned char ch : s) {",
                        "  char t = static_cast<char>(::tolower(ch));",
                        "  if (t=='a'||t=='e'||t=='i'||t=='o'||t=='u') ++c;",
                        "}",
                        "return c;",
                    ],
                ),
                (
                    "isPalindrome",
                    "bool",
                    "const std::string& s",
                    vec![
                        "int i = 0, j = (int)s.size()-1;",
                        "while (i < j) {",
                        "  if (s[i] != s[j]) return false;",
                        "  ++i; --j;",
                        "}",
                        "return true;",
                    ],
                ),
                (
                    "wordReverse",
                    "std::string",
                    "const std::string& s",
                    vec![
                        "std::istringstream iss(s);",
                        "std::vector<std::string> w;",
                        "for (std::string p; iss >> p; ) w.push_back(p);",
                        "std::ostringstream out;",
                        "for (int i = (int)w.size()-1; i >= 0; --i) {",
                        "  out << w[i]; if (i) out << ' ';",
                        "}",
                        "return out.str();",
                    ],
                ),
            ];

            let mut rng = rand::thread_rng();
            let method_count = match group {
                0 => base.len(),
                1 => rng.gen_range(3..=base.len()),
                2 => rng.gen_range(2..=3),
                _ => 3,
            };

            // pick subset
            let mut pool = base.clone();
            pool.shuffle(&mut rng);
            let chosen = &pool[..method_count];

            let mut protos = Vec::new();
            let mut impls = Vec::new();

            let cls = "StudentUtils";
            let ns = "student";

            for (_, (name, rett, args, body)) in chosen.iter().enumerate() {
                let mut fn_name = name.to_string();
                if group == 1 {
                    fn_name = format!("{}_U{}", name, user_id);
                }

                protos.push(format!("    static {} {}({});", rett, fn_name, args));

                let mut body_lines: Vec<String> = body.iter().map(|s| s.to_string()).collect();
                if group == 1 && rng.gen_bool(0.30) {
                    body_lines.insert(0, format!("// U{} tweak", user_id));
                } else if group == 2 {
                    body_lines.insert(0, format!("// UTIL_NOISE {}", random_token(6)));
                }

                impls.push(format!(
                    r#"{rett} {ns}::{cls}::{fname}({args}) {{
            {body}
        }}
        "#,
                    rett = rett,
                    ns = ns,
                    cls = cls,
                    fname = fn_name,
                    args = args,
                    body = body_lines.join("\n    ")
                ));
            }

            if group != 2 {
                protos.push(format!("    static std::string bucketize(int n);"));
                impls.push(
                    r#"std::string student::StudentUtils::bucketize(int n) {
            if (n >= 90) return "A";
            if (n >= 75) return "B";
            if (n >= 60) return "C";
            return "D";
        }
        "#
                    .to_string(),
                );
            }

            let header = format!(
                r#"#pragma once
        #include <string>
        #include <vector>
        #include <sstream>

        namespace {ns} {{
        struct {cls} {{
        {protos}
        }};
        }} // namespace {ns}
        "#,
                ns = ns,
                cls = cls,
                protos = protos.join("\n")
            );

            let impl_src = format!(
                r#"#include "{cls}.h"

        namespace {ns} {{

        {impls}

        }} // namespace {ns}
        "#,
                cls = cls,
                ns = ns,
                impls = impls.join("\n")
            );

            (header, impl_src)
        }

        fn random_token(len: usize) -> String {
            rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(len)
                .map(char::from)
                .collect()
        }
    }
}
