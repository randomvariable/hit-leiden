pub fn available() -> bool {
    cfg!(feature = "cuda")
}
