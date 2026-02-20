#[cfg(any(feature = "cuda", feature = "rocm"))]
extern "C" {
    fn launch_hit_leiden_gpu(
        active_nodes: *const i32,
        node_to_community: *mut i32,
        num_active: i32,
    );
}

pub fn run_on_gpu(active_nodes: &[i32], node_to_community: &mut [i32]) {
    #[cfg(any(feature = "cuda", feature = "rocm"))]
    unsafe {
        launch_hit_leiden_gpu(
            active_nodes.as_ptr(),
            node_to_community.as_mut_ptr(),
            active_nodes.len() as i32,
        );
    }
    #[cfg(not(any(feature = "cuda", feature = "rocm")))]
    {
        let _ = active_nodes;
        let _ = node_to_community;
        panic!("GPU features not enabled");
    }
}

pub fn available() -> bool {
    cfg!(feature = "cuda")
}
