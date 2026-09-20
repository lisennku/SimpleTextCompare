//! `TextDiff`产出的结果行的抽象为一个`Row`实例
//!
//! ```rust
//! struct Row {
//!     pub left_no:  Option<usize>,
//!     pub right_no: Option<usize>,
//!     pub left_line:     Option<Vec<(bool, String)>>,
//!     pub right_line:    Option<Vec<(bool, String)>>,
//!     pub status:   LineStatus,
//! }
//! ```
//! 结构体`Row`中，存储`TextDiff`中 每个`op`块里**每一行**的数据
//!
//! `*_line`是一个`Option<Vec>`，其内部容器的组成，是`(bool, String)`元组
//! - 针对`Equal`/`Insert`/`Delete`来说，bool都是false，表示不需要内部染色，且容器只有一个元素
//! - 针对`Replace`，
//!     - 如果未启用`--inline`，则和`Insert`/`Delete`存储内容类型一致
//!     - 如果启用`--inline`，元组的取值来自`TextDiff.iter_inline_changes(op)`的`values()`，容器内可能有多个元素
//!
//! 字段值
//!
//! - 针对`Equal`类型
//!     - 行号可能不同，但是left/right的代码是一致的
//! - 针对`Delete`类型
//!     - `right_*`都为`None`
//! - 针对`Insert`类型
//!     - `left_*`都为`None`
//! - 针对`Replace`类型
//!     - 如果启用`--inline`，则将`left_line`/`right_line`均填充值
//!     - 如果未启用`--inline`，则按照`Insert`/`Delete`处理
use crate::line_status::LineStatus;
use similar::{ChangeTag, DiffOp, DiffTag, TextDiff};

pub fn plain_seg(text: &str) -> Vec<(bool, String)> {
    vec![(
        false,
        text.trim_end_matches('\n')
            .trim_end_matches('\r')
            .to_string(),
    )]
}

#[derive(Debug)]
pub struct Row {
    pub left_no: Option<usize>,
    pub right_no: Option<usize>,
    pub left_line: Option<Vec<(bool, String)>>,
    pub right_line: Option<Vec<(bool, String)>>,
    pub status: LineStatus,
}

impl Row {
    pub fn new(
        left_no: Option<usize>,
        right_no: Option<usize>,
        left_line: Option<Vec<(bool, String)>>,
        right_line: Option<Vec<(bool, String)>>,
        status: LineStatus,
    ) -> Row {
        Row {
            left_no,
            right_no,
            left_line,
            right_line,
            status,
        }
    }
}

/// 用于对`Delete`/`Insert`/`Equal`块和`inline=false`时的Replace块进行拼接
///
/// 输入：TextDiff的引用，op块
///
/// 输出：`Vec<Row>`
///
/// 每个`op`块内部，通过`old_range`/`new_range`作为索引，获取对应文本，本身+1作为行号
///
/// 获取到的文本和`false`组装作为一个元组
fn assemble_op_rows_inline_false(diff: &TextDiff<str>, op: &DiffOp) -> Vec<Row> {
    let mut sub_rows: Vec<Row> = Vec::new();
    match op.tag() {
        DiffTag::Equal => {
            for (o, n) in op.old_range().zip(op.new_range()) {
                let left_no = Some(o + 1);
                let right_no = Some(n + 1);
                let left_line = diff.old_slice(o).map(plain_seg);
                let right_line = diff.new_slice(n).map(plain_seg);

                sub_rows.push(Row::new(
                    left_no,
                    right_no,
                    left_line,
                    right_line,
                    LineStatus::Equal,
                ))
            }
        }

        tag => {
            if matches!(tag, DiffTag::Delete | DiffTag::Replace) {
                for o in op.old_range() {
                    let left_no = Some(o + 1);
                    let left_line = diff.old_slice(o).map(plain_seg);
                    sub_rows.push(Row::new(left_no, None, left_line, None, LineStatus::Delete))
                }
            }

            if matches!(tag, DiffTag::Insert | DiffTag::Replace) {
                for n in op.new_range() {
                    let right_no = Some(n + 1);
                    let right_line = diff.new_slice(n).map(plain_seg);
                    sub_rows.push(Row::new(
                        None,
                        right_no,
                        None,
                        right_line,
                        LineStatus::Insert,
                    ))
                }
            }
        }
    }

    sub_rows
}

