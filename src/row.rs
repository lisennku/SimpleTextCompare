//! 将`TextDiff`产出的结果行的抽象为一个`DiffParseRow`实例
//!
//! 枚举`DiffParseRow`有两个变体，分别是`Diff`和`Folded`
//!
//! `Diff`变体是一个单元组，包括一个`Row`结构体实例
//!
//! `Folded`是一个结构体，包括左侧起始位置、右侧起始位置、长度
//!     - `Folded`用于表示`Equal`块
//!
//! ```rust
//! struct Row {
//!     pub left_no:  Option<usize>,
//!     pub right_no: Option<usize>,
//!     pub left_line:     Option<Vec<Segment>>,
//!     pub right_line:    Option<Vec<Segment>>,
//!     pub status:   LineStatus,
//! }
//! ```
//! 结构体`Row`中，存储`TextDiff`中 每个`op`块里**每一行**的数据
//!
//! `*_line`是一个`Option<Vec>`，其内部容器的组成，是Segment-> emphasis， seg_text
//! - 针对`Equal`/`Insert`/`Delete`来说，emphasis都是false，表示不需要内部染色，且容器只有一个元素
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
//!     - 如果启用`--inline`，则将`left_line`/`right_line`均填充值，可能是`None`
//!     - 如果未启用`--inline`，则按照`Insert`/`Delete`处理

use crate::common::{self, Segment};
use crate::consts;
use crate::line_status::LineStatus;
use similar::{ChangeTag, DiffOp, DiffTag, TextDiff};
use std::ops::Range;

/// 功能函数，非`inline`模式时，负责将闭包中的`&str`转为只有一个元素的`Vec`，元素为Segment
fn plain_seg(text: &str) -> Vec<Segment> {
    let ends_with_newline = text.ends_with(['\n', '\r']);

    let no_newline_text = text.trim_end_matches('\n').to_string();
    let mut sanitized_text = common::get_sanitized_string(&no_newline_text);

    if !ends_with_newline {
        sanitized_text.push_str("\n");
        sanitized_text.push_str(consts::NO_NEWLINE);
    }
    vec![Segment::new(false, sanitized_text)]
}

/// 功能函数，将`equal`块的处理抽象为函数
fn equal_row_handle(
    old_range: Range<usize>,
    new_range: Range<usize>,
    diff: &TextDiff<str>,
    rows: &mut Vec<DiffParseRow>,
) {
    for (o, n) in old_range.zip(new_range) {
        let left_no = Some(o + 1);
        let right_no = Some(n + 1);
        let left_line = diff.old_slice(o).map(plain_seg);
        let right_line = diff.new_slice(n).map(plain_seg);

        rows.push(DiffParseRow::Diff(Row::new(
            left_no,
            right_no,
            left_line,
            right_line,
            LineStatus::Equal,
        )))
    }
}

#[derive(Debug)]
pub struct Row {
    pub left_no: Option<usize>,
    pub right_no: Option<usize>,
    pub left_line: Option<Vec<Segment>>,
    pub right_line: Option<Vec<Segment>>,
    pub status: LineStatus,
}

