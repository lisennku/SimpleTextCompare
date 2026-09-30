use crossterm::{execute, terminal};
use std::io::{self, Write, stdout};

fn main() -> io::Result<()> {
    let mut stdout = stdout();
    let original_size = terminal::size()?;
    println!("Original size: {} x {}", original_size.0, original_size.1);

    execute!(stdout, terminal::SetSize(20, 20), terminal::ScrollUp(10))?;
    println!(
        "Current size: {} x {}",
        terminal::size()?.0,
        terminal::size()?.1
    );
    execute!(stdout, terminal::SetSize(original_size.0, original_size.1))?;
    println!(
        "restore to original size {} x {}",
        terminal::size()?.0,
        terminal::size()?.1
    );
    Ok(())
}
