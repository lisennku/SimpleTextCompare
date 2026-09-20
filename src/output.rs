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

use crate::ansi_config::RESET;
use crate::line_status::LineStatus;
use crate::row::Row;
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
/// 本函数用于填充宽度不满足width的文本
///
/// - `text` 文本
/// - `width` 指定宽度
/// - `left_align` 左对齐
/// - `line_status` `LineStatus`枚举，负责渲染对应的颜色
/// - `color` 是否渲染颜色，如果是重定向则不添加
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

/// 负责输出对比终端的抬头
/// - `left_file`  左文件
/// - `right_file` 右文件
/// - `code_width` 代码列宽
/// - `no_width`   行号列宽
/// - `writer` 写入对象， `less`或者标准输出等
/// - `color` 是否渲染颜色，如果是重定向则不添加
pub fn output_wrapped_header(
    left_file: &str,
    right_file: &str,
    code_width: usize,
    no_width: usize,
    writer: &mut dyn Write,
    _color: bool,
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

pub fn format_side(
    segs: Option<&[(bool, String)]>,
    width: usize,
    status: LineStatus,
    color: bool,
) -> Vec<String> {
    let Some(segs) = segs else {
        return Vec::new();
    };

    let mut result_lines: Vec<(usize, Vec<(bool, String)>)> = Vec::new();
    let mut current_line: Vec<(bool, String)> = Vec::new();
    let mut current_width: usize = 0_usize;

    for (is_emphasis, line) in segs {
        let is_emphasis = *is_emphasis;
        for ch in line.chars() {
            let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
            if current_width + ch_width > width {
                result_lines.push((current_width, current_line));
                current_line = Vec::new();
                current_width = 0;
            }
            if current_line.last().map(|(b, _)| *b) == Some(is_emphasis) {
                current_line.last_mut().unwrap().1.push(ch);
            } else {
                current_line.push((is_emphasis, ch.to_string()));
            }
            current_width += ch_width;
        }
    }

    if !current_line.is_empty() {
        result_lines.push((current_width, current_line));
    }

    if result_lines.is_empty() {
        result_lines.push((0, Vec::new()));
    }

    result_lines
        .into_iter()
        .map(|(line_width, line_codes)| {
            let mut line = String::new();
            for (is_emphasis, line_seg) in line_codes {
                match status.piece_color(color, is_emphasis) {
                    Some(c) => line.push_str(&format!("{c}{line_seg}{RESET}")),
                    None => line.push_str(&line_seg),
                }
            }
            let padding_cnts = width.saturating_sub(line_width);
            line.push_str(" ".repeat(padding_cnts).as_str());
            line
        })
        .collect()
}

pub fn render_rows(
    rows: &[Row],
    code_width: usize,
    no_width: usize,
    writer: &mut dyn Write,
    color: bool,
) -> io::Result<()> {
    for row in rows {
        let left_segs = format_side(row.left_line.as_deref(), code_width, row.status, color);
        let right_segs = format_side(row.right_line.as_deref(), code_width, row.status, color);

        let max_lines_cnt = left_segs.len().max(right_segs.len()).max(1);

        // let placeholder = if !color {
        //     format!("{}{}", "-", " ".repeat(code_width - 1))
        // } else {
        //     format!("{}{}{}{}", YELLOW, "-", RESET, " ".repeat(code_width - 1))
        // };
        let placeholder = match row.status.piece_color(color, true) {
            Some(c) => format!("{c}-{RESET}{}", " ".repeat(code_width - 1)),
            None => format!("{}{}", "-", " ".repeat(code_width - 1)),
        };

        for i in 0..max_lines_cnt {
            let left_no = format_line_no(i, row.left_no);
            let right_no = format_line_no(i, row.right_no);

            let left_seg = left_segs.get(i).unwrap_or(&placeholder);
            let right_seg = right_segs.get(i).unwrap_or(&placeholder);
            let status_text = if i == 0 { row.status.to_str() } else { "" };

            writeln!(
                writer,
                "{} | {} | {} | {} | {}",
                padding_white_space(&left_no, no_width, false, None, false),
                left_seg,
                padding_white_space(&right_no, no_width, false, None, false),
                right_seg,
                padding_white_space(status_text, STATUS_WIDTH, true, None, false)
            )?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saturating_minus() {
        let a = 10_usize;
        let b = a.saturating_sub(11);
        println!("{:?}", b);
        println!("{:?}", " ".repeat(b));
    }
}
