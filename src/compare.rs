//!
use crate::line_status::LineStatus;
use crate::output;
use anyhow::{Context, Result};
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::PathBuf;

///
pub fn compare_files_table_style(left: &PathBuf, right: &PathBuf) -> Result<()> {
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

    let code_width = 50_usize;
    let no_width = 6_usize;

    output::output_wrapped_header(left_file_name, right_file_name, code_width, no_width);
    output::output_separator_row(code_width * 3 + no_width * 2 + 3 * 4);

    let mut left_no = 1_usize;
    let mut right_no = 1_usize;

    for change in diff.iter_all_changes() {
        let line = change.value().trim_end_matches('\n').trim_end_matches('\r');
        match change.tag() {
            ChangeTag::Equal => {
                output::output_wrapped_row(
                    Some(left_no),
                    line,
                    Some(right_no),
                    line,
                    LineStatus::Equal.to_str(),
                    code_width,
                    no_width,
                );
                left_no += 1;
                right_no += 1;
            }
            ChangeTag::Delete => {
                output::output_wrapped_row(
                    Some(left_no),
                    line,
                    None,
                    "",
                    LineStatus::Delete.to_str(),
                    code_width,
                    no_width,
                );
                left_no += 1;
            }
            ChangeTag::Insert => {
                output::output_wrapped_row(
                    None,
                    "",
                    Some(right_no),
                    line,
                    LineStatus::Insert.to_str(),
                    code_width,
                    no_width,
                );
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
        let p1 = PathBuf::from(r"C:\Users\LFJ\Desktop\new.txt");
        let s = fs::read_to_string(p1)?;
        for (idx, line) in s.lines().enumerate() {
            println!("{}:{}+", idx + 1, line);
        }
        Ok(())
    }
    #[test]
    fn test_compare_files_table_style() -> Result<()> {
        let p1 = PathBuf::from(
            r"D:\vscode_workspace\vscode_workspace\codes_rust\rust_learn\text_compare_cli\base.txt",
        );
        let p2 = PathBuf::from(
            r"D:\vscode_workspace\vscode_workspace\codes_rust\rust_learn\text_compare_cli\comp.txt",
        );

        compare_files_table_style(&p1, &p2)?;
        Ok(())
    }
}
