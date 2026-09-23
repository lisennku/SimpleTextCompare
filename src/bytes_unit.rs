use crate::consts;
use anyhow::{Result, anyhow, bail};

#[derive(Debug, PartialEq)]
pub enum BytesUnit {
    Bytes,
    KiBytes,
    MiBytes,
    GiBytes,
}

impl BytesUnit {
    pub fn new(suffixes: &str) -> Result<Self> {
        let mut suffixes = suffixes.to_uppercase();
        if suffixes.is_empty() {
            suffixes = String::from("B");
        }

        if consts::KB_ALIAS.contains(&suffixes.as_str()) {
            Ok(BytesUnit::KiBytes)
        } else if consts::MB_ALIAS.contains(&suffixes.as_str()) {
            Ok(BytesUnit::MiBytes)
        } else if consts::GB_ALIAS.contains(&suffixes.as_str()) {
            Ok(BytesUnit::GiBytes)
        } else if consts::BYTES_ALIAS.contains(&suffixes.as_str()) {
            Ok(BytesUnit::Bytes)
        } else {
            bail!("Unknown bytes suffixes: {}", suffixes);
        }
    }

    fn from_bytes(bytes: u64) -> Self {
        let all_form = [Self::GiBytes, Self::MiBytes, Self::KiBytes, Self::Bytes];
        all_form
            .into_iter()
            .find(|u| bytes >= u.multiplier() && bytes % u.multiplier() == 0)
            .unwrap_or(BytesUnit::Bytes)
    }

    pub fn display(limit_bytes: u64) -> String {
        let unit = Self::from_bytes(limit_bytes);
        format!("{}{}", limit_bytes / unit.multiplier(), unit.label())
    }

    fn label(&self) -> String {
        match self {
            BytesUnit::GiBytes => String::from("GiB"),
            BytesUnit::MiBytes => String::from("MiB"),
            BytesUnit::KiBytes => String::from("KiB"),
            BytesUnit::Bytes => String::from("B"),
        }
    }
    fn multiplier(&self) -> u64 {
        match self {
            BytesUnit::Bytes => 1,
            BytesUnit::KiBytes => 1024,
            BytesUnit::MiBytes => 1024 * 1024,
            BytesUnit::GiBytes => 1024 * 1024 * 1024,
        }
    }

    pub fn multiply(&self, nums: u64) -> Result<u64> {
        nums.checked_mul(self.multiplier())
            .ok_or_else(|| anyhow!("Overflow"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_new() {
        assert_eq!(
            BytesUnit::Bytes,
            BytesUnit::new(consts::BYTES_ALIAS[0]).unwrap()
        );
        assert_eq!(
            BytesUnit::KiBytes,
            BytesUnit::new(consts::KB_ALIAS[0]).unwrap()
        );
        assert_eq!(BytesUnit::KiBytes, BytesUnit::new("kib").unwrap());
        assert_eq!(
            BytesUnit::MiBytes,
            BytesUnit::new(consts::MB_ALIAS[0]).unwrap()
        );
        assert_eq!(
            BytesUnit::GiBytes,
            BytesUnit::new(consts::GB_ALIAS[0]).unwrap()
        );
    }

    #[test]
    fn test_bytes_edge_cases() {
        assert_eq!(BytesUnit::new("").unwrap(), BytesUnit::Bytes);
        assert_eq!(BytesUnit::new("m").unwrap(), BytesUnit::MiBytes);
        assert_eq!(BytesUnit::new("gb").unwrap(), BytesUnit::GiBytes);
        assert_eq!(BytesUnit::new("K").unwrap(), BytesUnit::new("KB").unwrap());
        assert_eq!(
            BytesUnit::new("KB").unwrap(),
            BytesUnit::new("KiB").unwrap()
        );
        assert_eq!(BytesUnit::new("M").unwrap(), BytesUnit::new("MiB").unwrap());
        assert!(BytesUnit::new("X").is_err());
        assert!(BytesUnit::new("XYZ").is_err());
    }

    #[test]
    fn test_bytes_from_bytes() {
        assert_eq!(BytesUnit::from_bytes(1073741824), BytesUnit::GiBytes);
        assert_eq!(BytesUnit::from_bytes(52428800), BytesUnit::MiBytes);
        assert_eq!(BytesUnit::from_bytes(1048576), BytesUnit::MiBytes);
        assert_eq!(BytesUnit::from_bytes(1024), BytesUnit::KiBytes);
        assert_eq!(BytesUnit::from_bytes(5460000), BytesUnit::Bytes);
        assert_eq!(BytesUnit::from_bytes(10485761), BytesUnit::Bytes);
        assert_eq!(BytesUnit::from_bytes(1536), BytesUnit::Bytes);
        assert_eq!(BytesUnit::from_bytes(0), BytesUnit::Bytes);
    }

    #[test]
    fn test_bytes_multiply() {
        assert_eq!(BytesUnit::Bytes.multiply(500).unwrap(), 500);
        assert_eq!(BytesUnit::KiBytes.multiply(1).unwrap(), 1024);
        assert_eq!(BytesUnit::MiBytes.multiply(10).unwrap(), 10_485_760);
        assert_eq!(BytesUnit::GiBytes.multiply(1).unwrap(), 1_073_741_824);
    }

    #[test]
    fn test_bytes_multiply_overflow() {
        assert!(BytesUnit::GiBytes.multiply(u64::MAX).is_err());
        assert!(BytesUnit::MiBytes.multiply(u64::MAX).is_err());
    }

    #[test]
    fn test_bytes_display() {
        assert_eq!(BytesUnit::display(1073741824), "1GiB");
        assert_eq!(BytesUnit::display(52428800), "50MiB");
        assert_eq!(BytesUnit::display(10485760), "10MiB");
        assert_eq!(BytesUnit::display(1024), "1KiB");
        assert_eq!(BytesUnit::display(5460000), "5460000B");
        assert_eq!(BytesUnit::display(10485761), "10485761B");
        assert_eq!(BytesUnit::display(0), "0B");
        assert_eq!(BytesUnit::display(500), "500B");
    }
}