/// 用于对`inline=true`时的`Replace`块进行拆解
///
/// 通过`ChangeTag`区分迭代`iter_inline_changes`返回的迭代器的数据，分别放到`left_segs`和`right_segs`
///
/// 根据最长的一个进行循环处理，将`left`和`right`放到一行
///
fn assemble_op_rows_inline_true(diff: &TextDiff<str>, op: &DiffOp) -> Vec<Row> {
    let mut sub_rows: Vec<Row> = Vec::new();
    let mut left_segs: Vec<(Option<usize>, Option<Vec<(bool, String)>>)> = Vec::new();
    let mut right_segs: Vec<(Option<usize>, Option<Vec<(bool, String)>>)> = Vec::new();

    for inline in diff.iter_inline_changes(op) {
        match inline.tag() {
            ChangeTag::Delete => left_segs.push((
                inline.old_index().map(|i| i + 1),
                Some(
                    inline
                        .values()
                        .iter()
                        .map(|(b, s)| {
                            (
                                *b,
                                s.trim_end_matches('\n').trim_end_matches('\r').to_string(),
                            )
                        })
                        .collect(),
                ),
            )),
            ChangeTag::Insert => right_segs.push((
                inline.new_index().map(|i| i + 1),
                Some(
                    inline
                        .values()
                        .iter()
                        .map(|(b, s)| {
                            (
                                *b,
                                s.trim_end_matches('\n').trim_end_matches('\r').to_string(),
                            )
                        })
                        .collect(),
                ),
            )),
            _ => unreachable!(),
        }
    }

    let max_lines_cnt = left_segs.len().max(right_segs.len()).max(1);
    for i in 0..max_lines_cnt {
        // 使用get_mut是为了通过take获取owned数据，否则无法获取对应类型的数据
        let (left_no, left_line) = match left_segs.get_mut(i) {
            Some((no, line)) => (no.take(), line.take()),
            None => (None, None),
        };
        let (right_no, right_line) = match right_segs.get_mut(i) {
            Some((no, line)) => (no.take(), line.take()),
            None => (None, None),
        };

        sub_rows.push(Row::new(
            left_no,
            right_no,
            left_line,
            right_line,
            LineStatus::Replace,
        ))
    }

    sub_rows
}
/// 负责从`TextDiff`中解析出一个`Vec<Row>`
///
/// `TextDiff`是一个结构体，内部存储对于`old`/`new`的字符串的引用，可以通过`old_slice()`/`new_slice()`获取索引的文本引用
///
/// `diff.ops()`返回一个`&[DiffOp]`
///
/// `DiffOp`中并不实际存储字符串，而是通过`.new_range()`/`old_range()`获取对应索引
///
/// 启用`inline`参数，`Replace`块对应的每行`status`都是`Replace`，如果未启用`inline`则对应的是`Delete`/`Insert`
pub fn build_rows(diff: &TextDiff<str>, inline: bool) -> Vec<Row> {
    let mut rows: Vec<Row> = Vec::new();
    for op in diff.ops() {
        if inline && matches!(op.tag(), DiffTag::Replace) {
            rows.extend(assemble_op_rows_inline_true(diff, op));
        } else {
            rows.extend(assemble_op_rows_inline_false(diff, op));
        }
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    #[test]
    fn row_test_for_build_rows() {
        let base_path = PathBuf::from(r"C:\Users\LFJ\Desktop\Compare");
        let left = base_path.join("left.txt");
        let right = base_path.join("right.txt");

        let left_str = fs::read_to_string(&left).unwrap();
        let right_str = fs::read_to_string(&right).unwrap();

        let diff = TextDiff::from_lines(&left_str, &right_str);
        let res = build_rows(&diff, false);
        for item in res {
            println!("{:?}", item);
        }
    }

    #[test]
    fn row_test_for_build_rows_inline() {
        let base_path = PathBuf::from(r"C:\Users\LFJ\Desktop\Compare");
        let left = base_path.join("left.txt");
        let right = base_path.join("right.txt");

        let left_str = fs::read_to_string(&left).unwrap();
        let right_str = fs::read_to_string(&right).unwrap();

        let diff = TextDiff::from_lines(&left_str, &right_str);
        let res = build_rows(&diff, true);
        for item in res {
            println!("{:?}", item);
        }
    }
}
