//! 用于提供描述行状态的枚举
//!

use crate::ansi_config::{GREEN, RED, YELLOW};
/// 表示每行的状态
/// - `Equal` 表示左右一致
/// - `Delete` 表示右文件较于左文件 删除了该行
/// - `Insert` 表示右文件较于左文件 新增了该行
/// - `Replace` 表示左右文件不同
#[derive(Copy, Clone, Debug)]
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

    /// 返回着色的值
    ///
    /// 如果不需要着色，`color=false`，返回None
    ///
    /// 针对`Equal` 不需要着色
    ///
    /// 针对`Delete`/`Insert`，不考虑`is_emphasis`，整行着色
    ///
    /// 只有`Replace`时才考虑
    pub fn piece_color(&self, color: bool, is_emphasis: bool) -> Option<&str> {
        if !color {
            return None;
        }
        match self {
            LineStatus::Equal => None,
            LineStatus::Delete => Some(RED),
            LineStatus::Insert => Some(GREEN),
            LineStatus::Replace => {
                if is_emphasis {
                    Some(YELLOW)
                } else {
                    None
                }
            }
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
}
