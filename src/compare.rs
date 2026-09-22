//! 本模块主要用于进行文本比较
//! 提供
//! 1. 按照文本对照形式输出差异

use crate::ansi_config::{CYAN, GREEN, RED, RESET};
use crate::output;
use crate::row::{self, build_rows};
use anyhow::{Context, Result};
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::io::Write;
use std::path::Path;

/// 对文件名进行终端转义注入处理
fn get_file_name_display_safety(p: &Path, default_name: &str) -> String {
    p.file_name()
        .and_then(|name| name.to_str())
        .map(|name| output::get_sanitized_string(name))
        .unwrap_or(default_name.to_string())
}

/// 按照给定的文件，以表格形式输出两个文本之间的差异
/// - `left`  左文件
/// - `right` 右文件
/// - `code_width` 代码列宽
/// - `no_width` 行号列宽
/// - `writer` 写入对象， `less`或者标准输出等
/// - `inline` 控制是否启用行内比较
/// - `color` 控制是否进行`ANSI`着色
///
/// 通过`build_rows`函数，将`diff`，依据是否开启行内比较，产出每个块内，每行的数据集
///
/// 数据集是以`Row`为元素的容器，每个`Row`都是一行待处理的数据，有左行号，左代码，右行号，右代码，和状态
///
/// 产出的数据集，通过`render_rows`，进行渲染输出
pub fn compare_files_table_style(
    left: &Path,
    right: &Path,
    code_width: usize,
    no_width: usize,
    writer: &mut dyn Write,
    inline: bool,
    color: bool,
) -> Result<()> {
    let left_file_name = get_file_name_display_safety(left, "<left>");
    let right_file_name = get_file_name_display_safety(right, "<right>");

    let left_text = fs::read_to_string(left).with_context(|| {
        format!(
            "打开文件{}出错",
            output::get_sanitized_string(&(left.display().to_string()))
        )
    })?;

    let right_text = fs::read_to_string(right).with_context(|| {
        format!(
            "打开文件{}出错",
            output::get_sanitized_string(&(right.display().to_string()))
        )
    })?;

    let diff = TextDiff::from_lines(&left_text, &right_text);

    output::output_wrapped_header(
        &left_file_name,
        &right_file_name,
        code_width,
        no_width,
        writer,
        color,
    )?;
    output::output_separator_row(code_width * 2 + no_width * 2 + 3 * 4 + 7, writer)?;

    let rows = build_rows(&diff, inline);
    output::render_rows(&rows, code_width, no_width, writer, color)?;

    Ok(())
}
/// 使用`git diff`的样式进行输出
///
/// 没有行内染色的判断，因为当前模式下，都是按行比较
///
/// - `left`  左文件
/// - `right` 右文件
/// - `writer` 写入对象， `less`或者标准输出等
/// - `color` 控制是否进行`ANSI`着色
///
/// 通过`diff.unified_diff`返回的`UnifiedDiff`对象进行处理
pub fn compare_files_git_style(
    left: &Path,
    right: &Path,
    writer: &mut dyn Write,
    color: bool,
) -> Result<()> {
    let left_file_name = get_file_name_display_safety(left, "<left>");
    let right_file_name = get_file_name_display_safety(right, "<right>");

    let left_text = fs::read_to_string(left).with_context(|| {
        format!(
            "打开文件{}出错",
            output::get_sanitized_string(&(left.display().to_string()))
        )
    })?;

    let right_text = fs::read_to_string(right).with_context(|| {
        format!(
            "打开文件{}出错",
            output::get_sanitized_string(&(right.display().to_string()))
        )
    })?;

    let diff = TextDiff::from_lines(&left_text, &right_text);

    let unified_diff = diff.unified_diff();

    let chunks = unified_diff.iter_hunks().collect::<Vec<_>>();
    if chunks.is_empty() {
        return Ok(());
    }

    writeln!(writer, "--- {}", left_file_name)?;
    writeln!(writer, "+++ {}", right_file_name)?;

    for hunk in chunks {
        if color {
            writeln!(writer, "{}{}{}", CYAN, hunk.header(), RESET)?;
        } else {
            writeln!(writer, "{}", hunk.header())?;
        }
        for change in hunk.iter_changes() {
            let text = change.to_string_lossy();
            let text = text.trim_end_matches('\n');
            // 处理终端转义字符的时机应该放到后面，否则换行的\n会被替换导致所有内容均变为一行
            let text = output::get_sanitized_string(text);
            match change.tag() {
                ChangeTag::Equal => {
                    writeln!(writer, " {}", text)?;
                }
                ChangeTag::Insert => {
                    if color {
                        writeln!(writer, "{}+{}{}", GREEN, text, RESET)?;
                    } else {
                        writeln!(writer, "+{}", text)?;
                    }
                }
                ChangeTag::Delete => {
                    if color {
                        writeln!(writer, "{}-{}{}", RED, text, RESET)?;
                    } else {
                        writeln!(writer, "-{}", text)?;
                    }
                }
            }
            if change.missing_newline() {
                writeln!(writer, "{}", row::NO_NEWLINE)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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

    #[test]
    fn test_compare_files_git_style_color() -> Result<()> {
        let base_path = PathBuf::from(r"C:\Users\LFJ\Desktop\Compare");
        let left = base_path.join("left.txt");
        let right = base_path.join("right.txt");

        let mut w = std::io::stdout();

        compare_files_git_style(&left, &right, &mut w, true)?;

        Ok(())
    }
    #[test]
    fn test_compare_files_git_style_no_color() -> Result<()> {
        let base_path = PathBuf::from(r"C:\Users\LFJ\Desktop\Compare");
        let left = base_path.join("left.txt");
        let right = base_path.join("right.txt");

        let mut w = std::io::stdout();

        compare_files_git_style(&left, &right, &mut w, false)?;

        Ok(())
    }
    #[test]
    fn test_compare_files_table_injection() -> Result<()> {
        let base_path = PathBuf::from(r"C:\Users\LFJ\Desktop\Compare");
        let left = base_path.join("left_injection.txt");
        let right = base_path.join("right_injection.txt");

        let mut w = std::io::stdout();

        compare_files_table_style(&left, &right, 70, 3, &mut w, false, true)?;
        Ok(())
    }
}
