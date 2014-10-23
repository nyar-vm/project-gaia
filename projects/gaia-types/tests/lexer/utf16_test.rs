use gaia_types::lexer::{LexerState, TokenType};

#[derive(Clone, Copy, Debug, PartialEq)]
enum TestToken {
    Char,
    Eof,
}

impl TokenType for TestToken {
    const END_OF_STREAM: Self = TestToken::Eof;
}

#[test]
fn test_utf16_column_calculation() {
    // 测试 UTF-16 列计算
    let test_cases = vec![
        ("abc", vec![(1, 1), (1, 2), (1, 3)]),    // 基本拉丁字符
        ("你好", vec![(1, 1), (1, 2)]),           // 中文字符（BMP）
        ("😀", vec![(1, 1), (1, 3)]),             // 表情符号（代理对，跳过第2列）
        ("h😀l", vec![(1, 1), (1, 2), (1, 4)]),   // 混合内容
        ("a\nb", vec![(1, 1), (1, 2), (2, 1)]),   // 换行
        ("a\r\nb", vec![(1, 1), (1, 2), (2, 1)]), // \r\n 换行
    ];

    for (input, expected_positions) in test_cases {
        println!("\n测试输入: {:?}", input);
        let mut state: LexerState<TestToken> = LexerState::new(input, None);

        for (i, &(expected_line, expected_column)) in expected_positions.iter().enumerate() {
            if i > 0 {
                state.next_char();
            }
            let (_, line, column) = state.get_position();
            println!(
                "  位置 {}: 期望 (line: {}, column: {}), 实际 (line: {}, column: {})",
                i, expected_line, expected_column, line, column
            );

            assert_eq!(line, expected_line, "行号不匹配");
            assert_eq!(column, expected_column, "列号不匹配");
        }
        println!("  ✓ 测试通过");
    }
}

#[test]
fn test_mixed_content_utf16() {
    let input = "hello 世界 😀";
    let mut state: LexerState<TestToken> = LexerState::new(input, None);

    // 期望的位置序列
    let expected = vec![
        (1, 1),  // 'h'
        (1, 2),  // 'e'
        (1, 3),  // 'l'
        (1, 4),  // 'l'
        (1, 5),  // 'o'
        (1, 6),  // ' '
        (1, 7),  // '世' (BMP字符)
        (1, 8),  // '界' (BMP字符)
        (1, 9),  // ' '
        (1, 10), // '😀' (代理对，应该显示当前位置，即表情符号的起始位置)
    ];

    for (i, &(expected_line, expected_column)) in expected.iter().enumerate() {
        let (_, line, column) = state.get_position();
        println!("位置 {}: (line: {}, column: {})", i, line, column);
        assert_eq!(line, expected_line);
        assert_eq!(column, expected_column);

        if i < expected.len() - 1 {
            state.next_char();
        }
    }
}
