use crate::seed::Seeder;
use db::models::{assignment, assignment_submission::Model as AssignmentSubmissionModel, user};
use sea_orm::{DatabaseConnection, EntityTrait};
use std::io::{Cursor, Write};
use zip::write::FileOptions;

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

        // Regular seeding logic for all assignments/users
        for assignment in &assignments {
            if assignment.module_id == 9999 {
                continue;
            }
            for user in &users {
                for counter in 1..=2 {
                    // Skip the specific one we'll hardcode below
                    if assignment.id == 9999 && user.id == 1 && counter == 1 {
                        continue;
                    }

                    let dummy_filename = "submission.txt";
                    let dummy_content = format!(
                        "Dummy submission content for assignment {} by user {}",
                        assignment.id, user.id
                    );

                    match AssignmentSubmissionModel::save_file(
                        db,
                        assignment.id,
                        user.id,
                        counter,
                        dummy_filename,
                        dummy_content.as_bytes(),
                    )
                    .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!(
                                "Failed to save assignment_submission file for assignment {} user {}: {}",
                                assignment.id, user.id, e
                            );
                        }
                    }
                }
            }
        }

        //Hardcoded seeding of a "memo-as-submission" zip file for user 1, attempt 1, assignment 9999
        fn create_memo_zip_as_submission() -> Vec<u8> {
            let mut buf = Cursor::new(Vec::new());
            {
                let mut zip = zip::ZipWriter::new(&mut buf);
                let options = FileOptions::default().unix_permissions(0o644);

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

        let assignment_id = 9999;
        let user_id = 1;
        let attempt_number = 1;
        let filename = "submission_memo_clone.zip";
        let content = create_memo_zip_as_submission();

        match AssignmentSubmissionModel::save_file(
            db,
            assignment_id,
            user_id,
            attempt_number,
            filename,
            &content,
        )
        .await
        {
            Ok(_) => {
                //Nothing
            }
            Err(e) => {
                eprintln!(
                    "Failed to seed hardcoded submission for assignment {} user {}: {}",
                    assignment_id, user_id, e
                );
            }
        }
    }
}
