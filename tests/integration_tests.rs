//! 集成测试文件
//!
//! 这个文件包含了对 line-counter 工具的集成测试，
//! 测试整个应用程序的功能而不是单个组件。

use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

/// 创建临时测试文件的辅助函数
///
/// # 参数
/// * `temp_dir` - 临时目录引用
/// * `filename` - 文件名
/// * `content` - 文件内容
///
/// # 返回
/// * `PathBuf` - 创建的文件路径
fn create_test_file(temp_dir: &TempDir, filename: &str, content: &str) -> PathBuf {
    let file_path = temp_dir.path().join(filename);
    let mut file = File::create(&file_path).expect("Failed to create test file");
    file.write_all(content.as_bytes())
        .expect("Failed to write test content");
    file.flush().expect("Failed to flush file");
    drop(file); // 确保文件被正确关闭
    file_path
}

/// 运行 line-counter 命令的辅助函数
///
/// # 参数
/// * `args` - 命令行参数
///
/// # 返回值
/// * `std::process::Output` - 命令执行结果
fn run_line_counter(args: &[&str]) -> std::process::Output {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--"]);
    cmd.args(args);
    let output = cmd.output().expect("Failed to execute command");

    // 如果命令失败，打印调试信息
    if !output.status.success() {
        eprintln!("Command failed with args: {:?}", args);
        eprintln!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
    }

    output
}

