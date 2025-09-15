use serde::{Deserialize, Serialize};

/// All MOSS-supported languages with cleaner Rust-y names.
/// Serialized/deserialized in `lowercase` for config JSON.
/// Common aliases are accepted (e.g., "cc", "c++", "js", "c#").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// Not supported by MOSS, but supported in our runtime and starters.
    /// Serialized as "rust".
    Rust,
    /// Not supported by MOSS, but supported in our runtime and starters.
    /// Serialized as "go".
    #[serde(alias = "golang")]
    Go,
    C,                                  // "c"
    #[serde(alias = "cc", alias = "c++")]
    Cpp,                                // maps to MOSS "cc"
    Java,                               // "java"
    Ml,                                 // "ml" (OCaml/SML)
    Pascal,                             // "pascal"
    Ada,                                // "ada"
    Lisp,                               // "lisp"
    Scheme,                             // "scheme"
    Haskell,                            // "haskell"
    Fortran,                            // "fortran"
    Ascii,                              // "ascii"
    Vhdl,                               // "vhdl"
    Perl,                               // "perl"
    Matlab,                             // "matlab"
    Python,                             // "python"
    Mips,                               // "mips"
    Prolog,                             // "prolog"
    Spice,                              // "spice"
    Vb,                                 // "vb"
    #[serde(alias = "c#")]
    CSharp,                             // "csharp"
    Modula2,                            // "modula2"
    A8086,                              // "a8086"
    #[serde(alias = "js")]
    JavaScript,                         // "javascript"
    PlSql,                              // "plsql"
}

impl Language {
    /// Exact MOSS language string required by the service.
    pub fn to_moss(self) -> &'static str {
        match self {
            // MOSS does not have native Rust support. Use ascii fallback when needed.
            Language::Rust     => "ascii",
            // MOSS does not have native Go support either; use ascii.
            Language::Go       => "ascii",
            Language::C         => "c",
            Language::Cpp       => "cc",
            Language::Java      => "java",
            Language::Ml        => "ml",
            Language::Pascal    => "pascal",
            Language::Ada       => "ada",
            Language::Lisp      => "lisp",
            Language::Scheme    => "scheme",
            Language::Haskell   => "haskell",
            Language::Fortran   => "fortran",
            Language::Ascii     => "ascii",
            Language::Vhdl      => "vhdl",
            Language::Perl      => "perl",
            Language::Matlab    => "matlab",
            Language::Python    => "python",
            Language::Mips      => "mips",
            Language::Prolog    => "prolog",
            Language::Spice     => "spice",
            Language::Vb        => "vb",
            Language::CSharp    => "csharp",
            Language::Modula2   => "modula2",
            Language::A8086     => "a8086",
            Language::JavaScript=> "javascript",
            Language::PlSql     => "plsql",
        }
    }
}

pub trait LanguageExt {
    /// e.g., "Main.cpp", "main.c", "Main.java", ...
    fn main_filename(&self) -> &'static str;

    /// Heuristic: does `cmd` look like a compile/run line for this language?
    fn is_compile_cmd(&self, cmd: &str) -> bool;

    /// Produce a tiny valid program that prints `s`. None if not supported.
    fn synthesize_program(&self, s: &str) -> Option<String>;

    /// Quick sniff test for generated source.
    fn looks_like_source(&self, text: &str) -> bool;
}

