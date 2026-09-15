//! 本模块主要用于进行文本比较
//! 提供
//! 1. 按照文本对照形式输出差异
use crate::line_status::LineStatus;
use crate::output;
use anyhow::{Context, Result};
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::io::Write;
use std::path::Path;

/// 按照给定的文件，以表格形式输出两个文本之间的差异
/// - `left`  左文件
/// - `right` 右文件
/// - `code_width` 代码列宽
/// - `no_width` 行号列宽
/// - `writer` 写入对象， `less`或者标准输出等
pub fn compare_files_table_style(
    left: &Path,
    right: &Path,
    code_width: usize,
    no_width: usize,
    writer: &mut dyn Write,
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
    )?;
    output::output_separator_row(code_width * 2 + no_width * 2 + 3 * 4 + 6, writer)?;

    let mut left_no = 1_usize;
    let mut right_no = 1_usize;

    for change in diff.iter_all_changes() {
        let line = change.value().trim_end_matches('\n').trim_end_matches('\r');
        match change.tag() {
            ChangeTag::Equal => {
                output::output_wrapped_row(
                    Some(left_no),
                    Some(line),
                    Some(right_no),
                    Some(line),
                    LineStatus::Equal.to_str(),
                    code_width,
                    no_width,
                    writer,
                )?;
                left_no += 1;
                right_no += 1;
            }
            ChangeTag::Delete => {
                output::output_wrapped_row(
                    Some(left_no),
                    Some(line),
                    None,
                    None,
                    LineStatus::Delete.to_str(),
                    code_width,
                    no_width,
                    writer,
                )?;
                left_no += 1;
            }
            ChangeTag::Insert => {
                output::output_wrapped_row(
                    None,
                    None,
                    Some(right_no),
                    Some(line),
                    LineStatus::Insert.to_str(),
                    code_width,
                    no_width,
                    writer,
                )?;
                right_no += 1;
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
        compare_files_table_style(&p1, &p2, 50, 3, &mut w)?;
        Ok(())
    }
}
