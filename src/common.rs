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