impl LanguageExt for Language {
    fn main_filename(&self) -> &'static str {
        match self {
            Language::Rust      => "main.rs",
            Language::Go        => "main.go",
            Language::C          => "main.c",
            Language::Cpp        => "Main.cpp",
            Language::Java       => "Main.java",
            Language::Ml         => "Main.ml",
            Language::Pascal     => "main.pas",
            Language::Ada        => "main.adb",
            Language::Lisp       => "main.lisp",
            Language::Scheme     => "main.scm",
            Language::Haskell    => "Main.hs",
            Language::Fortran    => "main.f90",
            Language::Ascii      => "input.txt",
            Language::Vhdl       => "main.vhdl",
            Language::Perl       => "main.pl",
            Language::Matlab     => "main.m",
            Language::Python     => "main.py",
            Language::Mips       => "main.s",
            Language::Prolog     => "main.pro", // .pl also common, but avoid Perl clash
            Language::Spice      => "main.sp",
            Language::Vb         => "Main.vb",
            Language::CSharp     => "Program.cs",
            Language::Modula2    => "Main.mod",
            Language::A8086      => "main.asm",
            Language::JavaScript => "main.js",
            Language::PlSql      => "main.sql",
        }
    }

    fn is_compile_cmd(&self, cmd: &str) -> bool {
        let c = cmd.to_lowercase();
        match self {
            Language::Rust => c.contains("cargo ") || c.contains("rustc "),
            Language::Go => c.contains("go build") || c.contains("go run"),
            Language::Cpp => c.contains("g++") || c.contains("clang++") || c.contains("c++"),
            Language::C   => c.contains("gcc") || c.contains("clang ") || c.contains("clang-"),
            Language::Java => c.contains("javac"),
            Language::Python => c.contains("python "),
            Language::JavaScript => c.contains("node ") || c.contains("deno ") || c.contains("nodejs"),
            Language::CSharp => c.contains("dotnet build") || c.contains("csc"),
            Language::Pascal => c.contains("fpc") || c.contains("gpc"),
            Language::Haskell => c.contains("ghc"),
            Language::Fortran => c.contains("gfortran") || c.contains("ifort"),
            Language::Perl => c.contains("perl "),
            Language::Matlab => c.contains("matlab") || c.contains("octave"),
            Language::Ml => c.contains("ocaml") || c.contains("sml"),
            Language::Lisp => c.contains("sbcl") || c.contains("clisp"),
            Language::Scheme => c.contains("racket") || c.contains("chez") || c.contains("guile"),
            Language::Mips => c.contains("spim") || c.contains("mars.jar"),
            Language::Prolog => c.contains("swipl") || c.contains("gprolog"),
            Language::Spice => c.contains("ngspice"),
            Language::Vhdl => c.contains("ghdl") || c.contains("modelsim"),
            Language::Vb => c.contains("vbc ") || c.contains("dotnet build"),
            Language::Modula2 => c.contains("gm2"),
            Language::A8086 => c.contains("nasm") || c.contains("masm"),
            Language::Ada => c.contains("gnatmake") || c.contains("gprbuild") || c.contains("gnatgcc"),
            Language::Ascii | Language::PlSql => false,
        }
    }

    fn synthesize_program(&self, s: &str) -> Option<String> {
        let msg = s.replace('"', "\\\"");
        Some(match self {
            Language::Rust => format!(
r#"fn main() {{
    println!("{}");
}}"#, msg),
            Language::Go => format!(
r#"package main
import "fmt"
func main() {{
    fmt.Println("{}")
}}"#, msg),
            Language::Cpp => format!(
r#"#include <bits/stdc++.h>
int main() {{
    std::cout << "{}" << std::endl;
    return 0;
}}"#, msg),
            Language::C => format!(
r#"#include <stdio.h>
int main() {{
    printf("{}\n");
    return 0;
}}"#, msg),
            Language::Java => format!(
r#"public class Main {{
    public static void main(String[] args) {{
        System.out.println("{}");
    }}
}}"#, msg),
            Language::Python => format!(r#"print("{}")"#, msg),
            Language::JavaScript => format!(r#"console.log("{}");"#, msg),
            Language::CSharp => format!(
r#"using System;
class Program {{
    static void Main() {{
        Console.WriteLine("{0}");
    }}
}}"#, msg),
            Language::Pascal => format!(
r#"program Main;
begin
    writeln('{0}');
end."#, msg.replace('\'', "''")),
            Language::Haskell => format!("main = putStrLn \"{}\"", msg),
            // For everything else, we don’t know the template → return None
            _ => return None,
        })
    }

    fn looks_like_source(&self, text: &str) -> bool {
        let t = text;
        match self {
            Language::Rust => t.contains("fn main()") || t.contains("mod ") || t.contains("use "),
            Language::Go => t.contains("package main") && t.contains("func main()"),
            Language::Cpp | Language::C => t.contains("#include") || t.contains("int main"),
            Language::Java => t.contains("class Main") || t.contains("public static void main"),
            Language::Python => t.contains("def ") || t.contains("print("),
            Language::JavaScript => t.contains("function ") || t.contains("console.log"),
            Language::CSharp => t.contains("class Program") || t.contains("static void Main"),
            Language::Pascal => t.contains("program ") || (t.contains("begin") && t.contains("end")),
            Language::Haskell => t.contains("main ="),
            Language::Perl => t.contains("use strict") || t.contains("print "),
            Language::Ml => t.contains("let ") || t.contains("fun "),
            Language::Lisp | Language::Scheme => t.contains('(') && t.contains(')'),
            Language::Fortran => t.to_lowercase().contains("program "),
            Language::Vhdl => {
                let tl = t.to_lowercase();
                tl.contains("entity ") || tl.contains("architecture ")
            }
            Language::Matlab => t.contains("function ") || t.contains("disp("),
            Language::Prolog => t.contains(":-"),
            Language::Mips => t.contains(".text") || t.contains(".globl"),
            Language::Spice => t.contains(".end") || t.contains(".model"),
            Language::Vb => t.contains("Module ") || t.contains("Sub Main"),
            Language::Modula2 => t.contains("MODULE ") || t.contains("BEGIN"),
            Language::A8086 => t.contains("mov ") || t.contains("section .text"),
            Language::Ada => t.contains("procedure ") || t.contains("with Ada.Text_IO"),
            Language::Ascii | Language::PlSql => !t.trim().is_empty(),
        }
    }

}