impl Row {
    pub fn new(
        left_no: Option<usize>,
        right_no: Option<usize>,
        left_line: Option<Vec<Segment>>,
        right_line: Option<Vec<Segment>>,
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

/// `DiffParseRow` 枚举
///
/// 用于将差异行和相同行折叠为Hunk的对象统一起来
#[derive(Debug)]
pub enum DiffParseRow {
    Diff(Row),
    Folded {
        left_start: usize,
        right_start: usize,
        count: usize,
    },
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
fn assemble_op_rows_inline_false(
    diff: &TextDiff<str>,
    op: &DiffOp,
    enable_folded: bool,
    folded_radius: usize,
) -> Vec<DiffParseRow> {
    let mut sub_rows: Vec<DiffParseRow> = Vec::new();
    match op.tag() {
        DiffTag::Equal => {
            if !enable_folded {
                equal_row_handle(op.old_range(), op.new_range(), diff, &mut sub_rows);
            } else {
                let equal_total_len = op.old_range().len();
                match equal_total_len
                    .checked_sub(folded_radius.saturating_mul(2)) // 安全处理防止溢出
                    .filter(|&m| m > 0)
                {
                    None => {
                        equal_row_handle(op.old_range(), op.new_range(), diff, &mut sub_rows);
                    }
                    Some(remained) => {
                        let header_old_range =
                            op.old_range().start..op.old_range().start + folded_radius;
                        let header_new_range =
                            op.new_range().start..op.new_range().start + folded_radius;

                        let tail_old_range = op.old_range().end - folded_radius..op.old_range().end;
                        let tail_new_range = op.new_range().end - folded_radius..op.new_range().end;

                        equal_row_handle(header_old_range, header_new_range, diff, &mut sub_rows);

                        sub_rows.push(DiffParseRow::Folded {
                            left_start: op.old_range().start + folded_radius + 1,
                            right_start: op.new_range().start + folded_radius + 1,
                            count: remained,
                        });
                        equal_row_handle(tail_old_range, tail_new_range, diff, &mut sub_rows);
                    }
                }
            }
        }

        tag => {
            if matches!(tag, DiffTag::Delete | DiffTag::Replace) {
                for o in op.old_range() {
                    let left_no = Some(o + 1);
                    let left_line = diff.old_slice(o).map(plain_seg);
                    sub_rows.push(DiffParseRow::Diff(Row::new(
                        left_no,
                        None,
                        left_line,
                        None,
                        LineStatus::Delete,
                    )))
                }
            }

            if matches!(tag, DiffTag::Insert | DiffTag::Replace) {
                for n in op.new_range() {
                    let right_no = Some(n + 1);
                    let right_line = diff.new_slice(n).map(plain_seg);
                    sub_rows.push(DiffParseRow::Diff(Row::new(
                        None,
                        right_no,
                        None,
                        right_line,
                        LineStatus::Insert,
                    )))
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
    let mut left_segs: Vec<(Option<usize>, Option<Vec<Segment>>)> = Vec::new();
    let mut right_segs: Vec<(Option<usize>, Option<Vec<Segment>>)> = Vec::new();

    for inline in diff.iter_inline_changes(op) {
        let mut pieces: Vec<Segment> = inline
            .values()
            .iter()
            .map(|(b, s)| Segment::new(*b, common::get_sanitized_string(s.trim_end_matches('\n'))))
            .collect();
        if inline.missing_newline() {
            pieces.push(Segment::new(false, "\n".to_string() + consts::NO_NEWLINE))
        }

        match inline.tag() {
            ChangeTag::Delete => left_segs.push((inline.old_index().map(|i| i + 1), Some(pieces))),
            ChangeTag::Insert => right_segs.push((inline.new_index().map(|i| i + 1), Some(pieces))),
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
/// 负责从`TextDiff`中解析出一个`Vec<DiffParseRow>`
///
/// `TextDiff`是一个结构体，内部存储对于`old`/`new`的字符串的引用，可以通过`old_slice()`/`new_slice()`获取索引的文本引用
///
/// `diff.ops()`返回一个`&[DiffOp]`
///
/// `DiffOp`中并不实际存储字符串，而是通过`.new_range()`/`old_range()`获取对应索引
///
/// 启用`inline`参数，`Replace`块对应的每行`status`都是`Replace`，如果未启用`inline`则对应的是`Delete`/`Insert`
pub fn build_rows(
    diff: &TextDiff<str>,
    inline: bool,
    enable_folded: bool,
    folded_radius: usize,
) -> Vec<DiffParseRow> {
    let mut rows: Vec<DiffParseRow> = Vec::new();
    for op in diff.ops() {
        if inline && matches!(op.tag(), DiffTag::Replace) {
            rows.extend(
                assemble_op_rows_inline_true(diff, op)
                    .into_iter()
                    .map(DiffParseRow::Diff),
            );
        } else {
            rows.extend(assemble_op_rows_inline_false(
                diff,
                op,
                enable_folded,
                folded_radius,
            ));
        }
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 7 行完全相同的文本，`old`/`new`一致，构成一个长度为7的`Equal`块
    fn seven_same_lines() -> String {
        (1..=7).map(|i| format!("{}\n", i)).collect()
    }

    /// 关闭折叠时，长度为7的`Equal`块应展开为7个`Diff`行，行号左右一致
    #[test]
    fn fold_disabled_expands_all_equal() {
        let text = seven_same_lines();
        let diff = TextDiff::from_lines(&text, &text);
        let rows = build_rows(&diff, false, false, 1);

        assert_eq!(rows.len(), 7);
        for (i, row) in rows.iter().enumerate() {
            match row {
                DiffParseRow::Diff(r) => {
                    assert!(matches!(r.status, LineStatus::Equal));
                    assert_eq!(r.left_no, Some(i + 1));
                    assert_eq!(r.right_no, Some(i + 1));
                }
                _ => panic!("folding disabled should yield only Diff rows"),
            }
        }
    }

    /// radius=1 时，7行的`Equal`块折叠为：1个头行 + 1个`Folded` + 1个尾行
    #[test]
    fn fold_long_run_head_fold_tail() {
        let text = seven_same_lines();
        let diff = TextDiff::from_lines(&text, &text);
        let rows = build_rows(&diff, false, true, 1);

        assert_eq!(rows.len(), 3);

        match &rows[0] {
            DiffParseRow::Diff(r) => {
                assert!(matches!(r.status, LineStatus::Equal));
                assert_eq!(r.left_no, Some(1));
            }
            _ => panic!("row 0 should be the head Diff row"),
        }

        match &rows[1] {
            DiffParseRow::Folded {
                left_start,
                right_start,
                count,
            } => {
                assert_eq!(*left_start, 2);
                assert_eq!(*right_start, 2);
                assert_eq!(*count, 5); // 7 - 2*1
            }
            _ => panic!("row 1 should be the Folded marker"),
        }

        match &rows[2] {
            DiffParseRow::Diff(r) => {
                assert!(matches!(r.status, LineStatus::Equal));
                assert_eq!(r.left_no, Some(7));
            }
            _ => panic!("row 2 should be the tail Diff row"),
        }
    }

    /// radius=3 时，7行的`Equal`块仍折叠，但头尾各占3行，中间只剩1行被折叠
    #[test]
    fn fold_radius_controls_count() {
        let text = seven_same_lines();
        let diff = TextDiff::from_lines(&text, &text);
        let rows = build_rows(&diff, false, true, 3);

        // 3 head + 1 folded + 3 tail
        assert_eq!(rows.len(), 7);

        let folded = rows.iter().find_map(|row| match row {
            DiffParseRow::Folded {
                left_start, count, ..
            } => Some((*left_start, *count)),
            _ => None,
        });
        assert_eq!(folded, Some((4, 1))); // left_start = 0+3+1, count = 7 - 2*3
    }

    /// 边界：`Equal`块长度恰好等于`2*radius`时不折叠（`.filter(|&m| m>0)`拒绝余数0）
    #[test]
    fn no_fold_when_run_equals_2radius() {
        let text: String = (1..=6).map(|i| format!("{}\n", i)).collect();
        let diff = TextDiff::from_lines(&text, &text);
        let rows = build_rows(&diff, false, true, 3);

        assert_eq!(rows.len(), 6);
        assert!(rows
            .iter()
            .all(|row| matches!(row, DiffParseRow::Diff(_))));
        assert!(!rows
            .iter()
            .any(|row| matches!(row, DiffParseRow::Folded { .. })));
    }

    /// inline模式下，`Replace`块产出单个`Replace`行，左右行号都存在
    #[test]
    fn inline_replace_status_is_replace() {
        let old = "hello world\n";
        let new = "hello WORLD\n";
        let diff = TextDiff::from_lines(old, new);
        let rows = build_rows(&diff, true, false, 3);

        assert_eq!(rows.len(), 1);
        match &rows[0] {
            DiffParseRow::Diff(r) => {
                assert!(matches!(r.status, LineStatus::Replace));
                assert_eq!(r.left_no, Some(1));
                assert_eq!(r.right_no, Some(1));
            }
            _ => panic!("expected a single inline Replace row"),
        }
    }

    /// 非inline模式下，`Replace`块拆成先`Delete`后`Insert`两行
    #[test]
    fn non_inline_replace_is_delete_then_insert() {
        let old = "x\n";
        let new = "y\n";
        let diff = TextDiff::from_lines(old, new);
        let rows = build_rows(&diff, false, false, 1);

        assert_eq!(rows.len(), 2);
        match &rows[0] {
            DiffParseRow::Diff(r) => {
                assert!(matches!(r.status, LineStatus::Delete));
                assert_eq!(r.left_no, Some(1));
                assert_eq!(r.right_no, None);
            }
            _ => panic!("row 0 should be the Delete row"),
        }
        match &rows[1] {
            DiffParseRow::Diff(r) => {
                assert!(matches!(r.status, LineStatus::Insert));
                assert_eq!(r.left_no, None);
                assert_eq!(r.right_no, Some(1));
            }
            _ => panic!("row 1 should be the Insert row"),
        }
    }
}
