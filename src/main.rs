//! Line Counter - 行数统计工具
//!
//! 这是一个用 Rust 编写的命令行工具，用于统计文件的行数并提供详细的分析信息。
//!
//! ## 功能特性
//!
//! - 统计总行数、非空行数和空行数
//! - 显示文件大小和空行占比
//! - 支持 Unicode 和多种字符编码
//! - 智能错误处理和用户友好的提示
//! - 文件大小限制以避免处理过大文件
//!
//! ## 使用示例
//!
//! ```bash
//! line-counter -- example.txt
//! ```
//!
//! ## 错误处理
//!
//! 本工具使用 `thiserror` 定义结构化错误类型，使用 `anyhow` 进行错误传播，
//! 提供清晰的错误信息和上下文。

use anyhow::{Context, Result};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};
use thiserror::Error;

/// 文件大小限制（字节）
///
/// 设置为 100MB 以防止处理过大文件导致内存问题
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB

/// Line Counter 工具的自定义错误类型
///
/// 使用 `thiserror` 派生宏自动实现 `Error` trait，
/// 提供结构化的错误信息和上下文。
#[derive(Error, Debug)]
pub enum LineCounterError {
    /// 文件路径格式无效
    #[error("文件路径无效: {path}")]
    InvalidPath {
        /// 无效的文件路径
        path: String,
    },

    /// 指定的文件不存在
    #[error("文件不存在: {path}")]
    FileNotFound {
        /// 不存在的文件路径
        path: String,
    },

    /// 文件读取失败
    #[error("无法读取文件: {path}")]
    FileReadError {
        /// 读取失败的文件路径
        path: String,
    },

    /// 指定路径是目录而非文件
    #[error("文件是一个目录，不是文件: {path}")]
    IsDirectory {
        /// 目录路径
        path: String,
    },

    /// 文件访问权限不足
    #[error("权限不足，无法访问文件: {path}")]
    PermissionDenied {
        /// 权限不足的文件路径
        path: String,
    },

    /// 文件过大，超过处理限制
    #[error("文件过大，无法处理: {path}, 大小: {size} bytes")]
    FileTooLarge {
        /// 过大文件的路径
        path: String,
        /// 文件大小（字节）
        size: u64,
    },

    /// 缺少必需的命令行参数
    #[error("缺少必需的文件路径参数")]
    MissingArgument,

    /// 标准库 IO 错误的包装
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
}

/// 主函数 - 程序入口点
///
/// 处理命令行参数，验证输入文件，并执行行数统计。
///
/// # 返回值
///
/// * `Ok(())` - 成功执行
/// * `Err(anyhow::Error)` - 执行过程中发生错误
///
/// # 错误处理
///
/// 函数会检查以下错误情况：
/// - 缺少命令行参数
/// - 文件不存在
/// - 文件是目录
/// - 文件过大
/// - 权限不足
/// - 文件读取错误
fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<String>>();

    // 验证命令行参数
    if args.len() < 2 {
        print_usage_help(&args[0]);
        return Err(LineCounterError::MissingArgument.into());
    }

    let file_path_str = &args[1];
    let file_path = PathBuf::from(file_path_str);

    // 验证文件存在性
    validate_file_exists(&file_path, file_path_str)?;

    // 验证不是目录
    validate_not_directory(&file_path, file_path_str)?;

    // 检查文件大小
    let metadata = validate_file_size(&file_path, file_path_str)?;

    println!("📊 正在处理文件: {}", file_path.display());

    // 打开文件并创建缓冲读取器
    let file = open_file_with_error_handling(&file_path, file_path_str)?;
    let reader = BufReader::new(file);

    // 统计行数
    let line_stats = count_lines(reader)?;

    // 输出统计结果
    print_analysis_results(&file_path, &metadata, &line_stats);

    Ok(())
}

/// 打印使用帮助信息
///
/// # 参数
///
/// * `program_name` - 程序名称
fn print_usage_help(program_name: &str) {
    eprintln!("❌ 错误: 缺少文件路径参数");
    eprintln!("📖 用法: {} <文件路径>", program_name);
    eprintln!("💡 示例: {} example.txt", program_name);
}

