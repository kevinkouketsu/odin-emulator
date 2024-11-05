fn main() {
    if cfg!(feature = "sqlite") && cfg!(feature = "postgresql") {
        panic!("'sqlite' and 'postgresql' are mutually exclusive")
    }
}
