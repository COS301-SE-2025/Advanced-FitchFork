use crate::seed::Seeder;
use db::models::{assignment, assignment_submission::Model as AssignmentSubmissionModel, user};
use sea_orm::{DatabaseConnection, EntityTrait};
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

        let users = user::Entity::find()
            .all(db)
            .await
            .expect("Failed to fetch users");

        if users.is_empty() {
            panic!("No users found — at least one user must exist to seed assignment_submissions");
        }

        for assignment in &assignments {
            if assignment.module_id == 9999 || assignment.module_id == 9998 {
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
                        dummy_filename,
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
            filename_java,
            &content_java,
        )
        .await
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
            filename_cpp,
            &content_cpp,
        )
        .await
        {
            Ok(_) => {}
            Err(e) => {
                eprintln!(
                    "Failed to seed hardcoded C++ submission for assignment {} user {}: {}",
                    assignment_id_cpp, user_id, e
                );
            }
        }
    }
}