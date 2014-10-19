use gaia_types::GaiaError;

#[test]
fn test_invalid_range_error() {
    // 创建一个无效范围错误
    let error = GaiaError::invalid_range(1024, 2048);
    
    // 测试错误消息格式
    let error_message = format!("{}", error);
    assert_eq!(error_message, "无效范围: 实际长度 1024，期望长度 2048");
    
    // 测试 Debug 格式
    let debug_message = format!("{:?}", error);
    assert!(debug_message.contains("InvalidRange"));
    assert!(debug_message.contains("length: 1024"));
    assert!(debug_message.contains("expect: 2048"));
}

#[test]
fn test_invalid_range_usage() {
    // 模拟一个实际使用场景
    fn validate_buffer_size(actual: usize, expected: usize) -> Result<(), GaiaError> {
        if actual != expected {
            return Err(GaiaError::invalid_range(actual, expected));
        }
        Ok(())
    }
    
    // 测试成功情况
    assert!(validate_buffer_size(100, 100).is_ok());
    
    // 测试失败情况
    let result = validate_buffer_size(50, 100);
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    let message = format!("{}", error);
    assert_eq!(message, "无效范围: 实际长度 50，期望长度 100");
}