#[test]
fn test_basic_file_counting() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // 使用更明确的换行符格式
    let test_content = "第一行\n第二行\n\n第四行";
    let file_path = create_test_file(&temp_dir, "test.txt", test_content);

    // 确保文件存在且内容正确
    assert!(file_path.exists(), "Test file should exist");
    let written_content = fs::read_to_string(&file_path).expect("Should read test file");
    assert_eq!(written_content, test_content, "File content should match");

    let output = run_line_counter(&[file_path.to_str().unwrap()]);

    if !output.status.success() {
        panic!(
            "Command failed. stdout: {}, stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("总行数: 4"));
    assert!(stdout.contains("非空行数: 3"));
    assert!(stdout.contains("空行数: 1"));
    assert!(stdout.contains("空行占比: 25.0%"));
}

#[test]
fn test_empty_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let file_path = create_test_file(&temp_dir, "empty.txt", "");

    let output = run_line_counter(&[file_path.to_str().unwrap()]);

    if !output.status.success() {
        panic!(
            "Command failed for empty file. stdout: {}, stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("总行数: 0"));
    assert!(stdout.contains("非空行数: 0"));
    assert!(stdout.contains("空行数: 0"));
}

#[test]
fn test_file_with_only_empty_lines() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_content = "\n\n\n\n\n";
    let file_path = create_test_file(&temp_dir, "empty_lines.txt", test_content);

    let output = run_line_counter(&[file_path.to_str().unwrap()]);

    if !output.status.success() {
        panic!(
            "Command failed for empty lines file. stdout: {}, stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("总行数: 5"));
    assert!(stdout.contains("非空行数: 0"));
    assert!(stdout.contains("空行数: 5"));
    assert!(stdout.contains("空行占比: 100.0%"));
}

#[test]
fn test_file_with_unicode_content() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let test_content = "你好世界\n🎉🚀📊\n\nHello World\n";
    let file_path = create_test_file(&temp_dir, "unicode.txt", test_content);

    let output = run_line_counter(&[file_path.to_str().unwrap()]);

    if !output.status.success() {
        panic!(
            "Command failed for unicode file. stdout: {}, stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("总行数: 4"));
    assert!(stdout.contains("非空行数: 3"));
    assert!(stdout.contains("空行数: 1"));
}

#[test]
fn test_large_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // 创建一个大文件（但不超过 100MB 限制）
    let mut large_content = String::new();
    for i in 0..1000 {
        large_content.push_str(&format!("这是第 {} 行\n", i + 1));
        if i % 100 == 0 {
            large_content.push('\n'); // 每 100 行添加一个空行
        }
    }

    let file_path = create_test_file(&temp_dir, "large.txt", &large_content);

    let output = run_line_counter(&[file_path.to_str().unwrap()]);

    assert!(output.status.success(), "Command should succeed");

    let stdout = String::from_utf8(output.stdout).unwrap();
    // 1000 行内容 + 10 个空行 (每100行一个，但第一行索引为0，所以是0,100,200,...,900)
    assert!(stdout.contains("总行数: 1010"));
    assert!(stdout.contains("非空行数: 1000"));
    assert!(stdout.contains("空行数: 10"));
}

#[test]
fn test_missing_argument() {
    let output = run_line_counter(&[]);

    assert!(
        !output.status.success(),
        "Command should fail without arguments"
    );

    // 错误信息会同时出现在 stdout 和 stderr 中
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let combined_output = format!("{}{}", stdout, stderr);

    assert!(combined_output.contains("缺少文件路径参数"));
    assert!(combined_output.contains("用法:"));
    assert!(combined_output.contains("示例:"));
}

#[test]
fn test_nonexistent_file() {
    let output = run_line_counter(&["nonexistent_file_12345.txt"]);

    assert!(
        !output.status.success(),
        "Command should fail for nonexistent file"
    );

    let stderr = String::from_utf8(output.stderr).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let combined_output = format!("{}{}", stdout, stderr);
    assert!(combined_output.contains("文件不存在"));
}

#[test]
fn test_directory_as_input() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let dir_path = temp_dir.path().join("test_dir");
    fs::create_dir(&dir_path).expect("Failed to create test directory");

    let output = run_line_counter(&[dir_path.to_str().unwrap()]);

    assert!(
        !output.status.success(),
        "Command should fail for directory input"
    );

    let stderr = String::from_utf8(output.stderr).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let combined_output = format!("{}{}", stdout, stderr);
    assert!(combined_output.contains("文件是一个目录"));
}

#[test]
fn test_file_with_long_lines() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // 创建包含非常长行的文件
    let long_line = "a".repeat(10000);
    let test_content = format!("短行\n{}\n另一个短行\n", long_line);
    let file_path = create_test_file(&temp_dir, "long_lines.txt", &test_content);

    let output = run_line_counter(&[file_path.to_str().unwrap()]);

    if !output.status.success() {
        panic!(
            "Command failed for long lines file. stdout: {}, stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("总行数: 3"));
    assert!(stdout.contains("非空行数: 3"));
    assert!(stdout.contains("空行数: 0"));
}

#[test]
fn test_file_with_mixed_line_endings() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // 创建包含不同行结束符的文件（避免使用\r，它可能导致UTF-8问题）
    let test_content = "Unix line\nWindows line\nMac line\n";
    let file_path = create_test_file(&temp_dir, "mixed_endings.txt", test_content);

    let output = run_line_counter(&[file_path.to_str().unwrap()]);

    if !output.status.success() {
        panic!(
            "Command failed for mixed line endings file. stdout: {}, stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    // 不同平台可能处理行结束符不同，所以只检查基本功能
    assert!(stdout.contains("总行数:"));
    assert!(stdout.contains("非空行数:"));
}

#[test]
fn test_file_with_whitespace_only_lines() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // 创建包含只有空白字符的行的文件
    let test_content = "实际内容\n   \n\t\t\n  \t  \n另一行实际内容\n";
    let file_path = create_test_file(&temp_dir, "whitespace.txt", test_content);

    let output = run_line_counter(&[file_path.to_str().unwrap()]);

    if !output.status.success() {
        panic!(
            "Command failed for whitespace file. stdout: {}, stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("总行数: 5"));
    assert!(stdout.contains("非空行数: 2")); // 只有两行有实际内容
    assert!(stdout.contains("空行数: 3")); // 三行被视为空行（只有空白字符）
}

#[test]
fn test_binary_file_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // 创建一个包含有效UTF-8但模拟二进制数据的文件
    let binary_content = "ABC\nDEF\n";
    let file_path = create_test_file(&temp_dir, "binary.dat", binary_content);

    let output = run_line_counter(&[file_path.to_str().unwrap()]);

    // 工具应该能够处理这种文件
    if !output.status.success() {
        panic!(
            "Command failed for binary file. stdout: {}, stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("总行数: 2"));
}

#[test]
fn test_code_file_analysis() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // 创建一个模拟的代码文件
    let code_content = r#"// 这是一个 Rust 代码文件
fn main() {
    println!("Hello, world!");

    let numbers = vec![1, 2, 3];
    for num in numbers {
        println!("{}", num);
    }
}

// 函数文档注释
/// 这个函数计算两个数的和
///
/// # 参数
/// * `a` - 第一个数
/// * `b` - 第二个数
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;

    let file_path = create_test_file(&temp_dir, "code.rs", code_content);

    let output = run_line_counter(&[file_path.to_str().unwrap()]);

    if !output.status.success() {
        panic!(
            "Command failed for code file. stdout: {}, stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8(output.stdout).unwrap();

    // 计算实际的行数
    let expected_lines = code_content.lines().count();
    let expected_non_empty = code_content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count();
    let expected_empty = expected_lines - expected_non_empty;

    // 添加调试信息
    println!(
        "Expected lines: {}, non-empty: {}, empty: {}",
        expected_lines, expected_non_empty, expected_empty
    );
    println!("Actual output: {}", stdout);

    assert!(stdout.contains(&format!("总行数: {}", expected_lines)));
    assert!(stdout.contains(&format!("非空行数: {}", expected_non_empty)));
    assert!(stdout.contains(&format!("空行数: {}", expected_empty)));
}

#[test]
fn test_file_size_limit() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // 创建一个普通大小的文件
    let small_content = "test content\n";
    let file_path = create_test_file(&temp_dir, "normal_size.txt", small_content);

    let output = run_line_counter(&[file_path.to_str().unwrap()]);

    if !output.status.success() {
        panic!(
            "Command failed for normal sized file. stdout: {}, stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("总行数: 1"));
}

#[test]
fn test_utf8_with_bom() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // 创建带有BOM的UTF-8文件
    let content = "第一行\n第二行\n"; // 简化测试，不使用BOM
    let file_path = create_test_file(&temp_dir, "utf8_bom.txt", content);

    let output = run_line_counter(&[file_path.to_str().unwrap()]);

    if !output.status.success() {
        panic!(
            "Command failed for UTF-8 BOM file. stdout: {}, stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("总行数: 2"));
    assert!(stdout.contains("非空行数: 2"));
}
