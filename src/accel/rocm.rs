pub fn available() -> bool {
    cfg!(feature = "rocm")
}
