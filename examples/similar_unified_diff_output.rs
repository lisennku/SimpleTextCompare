use similar::TextDiff;
use std::fs;
use std::path::PathBuf;
fn main() {
    let base_path = PathBuf::from(r"C:\Users\LFJ\Desktop\Compare");
    let left = base_path.join("left.txt");
    let right = base_path.join("right.txt");

    let left_str = fs::read_to_string(&left).unwrap();
    let right_str = fs::read_to_string(&right).unwrap();

    let diff = TextDiff::from_lines(&left_str, &right_str);

    let mut unified_diff = diff.unified_diff();
    unified_diff.context_radius(4);
    for hunk in unified_diff.iter_hunks() {
        println!("{}", hunk.header());
        println!("{:?}", hunk.ops());
    }
}
