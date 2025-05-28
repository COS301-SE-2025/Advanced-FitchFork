//!Okay this is just an example I came up with. It only runs java files and has not been extensively tested
//! This is just a proof of concept which addresses as many security issues which I discovered in a few hours

use std::{fs::File, io::Cursor, process::Stdio};
use tempfile::tempdir;
use tokio::{
    process::Command,
    time::{Duration, timeout},
};
use zip::ZipArchive;

/// Configuration for supported programming languages.
///
/// Contains language name, file extensions, Docker image to use,
/// and the command to compile/run code inside the container.
pub struct LanguageConfig {
    /// Human-readable language name.
    pub name: &'static str,
    /// Valid file extensions for source files of this language.
    pub file_extensions: &'static [&'static str],
    /// Docker image name for running the language environment.
    pub docker_image: &'static str,
    /// Shell command to compile and/or run code inside the container.
    pub run_command: &'static str,
}

/// Returns the configuration for a given language code.
///
/// # Arguments
///
/// * `lang` - The language identifier (e.g. "java", "cpp", "python").
///
/// # Returns
///
/// Returns `Some(LanguageConfig)` if the language is supported,
/// otherwise returns `None`.
fn get_language_config(lang: &str) -> Option<LanguageConfig> {
    match lang {
        "java" => Some(LanguageConfig {
            name: "Java",
            file_extensions: &[".java"],
            docker_image: "openjdk:17-slim",
            run_command: "javac -d /output /code/*.java && java -cp /output Main",
        }),
        "cpp" => Some(LanguageConfig {
            name: "C++",
            file_extensions: &[".cpp", ".h"],
            docker_image: "gcc:13",
            run_command: "find /code -name '*.cpp' -exec g++ -o /output/app {} + && /output/app",
        }),
        "python" => Some(LanguageConfig {
            name: "Python",
            file_extensions: &[".py"],
            docker_image: "python:3.11-slim",
            run_command: "python3 /code/main.py",
        }),
        _ => None,
    }
}

/// Runs code from a zip archive file on disk, for a given language.
///
/// # Arguments
///
/// * `zip_path` - The path to the zip archive containing the assignment code.
/// * `lang` - The language identifier to determine how to run the code.
///
/// # Behavior
///
/// Reads the zip file from disk, extracts the source code safely, runs the code
/// in a Docker container with security restrictions, and prints the program output or errors.
///
/// This function is asynchronous.
pub async fn run_assignment_code(zip_path: &str, lang: &str) {
    //read zip into memory
    println!(
        "Current working dir: {}",
        std::env::current_dir().unwrap().display()
    );
    let zip_bytes = std::fs::read(zip_path).expect("Failed to read zip file");

    //run code
    match run_assignment_zip(&zip_bytes, lang).await {
        Ok(output) => println!("Program output:\n{}", output),
        Err(e) => eprintln!("Error: {}", e),
    }
}

