use crate::core::backend::AccelerationTarget;

pub fn is_available(target: AccelerationTarget) -> bool {
    match target {
        AccelerationTarget::PureRust => true,
        AccelerationTarget::Native => crate::accel::native::available(),
        AccelerationTarget::Cuda => crate::accel::cuda::available(),
        AccelerationTarget::Rocm => crate::accel::rocm::available(),
    }
}
