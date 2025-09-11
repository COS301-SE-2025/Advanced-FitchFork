use crate::seed::Seeder;
use services::service::{Service, AppError};
use services::assignment::AssignmentService;
use services::assignment_interpreter::{AssignmentInterpreterService, CreateAssignmentInterpreter};
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use std::pin::Pin;

pub struct AssignmentInterpreterSeeder;

impl Seeder for AssignmentInterpreterSeeder {
    fn seed<'a>(&'a self) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + 'a>> {
        Box::pin(async move {
            let assignments = AssignmentService::find_all(&[], None).await?;

            // For each assignment, insert an interpreter file with a sample command string
            for a in &assignments {
                if a.module_id == 9999 || a.module_id == 9998 || a.module_id == 10003 {
                    continue;
                }

                let filename = format!("interpreter_{}.zip", a.id);
                let command = "g++ -std=c++17 Main.cpp -o main && ./main".to_string();
                let content = create_example_interpreter_zip();

                AssignmentInterpreterService::create(
                    CreateAssignmentInterpreter{
                        assignment_id: a.id,
                        module_id: a.module_id,
                        filename: filename,
                        command: command,
                        bytes: content,
                    }
                ).await?;
            }

            // Special assignment with realistic interpreter file
            let special_module_id: i64 = 9998;
            let special_assignment_id: i64 = 9998;

            let special_filename = "interpreter_cpp.zip";
            let special_command = "g++ -std=c++17 Main.cpp -o main && ./main".to_string();
            let special_content = create_interpreter_zip_cpp();

            AssignmentInterpreterService::create(
                CreateAssignmentInterpreter{
                    assignment_id: special_assignment_id,
                    module_id: special_module_id,
                    filename: special_filename.to_string(),
                    command: special_command,
                    bytes: special_content,
                }
            ).await?;

            let gatlam_content = create_gatlam_interpreter_zip();

            AssignmentInterpreterService::create(
                CreateAssignmentInterpreter{
                    assignment_id: 10004,
                    module_id: 10003,
                    filename: "Interpreter.zip".to_string(),
                    command: "javac /code/Interpreter.java && java -cp /code Interpreter".to_string(),
                    bytes: gatlam_content,
                }
            ).await?;

            Ok(())
        })
    }
}

/// Dummy zip file
fn create_example_interpreter_zip() -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buf);
        let options = SimpleFileOptions::default().unix_permissions(0o644);

        let dummy_content = r#"// Dummy interpreter content"#;
        zip.start_file("interpreter.cpp", options).unwrap();
        zip.write_all(dummy_content.as_bytes()).unwrap();
        zip.finish().unwrap();
    }
    buf.into_inner()
}

fn create_interpreter_zip_cpp() -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buf);
        let options = SimpleFileOptions::default().unix_permissions(0o644);

        let interpreter_cpp = r##"
#include <iostream>
#include <vector>
#include <string>
#include <cstdlib>
#include <ctime>

std::string mapToFunction(char digit) {
    switch (digit) {
        case '0': return "HelperOne::subtaskA()";
        case '1': return "HelperTwo::subtaskB()";
        case '2': return "HelperThree::subtaskC()";
        case '3': return "HelperTwo::subtaskX()";
        case '4': return "HelperThree::subtaskY()";
        case '5': return "HelperOne::subtaskZ()";
        case '6': return "HelperThree::subtaskAlpha()";
        case '7': return "HelperOne::subtaskBeta()";
        case '8': return "HelperTwo::subtaskGamma()";
        default: return "INVALID()";
    }
}

std::string randomSubtaskName(const std::string& task, int index) {
    return task + "Subtask" + std::to_string(index + 1);
}

void writeTask(std::ostream& out, const std::string& taskName, const std::vector<std::string>& calls) {
    out << "static void run" << taskName << "() {\n";
    for (size_t i = 0; i < calls.size(); ++i) {
        out << "    std::cout << \"&-=-&" << randomSubtaskName(taskName, i) << "\" << std::endl;\n";
        out << "    std::cout << " << calls[i] << " << std::endl;\n";
    }
    out << "}\n\n";
}