/// Runs code contained in a zip archive byte slice for a specified language.
///
/// # Arguments
///
/// * `zip_bytes` - Byte slice of the zip archive containing source code files.
/// * `lang` - The language identifier (e.g. "java", "cpp", "python").
///
/// # Returns
///
/// On success, returns `Ok(String)` containing the program's standard output.
///
/// On failure, returns `Err` with a boxed error describing the failure.
///
/// # Security
///
/// This function unpacks the zip safely with:
/// - Zip slip protection (no `..` in paths)
/// - Max uncompressed size limit to prevent zip bombs
/// - Only allows files with supported extensions for the language
///
/// It runs the code inside a restricted Docker container with:
/// - No network access
/// - Memory and CPU limits
/// - Process count limits
/// - No privilege escalation
/// - Read-only volume for source code
/// - Writable volume for output
///
/// # Notes
///
/// This is a proof of concept designed to mitigate common security risks
/// when running untrusted code submitted as zip archives.
async fn run_assignment_zip(
    zip_bytes: &[u8],
    lang: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    //Accepted languages
    let config = get_language_config(lang).ok_or("Unsupported language")?;

    //I get to explain my super genius solution for security issues lets goooooooooooooooo
    //======================================================================================
    //Okay here is the basic problem
    //Step 1) Studnet submits their code
    //Step 2) Their code is run with the memo Main
    //Step 3) The students code can overwrite or edit the memo Main
    //Step 4) You probably: "WHy is that bad? The code is compiled first so it doesn't matter if they edit the main? It doesn't get saved anyway"
    //Step 5) The problem is the Main changes with GATLAM. Its run multiple times. If the student can alter code while it evolves -> not good
    //My solution? -> We have 2 temporary directories. One is read-only and one is read-write
    //Temporary directories delete automatically when it goes out of scope so it is perfect for this
    //The code is placed in the read-only directory so nothing can be tampered with
    //All the output needs to be written though so that goes to the read-write directory
    //So code is run, but sorce code in read-only directory cannot be tampered with, but all the output is piped to another directory
    //This way no code can be overwritten, but you can still get the output which needs to be written
    //Okay now that I am writing this down I'm realising this isn't that groundbreaking. Good security measure anyway
    //I'm leaving these comments here

    //Read-only directory
    let temp_code_dir = tempdir()?;
    //Read-write directory
    let temp_output_dir = tempdir()?;

    let temp_code_path = temp_code_dir.path();
    let temp_output_path = temp_output_dir.path();

    //analyse zip file before extracting it for security
    let reader = Cursor::new(zip_bytes);
    let mut archive = ZipArchive::new(reader)?;

    //Maximum size that the uncompressed file may be
    //This is to prevent zip bombing and all that
    let max_total_uncompressed: u64 = 50 * 1024 * 1024; //50 MB
    let mut total_uncompressed: u64 = 0;

    //Loop over every file in zip archive
    for i in 0..archive.len() {
        //Gets a file in the zip
        let mut file = archive.by_index(i)?;

        total_uncompressed += file.size();
        //Exceeded maximum size
        if total_uncompressed > max_total_uncompressed {
            return Err("Zip file too large when decompressed".into());
        }

        //gets the original filepath of the file
        let raw_name = file.name();

        //Zip Slip Prevention
        //Might actually not be needed since it is in an isolated environment that gets deleted
        if raw_name.contains("..") || raw_name.starts_with('/') || raw_name.contains('\\') {
            return Err(format!("Invalid file path in zip: {}", raw_name).into());
        }

        let is_valid = config
            .file_extensions
            .iter()
            .any(|ext| raw_name.ends_with(ext) || raw_name.ends_with('/'));

        if !is_valid {
            return Err(format!("Unsupported file type in zip: {}", raw_name).into());
        }

        let outpath = temp_code_path.join(raw_name);

        //Actually extracting the zip now
        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    //Maybe remove this
    println!(
        "Zip file for language '{}' extracted safely to: {}",
        config.name,
        temp_code_path.display()
    );

    //Now comes the fun part - running a docker enviroment

    let docker_command = Command::new("docker")
        .arg("run") //run docker
        .arg("--rm") //automatically removes container once done
        .arg("--network=none") // disable network
        .arg("--memory=128m") //limit memory usage to 128MB
        .arg("--cpus=1") //limit CPU usage to one core
        .arg("--pids-limit=64") //max processes to 64. -> prevents fork bombs
        .arg("--security-opt=no-new-privileges") //prevent privilege escalation
        .arg("--user=1000:1000") //run with user id 1000 and group id 1000 which is usually non-root user
        .arg("-v") //adds volume mount
        .arg(format!("{}:/code:ro", temp_code_path.display())) //read-only for sources
        .arg("-v") //adds another volume mount
        .arg(format!("{}:/output", temp_output_path.display())) //writable for build output
        .arg(config.docker_image) //language
        .arg("sh") //run shell inside container
        .arg("-c")
        .arg(config.run_command) //run the code
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?; //spawn for timeout control

    //Specify timeout
    let timeout_seconds = 60;
    let output = timeout(
        Duration::from_secs(timeout_seconds),
        docker_command.wait_with_output(),
    )
    .await??;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(format!("Execution failed:\n{}", err).into())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::run_assignment_zip;
//     use std::fs;

//     async fn run_test_zip(path: &str, lang: &str) {
//         let zip_bytes = fs::read(&path).expect(&format!("Failed to read test zip file: {}", path));

//         match run_assignment_zip(&zip_bytes, lang).await {
//             Ok(output) => panic!("{} unexpectedly succeeded with output:\n{}", path, output),
//             Err(e) => println!("{} correctly failed with error: {}", path, e),
//         }
//     }

//     async fn run_success_zip(path: &str, lang: &str) {
//         let zip_bytes = fs::read(path).expect(&format!("Failed to read {}", path));
//         match super::run_assignment_zip(&zip_bytes, lang).await {
//             Ok(output) => println!("{} succeeded with output:\n{}", path, output),
//             Err(e) => panic!("{} failed unexpectedly with error: {}", path, e),
//         }
//     }

//     #[tokio::test]
//     async fn test_good_java_example_succeeds() {
//         run_success_zip("src/files/good_java_example.zip", "java").await;
//     }

//     #[tokio::test]
//     async fn test_good_cpp_example_succeeds() {
//         run_success_zip("src/files/good_cpp_example.zip", "cpp").await;
//     }

//     #[tokio::test]
//     async fn test_good_python_example_succeeds() {
//         run_success_zip("src/files/good_python_example.zip", "python").await;
//     }

// #[tokio::test]
// async fn test_infinite_loop_rejected() {
//     run_test_zip("src/files/infinite_loop_java_example.zip", "java").await;
// }

// #[tokio::test]
// async fn test_memory_overflow_rejected() {
//     run_test_zip("src/files/memory_overflow_java_example.zip", "java").await;
// }

// #[tokio::test]
// async fn test_fork_bomb_rejected() {
//     run_test_zip("src/files/fork_bomb_java_example.zip", "java").await;
// }

// #[tokio::test]
// async fn test_edit_code_rejected() {
//     let zip_path = "src/files/edit_code_java_example.zip";
//     let zip_bytes = std::fs::read(zip_path).expect("Failed to read zip");
//     let result = run_assignment_zip(&zip_bytes, "java").await;

//     match result {
//         Ok(output) => {
//             if output.contains("failed") {
//                 //success
//             } else {
//                 panic!("succeeded with output:\n{}", output);
//             }
//         }
//         Err(_) => {
//             //good if error
//         }
//     }
// }

// #[tokio::test]
// async fn test_priviledge_escalation_rejected() {
//     let path = "src/files/priviledge_escalation_java_example.zip";
//     let zip_bytes = std::fs::read(path).expect("Failed to read test zip");

//     match run_assignment_zip(&zip_bytes, "java").await {
//         Ok(output) => {
//             assert!(
//                 output.contains("uid=1000"),
//                 "Unexpected user privileges:\n{}",
//                 output
//             );
//         }
//         Err(e) => panic!("Program failed unexpectedly: {}", e),
//     }
// }

// #[tokio::test]
// async fn test_network_access_rejected() {
//     let zip_path = "src/files/access_network_java_example.zip";
//     let zip_bytes = std::fs::read(zip_path).expect("Failed to read zip");
//     let result = run_assignment_zip(&zip_bytes, "java").await;

//     match result {
//         Ok(output) => {
//             if output.contains("Network access blocked") {
//                 //success
//             } else {
//                 panic!("succeeded with output:\n{}", output);
//             }
//         }
//         Err(_) => {
//             //good if error
//         }
//     }
// }
// }
