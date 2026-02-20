use crate::core::backend::AccelerationTarget;

pub fn all_targets() -> [AccelerationTarget; 4] {
    [
        AccelerationTarget::PureRust,
        AccelerationTarget::Native,
        AccelerationTarget::Cuda,
        AccelerationTarget::Rocm,
    ]
}
