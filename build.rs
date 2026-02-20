fn main() {
    let cuda_enabled = std::env::var("CARGO_FEATURE_CUDA").is_ok();
    let rocm_enabled = std::env::var("CARGO_FEATURE_ROCM").is_ok();

    if cuda_enabled || rocm_enabled {
        let mut build = cc::Build::new();

        if rocm_enabled {
            build.compiler("hipcc");
            build.file("src/accel/hit_leiden_kernel.cu");
            build.compile("hit_leiden_gpu");
        } else if cuda_enabled {
            build.cuda(true);
            build.file("src/accel/hit_leiden_kernel.cu");
            build.compile("hit_leiden_gpu");
        }

        println!("cargo:rerun-if-changed=src/accel/hit_leiden_kernel.cu");
    }
}
