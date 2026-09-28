//! 将多个源代码共用的函数或者结构抽象到该文档内

/// 针对终端控制字符，以十六进制来看，是0x00-0x1F，都是控制字符，其中可能会引起终端转义注入问题
///
/// 其中
///
/// - `0x09` `TAB`符号，转为空格`' '`
/// - 其他控制字符 转为`caret`字符
/// - 正常字符原样返回
/// - `0x7F` `DEL`符号 转为`^?`
pub fn terminal_control_convert_to_safety(ch: char) -> String {
    let ch_u32 = ch as u32;
    if ch_u32 == 0x09 {
        " ".to_string()
    } else if ch_u32 <= 0x1F {
        format!(
            "{}{}",
            '^',
            char::from_u32(ch_u32 + 0x40).expect("控制字符加0x40必落在合法ASCII区")
        )
    } else if ch_u32 == 0x7F {
        "^?".to_string()
    } else {
        ch.to_string()
    }
}

/// 负责处理终端转义注入的字符
pub fn get_sanitized_string(text: &str) -> String {
    text.chars()
        .map(terminal_control_convert_to_safety)
        .collect::<String>()
}

/// `Segment`表示一行代码中的一个片段
/// - `emphasis` 表示该片段在`inline`模式下，是否要标记颜色，非`Replace`块，均为`false`
/// - `seg_text` 代码片段字符串
#[derive(Debug)]
pub struct Segment {
    pub emphasis: bool,
    pub seg_text: String,
}

impl Segment {
    pub fn new(emphasis: bool, seg_text: String) -> Segment {
        Self { emphasis, seg_text }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 控制字符转义：`TAB`转空格，其余`0x00-0x1F`转caret，`0x7F`转`^?`，普通字符原样返回
    #[test]
    fn control_char_escaping() {
        assert_eq!(terminal_control_convert_to_safety('\u{1b}'), "^["); // ESC
        assert_eq!(terminal_control_convert_to_safety('\u{0}'), "^@"); // NUL
        assert_eq!(terminal_control_convert_to_safety('\u{7}'), "^G"); // BEL
        assert_eq!(terminal_control_convert_to_safety('\t'), " "); // TAB -> space
        assert_eq!(terminal_control_convert_to_safety('\u{7f}'), "^?"); // DEL
        assert_eq!(terminal_control_convert_to_safety('A'), "A"); // 普通ASCII
        assert_eq!(terminal_control_convert_to_safety('中'), "中"); // 非ASCII原样
    }

    /// 一段ANSI注入payload经处理后不应残留裸ESC，且正文文本保留
    #[test]
    fn sanitizes_ansi_injection_payload() {
        let out = get_sanitized_string("\u{1b}[31mRED\u{1b}[0m");
        assert!(!out.contains('\u{1b}')); // 没有裸ESC残留
        assert!(out.contains("^[")); // ESC被caret转义
        assert!(out.contains("31mRED")); // 正文保留
        assert!(out.contains("0m")); // 结尾reset文本保留
    }
}
