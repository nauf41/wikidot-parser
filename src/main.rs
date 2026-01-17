fn read<T: std::str::FromStr>() -> T {
    let mut s = String::new();
    std::io::stdin().read_line(&mut s).ok();
    s.trim().parse().ok().unwrap()
}

fn main() {
    // get input
    let mut input = String::new();

    loop {
        let s: String = read();
        if s == "EOF" {
            break;
        } else {
            input += &s;
            input += "\n";
        }
    }

    let output = wikidot_parser::parse(input);
    println!("{}", output);
}
