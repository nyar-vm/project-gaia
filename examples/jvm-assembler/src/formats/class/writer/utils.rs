/// 计算 JVM 方法描述符中的参数数量
pub fn calculate_parameter_count(descriptor: &str) -> usize {
    if !descriptor.starts_with('(') {
        return 0;
    }

    let mut count = 0;
    let mut chars = descriptor.chars().skip(1); // 跳过 '('

    while let Some(c) = chars.next() {
        if c == ')' {
            break;
        }

        match c {
            'L' => {
                // 引用类型，跳过直到 ';'
                while let Some(c) = chars.next() {
                    if c == ';' {
                        break;
                    }
                }
                count += 1;
            }
            '[' => {
                // 数组类型，跳过直到基本类型或引用类型
                while let Some(c) = chars.next() {
                    if c == '[' {
                        continue;
                    }
                    if c == 'L' {
                        while let Some(c) = chars.next() {
                            if c == ';' {
                                break;
                            }
                        }
                    }
                    break;
                }
                count += 1;
            }
            'J' | 'D' => {
                // long 和 double 占用两个槽位
                count += 2;
            }
            _ => {
                // 其他基本类型 (I, S, B, C, Z, F)
                count += 1;
            }
        }
    }

    count
}
