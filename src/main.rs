fn main() {
    let code = mitos_login::run(std::env::args().collect());
    std::process::exit(code);
}
