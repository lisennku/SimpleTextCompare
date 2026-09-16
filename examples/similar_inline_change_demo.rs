use similar::{ChangeTag, TextDiff};

fn main() {
    let s1 = String::from("aa bb cc dd");
    let s2 = String::from("ab dd bc de");
    let diff = TextDiff::from_words(&s1, &s2);
    for item in diff.iter_all_inline_changes() {
        println!("{:?}", item);
    }
    println!("\x1b[31mA");
    println!("\x1b[32mB");
    println!("\x1b[0mC");
}