int main(int argc, char* argv[]) {
    if (argc < 2) {
        std::cerr << "Usage: ./interpreter <digit_string>\n";
        return 1;
    }

    std::string input = argv[1];
    std::vector<std::string> task1, task2, task3;

    std::srand(static_cast<unsigned>(std::time(0)));

    // Assign up to 3 digits to each task in order
    for (size_t i = 0; i < input.size(); ++i) {
        std::string call = mapToFunction(input[i]);
        if (call == "INVALID()") continue;

        if (task1.size() < 3) task1.push_back(call);
        else if (task2.size() < 3) task2.push_back(call);
        else if (task3.size() < 3) task3.push_back(call);
        else break; // ignore extra input beyond 9 valid digits
    }

    // Print the generated main.cpp content to stdout

    std::cout << "#include <iostream>\n\n";

    std::cout << "class HelperOne {\n"
              << "public:\n"
              << "    static std::string subtaskA() { return \"HelperOne::subtaskA\"; }\n"
              << "    static std::string subtaskZ() { return \"HelperOne::subtaskZ\"; }\n"
              << "    static std::string subtaskBeta() { return \"HelperOne::subtaskBeta\"; }\n"
              << "};\n\n";

    std::cout << "class HelperTwo {\n"
              << "public:\n"
              << "    static std::string subtaskB() { return \"HelperTwo::subtaskB\"; }\n"
              << "    static std::string subtaskX() { return \"HelperTwo::subtaskX\"; }\n"
              << "    static std::string subtaskGamma() { return \"HelperTwo::subtaskGamma\"; }\n"
              << "};\n\n";

    std::cout << "class HelperThree {\n"
              << "public:\n"
              << "    static std::string subtaskC() { return \"HelperThree::subtaskC\"; }\n"
              << "    static std::string subtaskY() { return \"HelperThree::subtaskY\"; }\n"
              << "    static std::string subtaskAlpha() { return \"HelperThree::subtaskAlpha\"; }\n"
              << "};\n\n";

    // Declare runtask functions before main()
    std::cout << "static void runtask1();\n";
    std::cout << "static void runtask2();\n";
    std::cout << "static void runtask3();\n\n";

    std::cout << "int main(int argc, char* argv[]) {\n";
    std::cout << "    std::string task = argc > 1 ? argv[1] : \"task1\";\n";
    std::cout << "    if (task == \"task1\") runtask1();\n";
    std::cout << "    else if (task == \"task2\") runtask2();\n";
    std::cout << "    else if (task == \"task3\") runtask3();\n";
    std::cout << "    else std::cout << task << \" is not a valid task\" << std::endl;\n";
    std::cout << "    return 0;\n";
    std::cout << "}\n\n";

    // Define runtask functions after main()
    writeTask(std::cout, "task1", task1);
    writeTask(std::cout, "task2", task2);
    writeTask(std::cout, "task3", task3);

    return 0;
}
"##;

        zip.start_file("interpreter.cpp", options).unwrap();
        zip.write_all(interpreter_cpp.as_bytes()).unwrap();
        zip.finish().unwrap();
    }
    buf.into_inner()
}

//GATALM
fn create_gatlam_interpreter_zip() -> Vec<u8> {
    let mut buf = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buf);
        let options = SimpleFileOptions::default().unix_permissions(0o644);

        let main_java = r#"
import java.util.*;

public class Interpreter {
    
    public static String mapToFunction(String digit) {
        switch (digit) {
            case "0": return "solution.fibonacci_U1(5)";
            case "1": return "solution.factorial_U1(4)";
            case "2": return "solution.sumArray_U1(new int[]{1,2,3,4,5})";
            case "3": return "\"Grade: \" + solution.gradeStudent(85)";
            case "-1": return "solution.fibonacci_U1(8)";
            case "-2": return "solution.factorial_U1(6)";
            case "-3": return "solution.sumArray_U1(new int[]{10,20,30})";
            case "-4": return "\"Grade: \" + solution.gradeStudent(92)";
            default: return null; // we'll handle invalid below
        }
    }
    
    public static String getDefaultFunction(int index) {
        // always returns a function based on index % 4
        switch (index % 4) {
            case 0: return "solution.fibonacci_U1(5)";
            case 1: return "solution.factorial_U1(4)";
            case 2: return "solution.sumArray_U1(new int[]{1,2,3,4,5})";
            case 3: return "\"Grade: \" + solution.gradeStudent(85)";
            default: return "solution.fibonacci_U1(1)"; // fallback
        }
    }
    
    public static String randomSubtaskName(String task, int index) {
        return task + "Subtask" + (index + 1);
    }
    
    public static void writeTask(String taskName, List<String> calls) {
        System.out.println("    public static void run" + taskName + "() {");
        System.out.println("        StudentSolution solution = new StudentSolution();");
        for (int i = 0; i < calls.size(); i++) {
            System.out.println("        System.out.println(\"&-=-&" + randomSubtaskName(taskName, i) + "\");");
            System.out.println("        System.out.println(" + calls.get(i) + ");");
        }
        System.out.println("    }");
        System.out.println();
    }
    
    public static void main(String[] args) {
        if (args.length < 1) {
            System.err.println("Usage: java Interpreter <digit_string>");
            return;
        }
        
        String input = args[0];
        
        List<String> task1 = new ArrayList<>();
        
        // Parse the comma-separated input
        String[] digits = input.split(",");
        
        int count = 0;
        for (String digit : digits) {
            digit = digit.trim();
            String call = mapToFunction(digit);
            if (call == null) {
                // if invalid, pick a default based on count
                call = getDefaultFunction(count);
            }
            task1.add(call);
            count++;
            if (task1.size() >= 3) break; // max 3 calls
        }
        
        // If still empty, force at least one call
        if (task1.isEmpty()) {
            task1.add(getDefaultFunction(0));
        }
        
        // Print the generated Main.java to stdout
        System.out.println("public class Main {");
        System.out.println();
        
        System.out.println("    public static void main(String[] args) {");
        System.out.println("        runtask1();");
        System.out.println("    }");
        System.out.println();
        
        // Write only task1
        writeTask("task1", task1);
        
        System.out.println("}");
    }
}
"#;

        zip.start_file("Interpreter.java", options).unwrap();
        zip.write_all(main_java.as_bytes()).unwrap();
        zip.finish().unwrap();
    }
    buf.into_inner()
}
