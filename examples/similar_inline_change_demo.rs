use similar::{ChangeTag, DiffTag, TextDiff};
use std::fs;
use std::path::PathBuf;

fn main() {
    // let s1 = String::from("same1\ndelete_me\nsame2\nreplace_old\nsame3\nsame4\n");
    // let s2 = String::from("same1\nsame2\nreplace_new\nsame3\ninsert_me\nsame4\n");
    // let diff = TextDiff::from_lines(&s1, &s2);

    let base_path = PathBuf::from(r"C:\Users\LFJ\Desktop\Compare");
    let left = base_path.join("left.txt");
    let right = base_path.join("right.txt");

    let left_str = fs::read_to_string(&left).unwrap();
    let right_str = fs::read_to_string(&right).unwrap();

    let diff = TextDiff::from_lines(&left_str, &right_str);
    let mut cnt = 0_usize;
    for op in diff.ops() {
        println!("tag is {:#?}", op.tag());
        let iter = diff.iter_inline_changes(op);
        for change in iter {
            cnt += 1;
            println!("{:?}", change);
        }
        println!("cnt is {cnt}");
    }

    // let base_path = PathBuf::from(r"C:\Users\LFJ\Desktop\Compare");
    // let left = base_path.join("left.txt");
    // let right = base_path.join("right.txt");
    //
    // let left_str = fs::read_to_string(&left).unwrap();
    // let right_str = fs::read_to_string(&right).unwrap();
    //
    // let diff = TextDiff::from_lines(&left_str, &right_str);
}
