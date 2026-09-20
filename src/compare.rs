//! 本模块主要用于进行文本比较
//! 提供
//! 1. 按照文本对照形式输出差异

use crate::output;
use crate::output::render_rows;
use crate::row::build_rows;
use anyhow::{Context, Result};
use similar::TextDiff;
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

    let rows = build_rows(&diff, inline);
    render_rows(&rows, code_width, no_width, writer, color)?;

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
