// tests/run_code.rs

use code_manager::manager::manager::ContainerManager;
use util::execution_config::ExecutionConfig;

#[tokio::test]
async fn test_run_java_and_cpp_code() {
    let config = ExecutionConfig::default_config();

    let java_files = vec![
        ("Main.java".to_string(), b"public class Main { public static void main(String[] args) { System.out.println(\"Hello from Java Main\"); } }".to_vec()),
        ("Helper.java".to_string(), b"public class Helper { public static String greet() { return \"Hello from Helper\"; } }".to_vec()),
    ];

    let cpp_files = vec![
        ("main.cpp".to_string(), b"#include <iostream>\nint main() { std::cout << \"Hello from C++ main\" << std::endl; return 0; }".to_vec()),
        ("util.cpp".to_string(), b"// dummy util file\n".to_vec()),
    ];

    let java_commands = vec![
        "javac Main.java Helper.java".to_string(),
        "java Main".to_string(),
    ];

    let cpp_commands = vec![
        "g++ main.cpp util.cpp -o app".to_string(),
        "./app".to_string(),
    ];

    let mut all_files = Vec::new();
    all_files.extend(java_files);
    all_files.extend(cpp_files);

    let mut all_commands = Vec::new();
    all_commands.extend(java_commands);
    all_commands.extend(cpp_commands);

    let manager = ContainerManager::new(1);

    let result = manager.run(&config, all_commands, all_files, false).await;

    match result {
        Ok(outputs) => {
            assert!(
                outputs
                    .iter()
                    .any(|out| out.contains("Hello from Java Main")
                        || out.contains("Hello from Helper")
                        || out.contains("Hello from C++ main")),
                "Output should contain success message from Java or C++ code. Output: {:?}",
                outputs
            );
        }
        Err(e) => panic!("Run failed with error: {}", e),
    }
}