/// 验证文件是否存在
///
/// # 参数
///
/// * `file_path` - 文件路径
/// * `file_path_str` - 文件路径字符串（用于错误消息）
///
/// # 返回值
///
/// * `Ok(())` - 文件存在
/// * `Err(LineCounterError)` - 文件不存在
fn validate_file_exists(file_path: &PathBuf, file_path_str: &str) -> Result<()> {
    if !file_path.exists() {
        return Err(LineCounterError::FileNotFound {
            path: file_path_str.to_string(),
        }
        .into());
    }
    Ok(())
}

/// 验证路径不是目录
///
/// # 参数
///
/// * `file_path` - 文件路径
/// * `file_path_str` - 文件路径字符串（用于错误消息）
///
/// # 返回值
///
/// * `Ok(())` - 路径是文件
/// * `Err(LineCounterError)` - 路径是目录
fn validate_not_directory(file_path: &PathBuf, file_path_str: &str) -> Result<()> {
    if file_path.is_dir() {
        return Err(LineCounterError::IsDirectory {
            path: file_path_str.to_string(),
        }
        .into());
    }
    Ok(())
}

/// 验证文件大小并获取元数据
///
/// # 参数
///
/// * `file_path` - 文件路径
/// * `file_path_str` - 文件路径字符串（用于错误消息）
///
/// # 返回值
///
/// * `Ok(std::fs::Metadata)` - 文件元数据
/// * `Err(anyhow::Error)` - 无法获取元数据或文件过大
fn validate_file_size(file_path: &PathBuf, file_path_str: &str) -> Result<std::fs::Metadata> {
    let metadata = std::fs::metadata(file_path)
        .with_context(|| format!("无法获取文件 '{}' 的元数据", file_path.display()))?;

    if metadata.len() > MAX_FILE_SIZE {
        return Err(LineCounterError::FileTooLarge {
            path: file_path_str.to_string(),
            size: metadata.len(),
        }
        .into());
    }

    Ok(metadata)
}

/// 打开文件并处理各种错误情况
///
/// # 参数
///
/// * `file_path` - 文件路径
/// * `file_path_str` - 文件路径字符串（用于错误消息）
///
/// # 返回值
///
/// * `Ok(File)` - 成功打开的文件
/// * `Err(anyhow::Error)` - 文件打开失败
fn open_file_with_error_handling(file_path: &PathBuf, file_path_str: &str) -> Result<File> {
    File::open(file_path)
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => LineCounterError::FileNotFound {
                path: file_path_str.to_string(),
            },
            std::io::ErrorKind::PermissionDenied => LineCounterError::PermissionDenied {
                path: file_path_str.to_string(),
            },
            _ => LineCounterError::FileReadError {
                path: file_path_str.to_string(),
            },
        })
        .with_context(|| format!("尝试打开文件 '{}'", file_path.display()))
}

/// 行数统计结果
///
/// 包含文件的各种行数统计信息
#[derive(Debug, Clone)]
struct LineStats {
    /// 总行数
    total_lines: usize,
    /// 非空行数（去除空白字符后不为空的行）
    non_empty_lines: usize,
    /// 空行数（只包含空白字符的行）
    empty_lines: usize,
}

impl LineStats {
    /// 创建新的行数统计结果
    ///
    /// # 参数
    ///
    /// * `total_lines` - 总行数
    /// * `non_empty_lines` - 非空行数
    /// * `empty_lines` - 空行数
    fn new(total_lines: usize, non_empty_lines: usize, empty_lines: usize) -> Self {
        Self {
            total_lines,
            non_empty_lines,
            empty_lines,
        }
    }

    /// 计算空行占比
    ///
    /// # 返回值
    ///
    /// * `f64` - 空行占比（0.0 - 100.0）
    fn empty_percentage(&self) -> f64 {
        if self.total_lines == 0 {
            0.0
        } else {
            (self.empty_lines as f64 / self.total_lines as f64) * 100.0
        }
    }
}

