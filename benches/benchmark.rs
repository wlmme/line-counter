//! 基准测试文件
//!
//! 测试 Line Counter 工具在不同文件大小和内容类型下的性能。

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::fs::File;
use std::io::{BufRead, BufReader};
use tempfile::NamedTempFile;

/// 模拟 line-counter 的核心统计功能
fn count_lines_core<R: BufRead>(reader: R) -> Result<(usize, usize, usize), std::io::Error> {
    let mut total_lines = 0;
    let mut empty_lines = 0;
    let mut non_empty_lines = 0;

    for line_result in reader.lines() {
        let line = line_result?;
        total_lines += 1;

        if line.trim().is_empty() {
            empty_lines += 1;
        } else {
            non_empty_lines += 1;
        }
    }

    Ok((total_lines, non_empty_lines, empty_lines))
}

/// 创建测试文件的辅助函数
fn create_test_file(content: &str) -> NamedTempFile {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    std::io::Write::write_all(&mut temp_file, content.as_bytes())
        .expect("Failed to write test content");
    temp_file
}

/// 基准测试：小文件（100行）
fn bench_small_file(c: &mut Criterion) {
    let content = (0..100)
        .map(|i| format!("这是第 {} 行", i))
        .collect::<Vec<_>>()
        .join("\n");

    let temp_file = create_test_file(&content);
    let file_path = temp_file.path();

    c.bench_function("small_file_100_lines", |b| {
        b.iter(|| {
            let file = File::open(black_box(file_path)).unwrap();
            let reader = BufReader::new(file);
            count_lines_core(reader).unwrap()
        })
    });
}

/// 基准测试：中等文件（10,000行）
fn bench_medium_file(c: &mut Criterion) {
    let content = (0..10000)
        .map(|i| {
            if i % 10 == 0 {
                String::new() // 每10行插入一个空行
            } else {
                format!("这是第 {} 行，包含一些中文内容", i)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let temp_file = create_test_file(&content);
    let file_path = temp_file.path();

    c.bench_function("medium_file_10k_lines", |b| {
        b.iter(|| {
            let file = File::open(black_box(file_path)).unwrap();
            let reader = BufReader::new(file);
            count_lines_core(reader).unwrap()
        })
    });
}

/// 基准测试：大文件（100,000行）
fn bench_large_file(c: &mut Criterion) {
    let content = (0..100000)
        .map(|i| {
            if i % 20 == 0 {
                String::new() // 每20行插入一个空行
            } else {
                format!("Line {} with some content and unicode: 📊🎉", i)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let temp_file = create_test_file(&content);
    let file_path = temp_file.path();

    c.bench_function("large_file_100k_lines", |b| {
        b.iter(|| {
            let file = File::open(black_box(file_path)).unwrap();
            let reader = BufReader::new(file);
            count_lines_core(reader).unwrap()
        })
    });
}

/// 基准测试：长行文件
fn bench_long_lines_file(c: &mut Criterion) {
    let long_line = "a".repeat(1000);
    let content = (0..1000)
        .map(|i| {
            if i % 5 == 0 {
                format!("短行 {}", i)
            } else {
                format!("长行 {}: {}", i, long_line)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let temp_file = create_test_file(&content);
    let file_path = temp_file.path();

    c.bench_function("long_lines_file_1k_lines", |b| {
        b.iter(|| {
            let file = File::open(black_box(file_path)).unwrap();
            let reader = BufReader::new(file);
            count_lines_core(reader).unwrap()
        })
    });
}

/// 基准测试：只有空行的文件
fn bench_empty_lines_file(c: &mut Criterion) {
    let content = "\n".repeat(50000);
    let temp_file = create_test_file(&content);
    let file_path = temp_file.path();

    c.bench_function("empty_lines_file_50k_lines", |b| {
        b.iter(|| {
            let file = File::open(black_box(file_path)).unwrap();
            let reader = BufReader::new(file);
            count_lines_core(reader).unwrap()
        })
    });
}

/// 基准测试：混合 Unicode 内容
fn bench_unicode_file(c: &mut Criterion) {
    let unicode_content = vec![
        "Hello World",
        "你好世界",
        "🎉🚀📊✅❌",
        "Здравствуй мир",
        "مرحبا بالعالم",
        "こんにちは世界",
        "🌍🌎🌏",
        "",
        "Mixed content with émojis 🎯",
        "数字：１２３４５６７８９０",
    ];

    let content = (0..5000)
        .map(|i| unicode_content[i % unicode_content.len()].to_string())
        .collect::<Vec<_>>()
        .join("\n");

    let temp_file = create_test_file(&content);
    let file_path = temp_file.path();

    c.bench_function("unicode_file_5k_lines", |b| {
        b.iter(|| {
            let file = File::open(black_box(file_path)).unwrap();
            let reader = BufReader::new(file);
            count_lines_core(reader).unwrap()
        })
    });
}

/// 基准测试：不同缓冲区大小的影响
fn bench_buffer_sizes(c: &mut Criterion) {
    let content = (0..10000)
        .map(|i| format!("Line {} with some content", i))
        .collect::<Vec<_>>()
        .join("\n");

    let temp_file = create_test_file(&content);
    let file_path = temp_file.path();

    // 默认缓冲区大小
    c.bench_function("buffer_default", |b| {
        b.iter(|| {
            let file = File::open(black_box(file_path)).unwrap();
            let reader = BufReader::new(file);
            count_lines_core(reader).unwrap()
        })
    });

    // 小缓冲区 (1KB)
    c.bench_function("buffer_1kb", |b| {
        b.iter(|| {
            let file = File::open(black_box(file_path)).unwrap();
            let reader = BufReader::with_capacity(1024, file);
            count_lines_core(reader).unwrap()
        })
    });

    // 大缓冲区 (64KB)
    c.bench_function("buffer_64kb", |b| {
        b.iter(|| {
            let file = File::open(black_box(file_path)).unwrap();
            let reader = BufReader::with_capacity(64 * 1024, file);
            count_lines_core(reader).unwrap()
        })
    });
}

criterion_group!(
    benches,
    bench_small_file,
    bench_medium_file,
    bench_large_file,
    bench_long_lines_file,
    bench_empty_lines_file,
    bench_unicode_file,
    bench_buffer_sizes
);

criterion_main!(benches);
