#![allow(dead_code)]

pub const RESET: &str = "\x1b[0m";

// 标准前景色
pub const BLACK: &str = "\x1b[30m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const WHITE: &str = "\x1b[37m";

// 亮色前景
pub const BRIGHT_BLACK: &str = "\x1b[90m";
pub const BRIGHT_RED: &str = "\x1b[91m";
pub const BRIGHT_GREEN: &str = "\x1b[92m";
pub const BRIGHT_YELLOW: &str = "\x1b[93m";
pub const BRIGHT_BLUE: &str = "\x1b[94m";
pub const BRIGHT_MAGENTA: &str = "\x1b[95m";
pub const BRIGHT_CYAN: &str = "\x1b[96m";
pub const BRIGHT_WHITE: &str = "\x1b[97m";

// 文本样式
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const ITALIC: &str = "\x1b[3m";
pub const UNDERLINE: &str = "\x1b[4m";
pub const BLINK: &str = "\x1b[5m";
pub const REVERSE: &str = "\x1b[7m";
pub const HIDDEN: &str = "\x1b[8m";
pub const STRIKETHROUGH: &str = "\x1b[9m";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ansi_display() {
        println!("--- 标准前景色 ---");
        println!("{:<20} {BLACK}████ 示例文本{RESET}", "30 黑色");
        println!("{:<20} {RED}████ 示例文本{RESET}", "31 红色");
        println!("{:<20} {GREEN}████ 示例文本{RESET}", "32 绿色");
        println!("{:<20} {YELLOW}████ 示例文本{RESET}", "33 黄色");
        println!("{:<20} {BLUE}████ 示例文本{RESET}", "34 蓝色");
        println!("{:<20} {MAGENTA}████ 示例文本{RESET}", "35 洋红");
        println!("{:<20} {CYAN}████ 示例文本{RESET}", "36 青色");
        println!("{:<20} {WHITE}████ 示例文本{RESET}", "37 白色");

        println!("\n--- 亮色前景 ---");
        println!("{:<20} {BRIGHT_BLACK}████ 示例文本{RESET}", "90 亮黑");
        println!("{:<20} {BRIGHT_RED}████ 示例文本{RESET}", "91 亮红");
        println!("{:<20} {BRIGHT_GREEN}████ 示例文本{RESET}", "92 亮绿");
        println!("{:<20} {BRIGHT_YELLOW}████ 示例文本{RESET}", "93 亮黄");
        println!("{:<20} {BRIGHT_BLUE}████ 示例文本{RESET}", "94 亮蓝");
        println!("{:<20} {BRIGHT_MAGENTA}████ 示例文本{RESET}", "95 亮洋红");
        println!("{:<20} {BRIGHT_CYAN}████ 示例文本{RESET}", "96 亮青");
        println!("{:<20} {BRIGHT_WHITE}████ 示例文本{RESET}", "97 亮白");

        println!("\n--- 文本样式 ---");
        println!("{:<20} {BOLD}示例文本{RESET}", "1 加粗");
        println!("{:<20} {DIM}示例文本{RESET}", "2 暗淡");
        println!("{:<20} {ITALIC}示例文本{RESET}", "3 斜体");
        println!("{:<20} {UNDERLINE}示例文本{RESET}", "4 下划线");
        println!("{:<20} {BLINK}示例文本{RESET}", "5 闪烁");
        println!("{:<20} {REVERSE}示例文本{RESET}", "7 反显");
        println!("{:<20} {HIDDEN}示例文本{RESET}", "8 隐藏");
        println!("{:<20} {STRIKETHROUGH}示例文本{RESET}", "9 删除线");

        println!("\n--- 重置 ---");

        println!("{RED}红色 {RESET}默认颜色");

        println!(
            "{RED}红色默认前景色 {}\x1b[39m，背景保持 {RESET}",
            "\x1b[39m"
        );

        println!(
            "{RED}\x1b[44m红字蓝底 {}\x1b[49m，恢复背景 {RESET}",
            "\x1b[49m"
        );

        println!("{BOLD}加粗 {}\x1b[22m取消加粗 {RESET}", "\x1b[22m");
        println!("{UNDERLINE}下划线 {}\x1b[24m取消下划线 {RESET}", "\x1b[24m");
        println!("{REVERSE}反显 {}\x1b[27m取消反显 {RESET}", "\x1b[27m");
    }
}
