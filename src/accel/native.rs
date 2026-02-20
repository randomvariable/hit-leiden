pub fn available() -> bool {
    cfg!(feature = "native")
}
