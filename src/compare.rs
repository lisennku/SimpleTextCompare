//! 本模块主要用于进行文本比较
//! 提供
//! 1. 按照文本对照形式输出差异
use crate::line_status::LineStatus;
use crate::output;
use crate::output::output_replace_row;
use anyhow::{Context, Result};
use similar::{ChangeTag, DiffTag, TextDiff};
use std::fs;
use std::io::Write;
use std::path::Path;

/// 按照给定的文件，以表格形式输出两个文本之间的差异
/// - `left`  左文件
/// - `right` 右文件
/// - `code_width` 代码列宽
/// - `no_width` 行号列宽
/// - `writer` 写入对象， `less`或者标准输出等
///
/// 将代码中使用的`diff.iter_all_changes`替换为`diff.ops()` + `op.iter_inline_changes`
///
/// 并根据`op.tag`判断，只有`Replace`才会进入`iter_inline_changes`，
///
/// 其他场景通过`op.old_range`/`op.new_rang`和`diff.old_slice`/`diff.new_slice`获取文本
///
/// 避免inline算法耗时
pub fn compare_files_table_style(
    left: &Path,
    right: &Path,
    code_width: usize,
    no_width: usize,
    writer: &mut dyn Write,
    inline: bool,
    color: bool,
) -> Result<()> {
    let left_text =
        fs::read_to_string(left).with_context(|| format!("打开文件{}出错", left.display()))?;
    let right_text =
        fs::read_to_string(right).with_context(|| format!("打开文件{}出错", right.display()))?;
    let diff = TextDiff::from_lines(&left_text, &right_text);

    let left_file_name = left
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("<left>");
    let right_file_name = right
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("<right>");

    // let code_width = 50_usize;
    // let no_width = 6_usize;

    output::output_wrapped_header(
        left_file_name,
        right_file_name,
        code_width,
        no_width,
        writer,
        color,
    )?;
    output::output_separator_row(code_width * 2 + no_width * 2 + 3 * 4 + 6, writer)?;

    // 使用diff.ops按块捕获，以获取replace相关信息，而不是经iter_all_changes将Replace拆解为Delete和Insert
    for op in diff.ops() {
        match op.tag() {
            DiffTag::Equal => {
                for (o, n) in op.old_range().zip(op.new_range()) {
                    let left_no = Some(o + 1);
                    let right_no = Some(n + 1);
                    let left_line = diff
                        .old_slice(o)
                        .map(|x| x.trim_end_matches('\n').trim_end_matches('\r'));
                    let right_line = diff
                        .new_slice(n)
                        .map(|x| x.trim_end_matches('\n').trim_end_matches('\r'));
                    output::output_wrapped_row(
                        left_no,
                        left_line,
                        right_no,
                        right_line,
                        LineStatus::Equal,
                        code_width,
                        no_width,
                        writer,
                        color,
                    )?;
                }
            }
            DiffTag::Delete => {
                for o in op.old_range() {
                    let left_no = Some(o + 1);
                    let left_line = diff
                        .old_slice(o)
                        .map(|x| x.trim_end_matches('\n').trim_end_matches('\r'));
                    output::output_wrapped_row(
                        left_no,
                        left_line,
                        None,
                        None,
                        LineStatus::Delete,
                        code_width,
                        no_width,
                        writer,
                        color,
                    )?;
                }
            }
            DiffTag::Insert => {
                for n in op.new_range() {
                    let right_no = Some(n + 1);
                    let right_line = diff
                        .new_slice(n)
                        .map(|x| x.trim_end_matches('\n').trim_end_matches('\r'));
                    output::output_wrapped_row(
                        None,
                        None,
                        right_no,
                        right_line,
                        LineStatus::Insert,
                        code_width,
                        no_width,
                        writer,
                        color,
                    )?;
                }
            }
            DiffTag::Replace => {
                if !inline {
                    for inline in diff.iter_inline_changes(op) {
                        let left_no = inline.old_index().map(|n| n + 1);
                        let right_no = inline.new_index().map(|n| n + 1);
                        let line: String = inline.values().iter().map(|x| x.1).collect();
                        let line = line.as_str().trim_end_matches('\n').trim_end_matches('\r');

                        match inline.tag() {
                            ChangeTag::Delete => {
                                output::output_wrapped_row(
                                    left_no,
                                    Some(line),
                                    None,
                                    None,
                                    LineStatus::Delete,
                                    code_width,
                                    no_width,
                                    writer,
                                    color,
                                )?;
                            }
                            ChangeTag::Insert => {
                                output::output_wrapped_row(
                                    None,
                                    None,
                                    right_no,
                                    Some(line),
                                    LineStatus::Insert,
                                    code_width,
                                    no_width,
                                    writer,
                                    color,
                                )?;
                            }
                            _ => unreachable!(),
                        }
                    }
                } else {
                    let mut delete_vec: Vec<(Option<usize>, Vec<(bool, String)>)> = Vec::new();
                    let mut insert_vec: Vec<(Option<usize>, Vec<(bool, String)>)> = Vec::new();

                    for inline_item in diff.iter_inline_changes(op) {
                        match inline_item.tag() {
                            ChangeTag::Delete => delete_vec.push((
                                inline_item.old_index(),
                                inline_item
                                    .values()
                                    .iter()
                                    .map(|&(b, s)| {
                                        (
                                            b,
                                            s.trim_end_matches('\n')
                                                .trim_end_matches('\r')
                                                .to_string(),
                                        )
                                    })
                                    .collect(),
                            )),
                            ChangeTag::Insert => insert_vec.push((
                                inline_item.old_index(),
                                inline_item
                                    .values()
                                    .iter()
                                    .map(|&(b, s)| {
                                        (
                                            b,
                                            s.trim_end_matches('\n')
                                                .trim_end_matches('\r')
                                                .to_string(),
                                        )
                                    })
                                    .collect(),
                            )),
                            _ => unreachable!(),
                        }
                    }

                    let max_lines_cnt = delete_vec.len().max(insert_vec.len()).max(1);
                    for i in 0..max_lines_cnt {
                        let (left_no, left_segs) = match delete_vec.get(i) {
                            Some((no, segs)) => (
                                *no,
                                Some(
                                    segs.iter()
                                        .map(|(b, s)| (*b, s.as_str()))
                                        .collect::<Vec<(bool, &str)>>(),
                                ),
                            ),
                            None => (None, None),
                        };

                        let (right_no, right_segs) = match insert_vec.get(i) {
                            Some((no, segs)) => (
                                *no,
                                Some(
                                    segs.iter()
                                        .map(|(b, s)| (*b, s.as_str()))
                                        .collect::<Vec<(bool, &str)>>(),
                                ),
                            ),
                            None => (None, None),
                        };

                        let _ = output_replace_row(
                            left_no,
                            left_segs.as_deref(),
                            right_no,
                            right_segs.as_deref(),
                            code_width,
                            no_width,
                            writer,
                            color,
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_empty_lines() -> Result<()> {
        let p1 = Path::new(r"C:\Users\LFJ\Desktop\new.txt");
        let s = fs::read_to_string(p1)?;
        for (idx, line) in s.lines().enumerate() {
            println!("{}:{}+", idx + 1, line);
        }
        Ok(())
    }
    #[test]
    fn test_compare_files_table_style() -> Result<()> {
        let p1 = Path::new(
            r"D:\vscode_workspace\vscode_workspace\codes_rust\rust_learn\text_compare_cli\base.txt",
        );
        let p2 = Path::new(
            r"D:\vscode_workspace\vscode_workspace\codes_rust\rust_learn\text_compare_cli\comp.txt",
        );
        let mut w = std::io::stdout();
        compare_files_table_style(&p1, &p2, 50, 3, &mut w, false, true)?;
        Ok(())
    }
}
