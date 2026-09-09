//! 本模块用于在终端进行打印
//! 打印概览如下
//! 左行号 left文件 | 右行号 right文件 | 状态
//!  ---------------------------------------
//! 主要入口有三部分
//! 1. `output_wrapped_row(
//! left_no: Option<usize>, left_line: &str,
//! right_no: Option<usize>, right_line: &str,
//! line_status: &str, code_width: usize
//! )`
//!     - 负责输出行内容
//!     - 参数说明
//!         - `left_no` 左文件行号
//!         - `left_line` 左文件行文本，对于`ChangeTag::Insert`来说，该参数为""
//!         - `right_no` 右文件行号
//!         - `right_line` 右文件行文本，对于`ChangeTag::Delete`来说，该参数为""
//!         - `line_status` 该行状态，相同、插入、删除
//!         - `code_width` 代码列的宽度，如果一行文本的长度(指的是终端显示长度)超过该值会进行换行
//! 2. `output_wrapped_header(left_file: &str, right_file: &str, width)`
//!     - 负责输出header
//!     - 参数说明
//!         - `left_file` 左文件名
//!         - `right_file` 右文件名
//!         - `width` 列宽
//! 3. `output_seperator_row(width: usize)`
//!     - 负责输出分割行
//!     - 参数说明
//!         - `width` 整个行的行宽

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
const CODE_WIDTH: usize = 10;
const NO_WIDTH: usize = 5;
/// 将文本按照显示宽度进行计算折叠
/// - `text` 输入文本
/// - `width` 指定的宽度
pub fn wrap_code_width(text: &str, width: usize) -> Vec<String> {
    // assert!(width > 40, "宽度至少为40，现在是{width}");

    // 如果输入文本为空，直接返回一个vector，包空字符串
    if text.is_empty() {
        return vec![String::new()];
    }

    let mut lines: Vec<String> = Vec::new(); // 存储多个字符串，每个字符串的显示宽度均小于指定的width
    let mut current_chars = String::new(); // 当前遍历过的字符
    let mut current_width: usize = 0; // 当前遍历过的字符的显示宽度

    // 遍历text的字符
    // current_chars保存已经遍历过的字符，current_width保存已经遍历过的字符的显示宽度
    // 如果算到下一个字符，超过了给定宽度，则将current_chars放到lines里，重置两个current变量
    // 遇到换行符，则直接将current_chars放到lines里，重置两个current变量
    for ch in text.chars() {
        if ch == '\n' {
            lines.push(current_chars);
            current_chars = String::new();
            current_width = 0;
            continue;
        }

        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if current_width + ch_width > width {
            lines.push(current_chars);
            current_chars = String::new();
            current_width = 0;
        }
        current_chars.push(ch);
        current_width += ch_width;
    }

    if !current_chars.is_empty() {
        lines.push(current_chars);
    }

    lines
}

/// 本函数用于填充宽度不满足width的文本
///
/// - `text` 文本
/// - `width` 指定宽度
/// - `left_align` 左对齐
pub fn padding_white_space(text: &str, width: usize, left_align: bool) -> String {
    let occupied_width = UnicodeWidthStr::width(text);
    if occupied_width >= width {
        return text.to_string();
    }

    let whites = " ".repeat(width - occupied_width);
    if left_align {
        format!("{text}{whites}")
    } else {
        format!("{whites}{text}")
    }
}

/// 对比结果行输出函数
///
/// 每次处理一行`TextDiff::Change`对象
///
/// 要处理行转换，将一行超过给定宽度的文本进行wrap处理
///
/// 输出时，对于wrap处理的文本要对行号，状态特殊判断，如果确实换行，则不增加行号，也不展示雷同的状态
///
/// - `left_no` 左文件行号
/// - `left_line` 左文件行文本，对于`ChangeTag::Insert`来说，该参数为""
/// - `right_no` 右文件行号
/// - `right_line` 右文件行文本，对于`ChangeTag::Delete`来说，该参数为""
/// - `line_status` 该行状态，相同、插入、删除
/// - `code_width` 代码列的宽度，如果一行文本的长度(指的是终端显示长度)超过该值会进行换行

pub fn output_wrapped_row(
    left_no: Option<usize>,
    left_line: &str,
    right_no: Option<usize>,
    right_line: &str,
    line_status: &str,
    code_width: usize,
) {
    // left_lines和right_lines此时为Vec<String>容器
    // 可能有一个元素，表示代码行没超过列宽
    // 可能有多个元素，表示代码行超过列宽发生折叠
    let left_lines = wrap_code_width(left_line, code_width);
    let right_lines = wrap_code_width(right_line, code_width);

    // 用1兜底max_lines_cnt，防止意外导致for循环panic
    let max_lines_cnt = left_lines.len().max(right_lines.len()).max(1);

    // 循环处理left_lines和right_lines
    // 确保，只有在第一个元素时，才输出有效的行号、状态，其他保持占位符
    // 对于多行文本，padding后再输出，如果左右对应的地方没有文本，使用""占位
    for i in 0..max_lines_cnt {
        // 先处理行号，因为传入的是Option，要进行处理拿到真实行号
        // 并且结合索引，只有第一个元素才显示行号
        let left_no = if i == 0 {
            left_no.map(|n| n.to_string()).unwrap_or(String::new())
        } else {
            String::new()
        };

        let right_no = if i == 0 {
            right_no.map(|n| n.to_string()).unwrap_or(String::new())
        } else {
            String::new()
        };

        // Vec的get方法，返回的是Option<&T>
        // 闭包的as_str显式进行从&String 到 &str的转换
        let left_code = left_lines.get(i).map(|l| l.as_str()).unwrap_or("-");
        let right_code = right_lines.get(i).map(|l| l.as_str()).unwrap_or("-");
        let line_status = if i == 0 { line_status } else { "" };

        println!(
            "{} | {} | {} | {} | {}",
            padding_white_space(&left_no, NO_WIDTH, true),
            padding_white_space(left_code, code_width, true),
            padding_white_space(&right_no, NO_WIDTH, true),
            padding_white_space(right_code, code_width, true),
            padding_white_space(line_status, code_width, true)
        )
    }
}

pub fn output_wrapped_header(
    left_file: &str,
    right_file: &str,
    code_width: usize,
    no_width: usize,
) {
    let left_file = padding_white_space(left_file, code_width, true);

    let right_file = padding_white_space(right_file, code_width, true);

    println!(
        "{} | {} | {} | {} | {}",
        padding_white_space("行号", no_width, true),
        &left_file,
        padding_white_space("行号", no_width, true),
        &right_file,
        padding_white_space("状态", code_width, true),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_line_to_wrap() {
        let text = String::from("");
        let v = wrap_code_width(&text, 4);
        println!("{:#?}", v);
    }
    #[test]
    fn no_need_to_wrap() {
        let text = String::from("Hello, world!");
        let v = wrap_code_width(&text, 4);
        println!("{:#?}", v);
    }

    #[test]
    fn newline_wrap() {
        let text = String::from("Hello\nworld!");
        let v = wrap_code_width(&text, 4);
        println!("{:#?}", v);
    }

    #[test]
    fn special_chars() {
        let text = String::from("y̆éñäôüçi̊");
        let v = wrap_code_width(&text, 4);
        for line in &v {
            println!("{}", line);
        }
    }
    #[test]
    fn output_wrapped_delete() {
        output_wrapped_header("left file", "right file", CODE_WIDTH, NO_WIDTH);
        output_wrapped_row(
            Some(1),
            "a".repeat(16).as_str(),
            None,
            "",
            "Insert",
            CODE_WIDTH,
        )
    }
}
