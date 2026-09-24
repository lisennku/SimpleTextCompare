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
use crate::consts;
use crate::line_status::LineStatus;
use crate::row::Row;
use std::io::{self, Write};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// `Segment`表示一行代码中的一个片段
/// - `bool` 表示该片段在`inline`模式下，是否要标记颜色
/// - `String` 代码片段字符串
type Segment = (bool, String);

/// `format_side`函数用到的数据结构
///
/// 主要表示一行代码中，按照指定宽度进行折叠后的子行数据对象
///
/// - `line_seg_width` 表示当前子行的`unicode`字符长度
/// - `line_segs` 表示当前子行内的着色片段列表
///     - `bool`表示是否进行重点着色
///     - `String` 表示该片段的文本
/// - `newline_flag` 表示是否为missing_newline的标记
struct FormattedLine {
    line_seg_width: usize,
    line_segs: Vec<Segment>,
    newline_flag: bool,
}

impl FormattedLine {
    fn new(line_seg_width: usize, line_segs: Vec<Segment>, newline_flag: bool) -> Self {
        Self {
            line_seg_width,
            line_segs,
            newline_flag,
        }
    }
}

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
fn padding_white_space(text: &str, width: usize, left_align: bool) -> String {
    let occupied_width = UnicodeWidthStr::width(text);

    if occupied_width >= width {
        return text.to_string();
    }

    let whites = " ".repeat(width.saturating_sub(occupied_width));
    if left_align {
        format!("{text}{whites}")
    } else {
        format!("{whites}{text}")
    }
}

/// 负责输出对比终端的抬头
/// - `left_file`  左文件
/// - `right_file` 右文件
/// - `code_width` 代码列宽
/// - `no_width`   行号列宽
/// - `writer` 写入对象， `less`或者标准输出等
/// - `_color` 是否渲染颜色，如果是重定向则不添加，但此函数不涉及染色，因此只声明不启用
pub fn output_wrapped_header(
    left_file: &str,
    right_file: &str,
    code_width: usize,
    no_width: usize,
    writer: &mut dyn Write,
    _color: bool,
) -> io::Result<()> {
    let left_file = padding_white_space(left_file, code_width, true);

    let right_file = padding_white_space(right_file, code_width, true);

    writeln!(
        writer,
        "{} | {} | {} | {} | {}",
        padding_white_space("行号", no_width, true),
        &left_file,
        padding_white_space("行号", no_width, true),
        &right_file,
        padding_white_space("状态", consts::STATUS_WIDTH, true),
    )?;

    Ok(())
}

/// 负责输出分割线
pub fn output_separator_row(width: usize, writer: &mut dyn Write) -> io::Result<()> {
    writeln!(writer, "{}", "-".repeat(width))?;
    Ok(())
}

/// 进行代码折叠，只处理`Row`里的`left_line`和`right_line`
///
/// 负责依据指定的代码列宽进行折叠，染色和填充列宽(针对不满足指定列宽的子行)
///
/// 内部使用`Vec<FormattedLine>`来表示对一行代码的折叠结构
/// - `(usize, Vec<Segment>, bool)`
///     - `usize` 表示当前子行的`unicode`字符长度
///     - `Vec<Segment>` 表示当前子行内的着色片段列表
///         - `bool`表示是否进行重点着色
///         - `String` 表示该片段的文本
///     - `bool` 表示是否为missing_newline的标记
///
///     - 处理逻辑是每次填充字符时判断当前`Vec`里的`bool`和最后一个元素的`bool`是否相同，相同则直接`append`，不同则开启一个新的元素
///         - 原因是要保留行内着色的`bool`
///
/// 收集后进行闭包着色与填充处理
pub fn format_side(
    segs: Option<&[Segment]>,
    width: usize,
    status: LineStatus,
    color: bool,
) -> Vec<String> {
    let Some(segs) = segs else {
        return Vec::new();
    };

    let mut result_lines: Vec<FormattedLine> = Vec::new();
    let mut current_line: Vec<Segment> = Vec::new();
    let mut current_width: usize = 0_usize;
    // 当前处理FormattedLine是否为NEWLINE标记
    let mut newline_flag: bool = false;

    for (is_emphasis, line) in segs {
        let is_emphasis = *is_emphasis;
        for ch in line.chars() {
            // 进入到这里的\n，只有新行判断增加的\n和NO_NEWLINE
            // 且missing_newline这个是文件级别的，只会出现在最后的位置，newline_flag不需要重置
            if ch == '\n' {
                result_lines.push(FormattedLine::new(
                    current_width,
                    current_line,
                    newline_flag,
                ));
                current_line = Vec::new();
                current_width = 0;
                newline_flag = true;
                continue;
            }
            let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
            if current_width + ch_width > width {
                result_lines.push(FormattedLine::new(
                    current_width,
                    current_line,
                    newline_flag,
                ));
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
        result_lines.push(FormattedLine::new(
            current_width,
            current_line,
            newline_flag,
        ));
    }

    if result_lines.is_empty() {
        result_lines.push(FormattedLine::new(0, Vec::new(), false));
    }

    result_lines
        .into_iter()
        .map(|fmt_line| {
            let mut line = String::new();
            for (is_emphasis, line_seg) in fmt_line.line_segs {
                if fmt_line.newline_flag {
                    line.push_str(&line_seg);
                } else {
                    match status.piece_color(color, is_emphasis) {
                        Some(c) => line.push_str(&format!("{c}{line_seg}{RESET}")),
                        None => line.push_str(&line_seg),
                    }
                }
            }
            let padding_cnts = width.saturating_sub(fmt_line.line_seg_width);
            line.push_str(" ".repeat(padding_cnts).as_str());
            line
        })
        .collect()
}

/// 输出每个处理后的行
///
/// 因为要输出的行可能被分成多个子行，因此需要循环处理
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

        let placeholder = match row.status.piece_color(color, true) {
            Some(c) => format!("{c}-{RESET}{}", " ".repeat(code_width.saturating_sub(1))),
            None => format!("{}{}", "-", " ".repeat(code_width.saturating_sub(1))),
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
                padding_white_space(&left_no, no_width, false,),
                left_seg,
                padding_white_space(&right_no, no_width, false,),
                right_seg,
                padding_white_space(status_text, consts::STATUS_WIDTH, true,)
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

    /// 回归：无尾换行行的正文子行必须染色，marker 子行不染色
    ///
    /// 旧代码在 map 里误用外层局部 `newline_flag`（无尾换行文件在循环里已翻成 true），
    /// 正文子行也被去色；改成逐行的 `fmt_line.newline_flag` 后正文恢复染色。
    #[test]
    fn format_side_colors_content_but_not_missing_newline_marker() {
        // plain_seg 对“无尾换行行”的产出：单个 Segment 内嵌 \n + marker
        let segs = vec![(false, format!("beta\n{}", consts::NO_NEWLINE))];

        // width=50 保证不折行（"beta"=4、marker=27 均 <50）；Insert→绿色；color=true
        let out = format_side(Some(&segs), 50, LineStatus::Insert, true);

        // 正文子行 + marker 子行
        assert_eq!(out.len(), 2);

        // 期望色码不硬写，走 piece_color（顺带验证 Insert 无视 is_emphasis）
        let code = LineStatus::Insert.piece_color(true, false).unwrap();

        // 回归闸：正文子行必须染色（旧代码此处为 false → 测试红）
        assert!(out[0].contains(code), "正文子行应被染色");
        assert!(out[0].contains("beta"));

        // marker 子行不染色
        assert!(!out[1].contains(code), "missing-newline marker 不应染色");
        assert!(out[1].contains("No newline"));
    }
}
