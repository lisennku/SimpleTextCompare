//! 用于提供描述行状态的枚举
//!

/// 表示每行的状态
/// - `Equal` 表示左右一致
/// - `Delete` 表示右文件较于左文件 删除了改行
/// - `Insert` 表示右文件较于左文件 新增了改行
use crate::ansi_config::{GREEN, RED, RESET};

#[derive(Copy, Clone)]
#[allow(dead_code)]
pub enum LineStatus {
    Equal,
    Delete,
    Insert,
    Replace,
}

impl LineStatus {
    pub fn to_str(&self) -> &str {
        match self {
            LineStatus::Equal => "Equal",
            LineStatus::Delete => "Delete",
            LineStatus::Insert => "Insert",
            LineStatus::Replace => "Replace",
        }
    }

    pub fn wrap_ansi(&self, input: &str, color: bool) -> String {
        if !color {
            return format!("{}", input);
        }
        match self {
            LineStatus::Equal => format!("{input}"),
            LineStatus::Insert => format!("{GREEN}{input}{RESET}"),
            LineStatus::Delete => format!("{RED}{input}{RESET}"),
            LineStatus::Replace => unreachable!("do not use this"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_status_equal_test() {
        let e = LineStatus::Equal;
        assert_eq!(e.to_str(), "Equal");
    }
    #[test]
    fn line_status_delete_test() {
        let e = LineStatus::Delete;
        assert_eq!(e.to_str(), "Delete");
    }
    #[test]
    fn line_status_insert_test() {
        let e = LineStatus::Insert;
        assert_eq!(e.to_str(), "Insert");
    }
    #[test]
    fn line_status_wrap_ansi_test() {
        let d = LineStatus::Delete;
        let i = LineStatus::Insert;
        let e = LineStatus::Equal;

        println!("{}", d.wrap_ansi("this line is deleted", true));
        println!("{}", i.wrap_ansi("this line is inserted", true));
        println!("{}", e.wrap_ansi("this line is not changed", true));
    }
}