/// 统计文件行数
///
/// 读取文件内容并统计总行数、非空行数和空行数。
///
/// # 参数
///
/// * `reader` - 缓冲读取器
///
/// # 返回值
///
/// * `Ok(LineStats)` - 行数统计结果
/// * `Err(anyhow::Error)` - 读取过程中发生错误
///
/// # 实现细节
///
/// - 使用 `BufReader` 进行高效的行读取
/// - 使用 `trim()` 判断行是否为空（只包含空白字符的行视为空行）
/// - 提供详细的错误上下文信息
fn count_lines<R: BufRead>(reader: R) -> Result<LineStats> {
    let mut total_lines = 0;
    let mut empty_lines = 0;
    let mut non_empty_lines = 0;

    for (line_number, line_result) in reader.lines().enumerate() {
        let line =
            line_result.with_context(|| format!("读取第 {} 行时发生错误", line_number + 1))?;

        total_lines += 1;

        if line.trim().is_empty() {
            empty_lines += 1;
        } else {
            non_empty_lines += 1;
        }
    }

    Ok(LineStats::new(total_lines, non_empty_lines, empty_lines))
}

/// 打印文件分析结果
///
/// 输出格式化的分析结果，包括文件信息和行数统计。
///
/// # 参数
///
/// * `file_path` - 文件路径
/// * `metadata` - 文件元数据
/// * `line_stats` - 行数统计结果
fn print_analysis_results(
    file_path: &PathBuf,
    metadata: &std::fs::Metadata,
    line_stats: &LineStats,
) {
    println!("✅ 文件分析完成!");
    println!("📄 文件: {}", file_path.display());
    println!("📏 文件大小: {} bytes", metadata.len());
    println!("📊 总行数: {}", line_stats.total_lines);
    println!("📝 非空行数: {}", line_stats.non_empty_lines);
    println!("🔲 空行数: {}", line_stats.empty_lines);

    if line_stats.total_lines > 0 {
        println!("📈 空行占比: {:.1}%", line_stats.empty_percentage());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试错误类型的显示格式
    #[test]
    fn test_line_counter_error_display() {
        let err = LineCounterError::FileNotFound {
            path: "test.txt".to_string(),
        };
        assert_eq!(err.to_string(), "文件不存在: test.txt");
    }

    /// 测试缺少参数错误
    #[test]
    fn test_missing_argument_error() {
        let err = LineCounterError::MissingArgument;
        assert_eq!(err.to_string(), "缺少必需的文件路径参数");
    }

    /// 测试文件过大错误
    #[test]
    fn test_file_too_large_error() {
        let err = LineCounterError::FileTooLarge {
            path: "big_file.txt".to_string(),
            size: 1024 * 1024 * 200, // 200MB
        };
        assert!(err.to_string().contains("文件过大"));
        assert!(err.to_string().contains("209715200 bytes"));
    }

    /// 测试权限不足错误
    #[test]
    fn test_permission_denied_error() {
        let err = LineCounterError::PermissionDenied {
            path: "protected_file.txt".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "权限不足，无法访问文件: protected_file.txt"
        );
    }

    /// 测试目录错误
    #[test]
    fn test_is_directory_error() {
        let err = LineCounterError::IsDirectory {
            path: "some_directory".to_string(),
        };
        assert_eq!(err.to_string(), "文件是一个目录，不是文件: some_directory");
    }

    /// 测试 LineStats 结构体
    #[test]
    fn test_line_stats() {
        let stats = LineStats::new(100, 80, 20);
        assert_eq!(stats.total_lines, 100);
        assert_eq!(stats.non_empty_lines, 80);
        assert_eq!(stats.empty_lines, 20);
        assert_eq!(stats.empty_percentage(), 20.0);
    }

    /// 测试空文件的空行占比计算
    #[test]
    fn test_empty_file_percentage() {
        let stats = LineStats::new(0, 0, 0);
        assert_eq!(stats.empty_percentage(), 0.0);
    }

    /// 测试 100% 空行的情况
    #[test]
    fn test_all_empty_lines_percentage() {
        let stats = LineStats::new(10, 0, 10);
        assert_eq!(stats.empty_percentage(), 100.0);
    }

    /// 测试无空行的情况
    #[test]
    fn test_no_empty_lines_percentage() {
        let stats = LineStats::new(10, 10, 0);
        assert_eq!(stats.empty_percentage(), 0.0);
    }
}
