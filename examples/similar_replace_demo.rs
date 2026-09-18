use similar::TextDiff;
fn main() {
    let s1 = String::from("aa bb cc dd\nff ee gg hh");
    let s2 = String::from("ab dd bc de\nfg ef hh gg");
    let diff = TextDiff::from_words(&s1, &s2);
    for item in diff.ops() {
        println!("{:?}", item);
    }

    for item in diff.iter_all_changes() {
        println!("{:?}", item);
    }
}
