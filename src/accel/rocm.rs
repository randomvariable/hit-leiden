pub use crate::accel::cuda::run_on_gpu;

pub fn available() -> bool {
    cfg!(feature = "rocm")
}
