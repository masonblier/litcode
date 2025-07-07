mod skip_list;

/// main
fn main() {
    // skip list
    let nums: [i64; 10] = [3, 1, 9, 12, 11, 16, 99, 18, 7, 22];

    let mut skip_list = skip_list::SkipList::default();
    for n in &nums {
        skip_list.insert(*n);
    }

    println!("SkipList");
    println!("");
    println!("{}", skip_list);
    println!("");
    println!("  contains 3? {}", skip_list.contains(3));
    println!("  contains 4? {}", skip_list.contains(4));
    println!("");
}
