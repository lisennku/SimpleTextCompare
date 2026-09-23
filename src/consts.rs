/// 指定配置文件名
pub const CONFIG_FILE_NAME: &str = r"stc.toml";
/// 指定存放配置文件的目录名
pub const CONFIG_FOLDER_NAME: &str = "stc_conf";
/// 指定文件最大字节数的限制 1GiB
pub const FILE_MAX_BYTES: u64 = 1024 * 1024 * 1024;

/// table输出方式下，最后一列状态的宽度
pub const STATUS_WIDTH: usize = 7;

/// diff后文件末尾无新行统一提示
pub const NO_NEWLINE: &str = r"\ No newline at end of file";
