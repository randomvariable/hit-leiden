#if defined(__HIPCC__)
    #include <hip/hip_runtime.h>
#elif defined(__CUDACC__)
    #include <cuda_runtime.h>
    #define hipBlockDim_x blockDim.x
    #define hipBlockIdx_x blockIdx.x
    #define hipThreadIdx_x threadIdx.x
    #define hipLaunchKernelGGL(F, G, B, M, S, ...) F<<<G, B, M, S>>>(__VA_ARGS__)
#else
    // Fallback for IDEs or non-GPU compilers
    #define __global__
    #define hipBlockDim_x 1
    #define hipBlockIdx_x 1
    #define hipThreadIdx_x 1
#endif

extern "C" __global__ void hit_leiden_shard_kernel(
    const int* active_nodes,
    int* node_to_community,
    int num_active
) {
    int idx = hipBlockDim_x * hipBlockIdx_x + hipThreadIdx_x;
    if (idx < num_active) {
        int node = active_nodes[idx];
        // Placeholder for actual graph logic
        // In a real implementation, this would:
        // 1. Read the node's neighbors from the CSR graph arrays
        // 2. Calculate modularity gain for each neighboring community
        // 3. Use atomicAdd to update community degrees
        // 4. Update node_to_community[node]
        
        // For demonstration, we just do a dummy read/write to ensure the FFI works
        node_to_community[node] = node_to_community[node];
    }
}

extern "C" void launch_hit_leiden_gpu(
    const int* active_nodes,
    int* node_to_community,
    int num_active
) {
    int threads = 256;
    int blocks = (num_active + threads - 1) / threads;
    
    #if defined(__HIPCC__)
        hipLaunchKernelGGL(hit_leiden_shard_kernel, dim3(blocks), dim3(threads), 0, 0, active_nodes, node_to_community, num_active);
    #elif defined(__CUDACC__)
        hit_leiden_shard_kernel<<<blocks, threads>>>(active_nodes, node_to_community, num_active);
    #endif
}
