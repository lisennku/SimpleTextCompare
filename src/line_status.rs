//! 用于提供描述行状态的枚举
//!

/// 表示每行的状态
/// - `Equal` 表示左右一致
/// - `Delete` 表示右文件较于左文件 删除了改行
/// - `Insert` 表示右文件较于左文件 新增了改行
pub enum LineStatus {
    Equal,
    Delete,
    Insert,
}

impl LineStatus {
    pub fn to_str(&self) -> &str {
        match self {
            LineStatus::Equal => "Equal",
            LineStatus::Delete => "Delete",
            LineStatus::Insert => "Insert",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_test() {
        let e = LineStatus::Equal;
        assert_eq!(e.to_str(), "Equal");
    }
    #[test]
    fn delete_test() {
        let e = LineStatus::Delete;
        assert_eq!(e.to_str(), "Delete");
    }
    #[test]
    fn insert_test() {
        let e = LineStatus::Insert;
        assert_eq!(e.to_str(), "Insert");
    }
}
