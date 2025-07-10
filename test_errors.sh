#!/bin/bash

# 错误处理测试脚本
# 用于验证 line-counter 工具的各种错误情况

echo "🧪 开始测试 Line Counter 工具的错误处理..."
echo

# 构建项目
echo "📦 构建项目..."
cargo build --release
if [ $? -ne 0 ]; then
    echo "❌ 构建失败"
    exit 1
fi
echo "✅ 构建成功"
echo

BINARY="./target/release/line-counter"

# 测试计数器
PASSED=0
FAILED=0

# 测试函数
test_case() {
    local description="$1"
    local command="$2"
    local expected_exit_code="$3"
    local expected_message="$4"

    echo "🔍 测试: $description"

    # 执行命令并捕获输出
    output=$(eval "$command" 2>&1)
    exit_code=$?

    # 检查退出码
    if [ $exit_code -eq $expected_exit_code ]; then
        echo "  ✅ 退出码正确: $exit_code"
    else
        echo "  ❌ 退出码错误: 期望 $expected_exit_code, 实际 $exit_code"
        ((FAILED++))
        echo "  输出: $output"
        echo
        return
    fi

    # 检查错误消息
    if [[ "$output" == *"$expected_message"* ]]; then
        echo "  ✅ 错误消息正确"
        ((PASSED++))
    else
        echo "  ❌ 错误消息不匹配"
        echo "  期望包含: $expected_message"
        echo "  实际输出: $output"
        ((FAILED++))
    fi
    echo
}

# 测试用例

# 1. 缺少参数
test_case "缺少文件路径参数" \
    "$BINARY" \
    1 \
    "缺少文件路径参数"

# 2. 文件不存在
test_case "文件不存在" \
    "$BINARY nonexistent_file_12345.txt" \
    1 \
    "文件不存在"

# 3. 尝试处理目录
test_case "尝试处理目录" \
    "$BINARY src" \
    1 \
    "文件是一个目录"

# 4. 创建测试文件进行成功测试
echo "📝 创建测试文件..."
cat > test_success.txt << 'EOF'
第一行
第二行

第四行
EOF

test_case "正常文件处理" \
    "$BINARY test_success.txt" \
    0 \
    "文件分析完成"

# 5. 创建空文件测试
touch test_empty.txt

test_case "空文件处理" \
    "$BINARY test_empty.txt" \
    0 \
    "总行数: 0"

# 6. 创建只有空行的文件
cat > test_empty_lines.txt << 'EOF'



EOF

test_case "只有空行的文件" \
    "$BINARY test_empty_lines.txt" \
    0 \
    "空行占比: 100.0%"

# 7. 创建大文件测试（但不超过限制）
echo "📝 创建大文件测试..."
{
    for i in {1..1000}; do
        echo "这是第 $i 行"
    done
} > test_large.txt

test_case "大文件处理" \
    "$BINARY test_large.txt" \
    0 \
    "总行数: 1000"

# 8. 测试 Unicode 文件
cat > test_unicode.txt << 'EOF'
你好世界
🎉🚀📊
Hello World
EOF

test_case "Unicode 文件处理" \
    "$BINARY test_unicode.txt" \
    0 \
    "总行数: 3"

# 9. 测试包含特殊字符的文件
cat > test_special.txt << 'EOF'
包含特殊字符：@#$%^&*()
包含 emoji：😀😃😄
包含中文：你好世界
EOF

test_case "特殊字符文件处理" \
    "$BINARY test_special.txt" \
    0 \
    "非空行数: 3"

# 10. 测试只有空白字符的行
cat > test_whitespace.txt << 'EOF'
实际内容



另一行内容
EOF

test_case "空白字符行处理" \
    "$BINARY test_whitespace.txt" \
    0 \
    "空行数: 3"

# 清理测试文件
echo "🧹 清理测试文件..."
rm -f test_success.txt test_empty.txt test_empty_lines.txt test_large.txt test_unicode.txt test_special.txt test_whitespace.txt

# 测试结果汇总
echo "📊 测试结果汇总:"
echo "  ✅ 通过: $PASSED"
echo "  ❌ 失败: $FAILED"
echo "  📝 总计: $((PASSED + FAILED))"
echo

if [ $FAILED -eq 0 ]; then
    echo "🎉 所有测试通过！"
    exit 0
else
    echo "💥 有 $FAILED 个测试失败"
    exit 1
fi
