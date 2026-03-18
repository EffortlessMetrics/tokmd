fn main() {
    let mut vec = vec![(1, "b"), (1, "a"), (2, "c")];
    vec.sort_by_key(|i| i.0);
    println!("{:?}", vec);
}
