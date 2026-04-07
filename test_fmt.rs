fn main() {
    use std::fmt::Write;
    let mut s = String::new();
    write!(&mut s, "hello {}", 1).unwrap();
    println!("{}", s);
}
