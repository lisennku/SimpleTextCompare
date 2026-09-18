//! 本模块用于在终端进行打印
//!
//! 打印概览如下
//!
//! 左行号 left文件 | 右行号 right文件 | 状态
//!
//!  ---------------------------------------
//!
//! 主要提供
//!
//! 1. 文本按照指定的终端宽度进行换行
//!
//! 2. 填充文本到指定宽度
//!
//! 3. 输出行、表头、分割线
//!

use crate::ansi_config::{RESET, YELLOW};
use crate::line_status::LineStatus;
use clap::builder::styling::Color;
use std::io::{self, Write};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[allow(dead_code)]
const CODE_WIDTH: usize = 10;
#[allow(dead_code)]
const NO_WIDTH: usize = 5;
const STATUS_WIDTH: usize = 6;

/// 优化行号处理代码
/// 抽象为一个函数
fn format_line_no(index: usize, line_no: Option<usize>) -> String {
    match (index, line_no) {
        (0, Some(n)) => n.to_string(),
        _ => "/".to_string(),
    }
}
/// 将文本按照显示宽度进行计算折叠
/// - `text` 输入文本
/// - `width` 指定的宽度
pub fn wrap_code_width(text: Option<&str>, width: usize) -> Vec<String> {
    // assert!(width > 40, "宽度至少为40，现在是{width}");

    // 参数文本修改为Option，Some(text)表示真实文档内容，None表示这一侧没有对应文本
    let Some(text) = text else {
        return Vec::new();
    };

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
/// - `line_status` `LineStatus`枚举，负责渲染对应的颜色
pub fn padding_white_space(
    text: &str,
    width: usize,
    left_align: bool,
    line_status: Option<LineStatus>,
    color: bool,
) -> String {
    let occupied_width = UnicodeWidthStr::width(text);
    let wrapped = match line_status {
        Some(ls) => ls.wrap_ansi(text, color),
        None => text.to_string(),
    };

    if occupied_width >= width {
        return wrapped;
    }

    let whites = " ".repeat(width - occupied_width);
    if left_align {
        format!("{wrapped}{whites}")
    } else {
        format!("{whites}{wrapped}")
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
/// - `left_line` 左文件行文本，对于`ChangeTag::Insert`来说，该参数为None
/// - `right_no` 右文件行号
/// - `right_line` 右文件行文本，对于`ChangeTag::Delete`来说，该参数为None
/// - `line_status` `LineStatus`枚举，负责表示具体文本和渲染颜色
/// - `code_width` 代码列的宽度，如果一行文本的长度(指的是终端显示长度)超过该值会进行换行
/// - `writer` 写入对象， `less`或者标准输出等

pub fn output_wrapped_row(
    left_no: Option<usize>,
    left_line: Option<&str>,
    right_no: Option<usize>,
    right_line: Option<&str>,
    line_status: LineStatus,
    code_width: usize,
    no_width: usize,
    writer: &mut dyn Write,
    color: bool,
) -> io::Result<()> {
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
        let left_no = format_line_no(i, left_no);

        let right_no = format_line_no(i, right_no);

        // Vec的get方法，返回的是Option<&T>
        // 闭包的as_str显式进行从&String 到 &str的转换
        let left_code = left_lines.get(i).map(|l| l.as_str()).unwrap_or("-");
        let right_code = right_lines.get(i).map(|l| l.as_str()).unwrap_or("-");
        let line_status_text = if i == 0 { line_status.to_str() } else { "" };

        writeln!(
            writer,
            "{} | {} | {} | {} | {}",
            padding_white_space(&left_no, no_width, false, None, color),
            padding_white_space(left_code, code_width, true, Some(line_status), color),
            padding_white_space(&right_no, no_width, false, None, color),
            padding_white_space(right_code, code_width, true, Some(line_status), color),
            padding_white_space(line_status_text, STATUS_WIDTH, true, None, color)
        )?;
    }
    Ok(())
}

/// 负责输出对比终端的抬头
/// - `left_file`  左文件
/// - `right_file` 右文件
/// - `code_width` 代码列宽
/// - `no_width`   行号列宽
/// - `writer` 写入对象， `less`或者标准输出等
pub fn output_wrapped_header(
    left_file: &str,
    right_file: &str,
    code_width: usize,
    no_width: usize,
    writer: &mut dyn Write,
    color: bool,
) -> io::Result<()> {
    let left_file = padding_white_space(left_file, code_width, true, None, false);

    let right_file = padding_white_space(right_file, code_width, true, None, false);

    writeln!(
        writer,
        "{} | {} | {} | {} | {}",
        padding_white_space("行号", no_width, true, None, false),
        &left_file,
        padding_white_space("行号", no_width, true, None, false),
        &right_file,
        padding_white_space("状态", STATUS_WIDTH, true, None, false),
    )?;

    Ok(())
}

/// 负责输出分割线
pub fn output_separator_row(width: usize, writer: &mut dyn Write) -> io::Result<()> {
    writeln!(writer, "{}", "-".repeat(width))?;
    Ok(())
}

/// 负责将Replace部分的代码块，进行
///
/// 1. 按宽度折叠
/// 2. 补齐到宽度
/// 3. 对修改片段进行染色
///
/// 需要注意的是，Replace片段传进来的Option<&[(bool, &str)]>，里面是个序列
///
/// 代码折叠可能发生在需要标记颜色的位置
///
/// 外部函数传入的`seg`参数可能是None，表示没有传入
///
/// 也可能传入的Some内部带有(bool, "") 表示外部确有传入，但是为空字符串
///
/// 参数
/// - `segs` 外部`diff.iter_inline_changes(op)`产出结果的`values`
/// - `width` 指定的宽度

pub fn wrap_pad_emphasis_for_replace(
    segs: Option<&[(bool, &str)]>,
    width: usize,
    color: bool,
) -> Vec<String> {
    // 边界条件，表示实际无传入
    let Some(segs) = segs else {
        return Vec::new();
    };
    // 结果容器
    // 每个子项目都表示一个独占的行
    let mut result_lines: Vec<(usize, Vec<(bool, String)>)> = Vec::new();

    // current_line用于存储满足行宽的代码，但是因为涉及到着色，所以分成若干个<bool, String>
    let mut current_line: Vec<(bool, String)> = Vec::new();
    let mut current_width: usize = 0_usize;

    // 此时segs，是&[(bool, &str)]
    // 下方循环匹配时统一使用 &
    // 在模式前面加个 & —— 它先把 &(bool, &str) 解引用，然后 (is_emphasis, codes) 按值绑
    // 因为 bool 和 &str 都是 Copy，这俩会被直接拷出来
    for &(is_emphasis, codes) in segs {
        for ch in codes.chars() {
            let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
            // 超过指定宽度时，将current_line存入result_lines，并重置
            if current_width + ch_width > width {
                result_lines.push((current_width, current_line));
                current_line = Vec::new();
                current_width = 0;
            }

            if current_line.last().map(|(is_emphasis, _)| *is_emphasis) == Some(is_emphasis) {
                current_line.last_mut().unwrap().1.push(ch);
            } else {
                current_line.push((is_emphasis, ch.to_string()));
            }

            current_width += ch_width;
        }
    }

    if !(current_line).is_empty() {
        result_lines.push((current_width, current_line));
    }

    result_lines
        .into_iter()
        .map(|(line_width, line_pair)| {
            let mut true_line = String::new();
            for (is_emphasis, line) in line_pair {
                if is_emphasis && color {
                    true_line.push_str(format!("{}{}{}", YELLOW, line, RESET).as_str());
                } else {
                    true_line.push_str(line.as_str());
                }
            }
            let padding_cnts = width.saturating_sub(line_width);
            true_line.push_str(" ".repeat(padding_cnts).as_str());
            true_line
        })
        .collect()
}

pub fn output_replace_row(
    left_no: Option<usize>,
    left_segs: Option<&[(bool, &str)]>,
    right_no: Option<usize>,
    right_segs: Option<&[(bool, &str)]>,
    code_width: usize,
    no_width: usize,
    writer: &mut dyn Write,
    color: bool,
) -> io::Result<()> {
    // left_lines和right_lines此时为Vec<String>容器
    // 可能有一个元素，表示代码行没超过列宽
    // 可能有多个元素，表示代码行超过列宽发生折叠
    let left_lines = wrap_pad_emphasis_for_replace(left_segs, code_width, color);
    let right_lines = wrap_pad_emphasis_for_replace(right_segs, code_width, color);

    // 用1兜底max_lines_cnt，防止意外导致for循环panic
    let max_lines_cnt = left_lines.len().max(right_lines.len()).max(1);

    // 循环处理left_lines和right_lines
    // 确保，只有在第一个元素时，才输出有效的行号、状态，其他保持占位符
    // 对于多行文本，padding后再输出，如果左右对应的地方没有文本，使用""占位
    for i in 0..max_lines_cnt {
        // 先处理行号，因为传入的是Option，要进行处理拿到真实行号
        // 并且结合索引，只有第一个元素才显示行号
        let left_no = format_line_no(i, left_no);

        let right_no = format_line_no(i, right_no);

        // Vec的get方法，返回的是Option<&T>
        // 闭包的as_str显式进行从&String 到 &str的转换
        if !color {
            let placeholder = format!("{}{}", "-", " ".repeat(code_width - 1));
        } else {
            let placeholder = format!("{}{}{}{}", YELLOW, "-", RESET, " ".repeat(code_width - 1));
        }
        let placeholder = format!("{}{}{}{}", YELLOW, "-", RESET, " ".repeat(code_width - 1));
        let left_code = left_lines
            .get(i)
            .map(|l| l.as_str())
            .unwrap_or(placeholder.as_str());
        let right_code = right_lines
            .get(i)
            .map(|l| l.as_str())
            .unwrap_or(placeholder.as_str());
        let line_status_text = if i == 0 { "Replace" } else { "" };

        writeln!(
            writer,
            "{} | {} | {} | {} | {}",
            padding_white_space(&left_no, no_width, false, None, false),
            left_code,
            padding_white_space(&right_no, no_width, false, None, false),
            right_code,
            padding_white_space(line_status_text, STATUS_WIDTH, true, None, false)
        )?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_row_cases() -> std::io::Result<()> {
        let out = &mut std::io::stdout();

        println!("--- Case 1 普通替换 ---");
        output_replace_row(
            Some(1),
            Some(&[(false, "let x = "), (true, "1"), (false, ";")]),
            Some(1),
            Some(&[(false, "let x = "), (true, "2"), (false, ";")]),
            8,
            4,
            out,
            true,
        )?;

        println!("--- Case 2 行数不等 ---");
        output_replace_row(
            Some(10),
            Some(&[(false, "aaaaaaaaaa")]),
            Some(10),
            Some(&[(true, "bb")]),
            8,
            4,
            out,
            true,
        )?;

        println!("--- Case 3 超长 true 跨行 ---");
        output_replace_row(
            Some(3),
            Some(&[(true, "0123456789abcd")]),
            Some(3),
            Some(&[(false, "x")]),
            8,
            4,
            out,
            true,
        )?;

        println!("--- Case 4 一侧缺失 ---");
        output_replace_row(
            None,
            None,
            Some(7),
            Some(&[(false, "hello")]),
            8,
            4,
            out,
            true,
        )?;

        println!("--- Case 5 多片段+窄折叠 ---");
        output_replace_row(
            Some(5),
            Some(&[
                (false, "abcdefgh"),
                (true, "123134356"),
                (false, "czcvzxcvcx"),
            ]),
            Some(5),
            Some(&[(true, "short")]),
            5,
            4,
            out,
            true,
        )?;

        Ok(())
    }
    #[test]
    fn replace_output_wrapped() {
        // Some(&[(false, "abc")])                    // 不折 → 1 行: "abc"
        // Some(&[(false, "abcdefgh")])               // 单段超宽 → 2 行: "abcde" / "fgh"
        // Some(&[(false, "ab"), (true, "cd")])       // 两段拼起来不超 → 1 行: "abcd"
        // Some(&[(false, "ab"), (true, "cdefgh")])   // 断点落在第二段内部 → 2 行: "abcde" / "fgh"
        // Some(&[(true, "abcdefghij")])              // 长段 → 2 行: "abcde" / "fghij"
        // Some(&[(false, "a"), (true, "b"), (false, "cdefg")])
        let v = wrap_pad_emphasis_for_replace(
            Some(&[(false, "人民币"), (true, "1dfasdfa234")]),
            5,
            true,
        );
        for s in v {
            println!("{}", s);
        }
    }

    #[test]
    fn test_saturating_minus() {
        let a = 10_usize;
        let b = a.saturating_sub(11);
        println!("{:?}", b);
        println!("{:?}", " ".repeat(b));
    }

    #[test]
    fn blank_line_to_wrap() {
        let text = String::from("");
        let v = wrap_code_width(Some(&text), 4);
        assert_eq!(v, vec![String::new()]);
        println!("{:#?}", v);
    }

    #[test]
    fn empty_line_is_not_missing_line() {
        assert_eq!(wrap_code_width(Some(""), 4), vec![String::new()]);

        assert_eq!(wrap_code_width(None, 4), Vec::<String>::new());
    }
    #[test]
    fn no_need_to_wrap() {
        let text = String::from("Hello, world!");
        let v = wrap_code_width(Some(&text), 4);
        println!("{:#?}", v);
    }

    #[test]
    fn newline_wrap() {
        let text = String::from("Hello\nworld!");
        let v = wrap_code_width(Some(&text), 4);
        println!("{:#?}", v);
    }

    #[test]
    fn special_chars() {
        let text = String::from("y̆éñäôüçi̊");
        let v = wrap_code_width(Some(&text), 4);
        for line in &v {
            println!("{}", line);
        }
    }
    #[test]
    fn output_wrapped_delete() {
        let mut w = std::io::stdout();
        output_wrapped_header(
            "left file",
            "right file",
            CODE_WIDTH,
            NO_WIDTH,
            &mut w,
            true,
        )
        .unwrap();
        output_separator_row(2 * NO_WIDTH + 3 * CODE_WIDTH + 3 * 4, &mut w).unwrap();
        output_wrapped_row(
            Some(1),
            Some("a".repeat(16).as_str()),
            None,
            None,
            LineStatus::Delete,
            CODE_WIDTH,
            NO_WIDTH,
            &mut w,
            true,
        )
        .unwrap();
    }
    #[test]
    fn output_wrapped_insert() {
        let mut w = std::io::stdout();
        output_wrapped_header(
            "left file",
            "right file",
            CODE_WIDTH,
            NO_WIDTH,
            &mut w,
            true,
        )
        .unwrap();
        output_separator_row(2 * NO_WIDTH + 3 * CODE_WIDTH + 3 * 4, &mut w).unwrap();
        output_wrapped_row(
            None,
            None,
            Some(1),
            Some("a".repeat(16).as_str()),
            LineStatus::Insert,
            CODE_WIDTH,
            NO_WIDTH,
            &mut w,
            true,
        )
        .unwrap();
    }
}
