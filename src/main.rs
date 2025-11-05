fn main() {
    let input = std::env::args().nth(1).unwrap_or("".to_string());
    println!("{}", md5::convert(&input));
}
