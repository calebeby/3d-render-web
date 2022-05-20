use crate::vector3d::Vector3D;

#[derive(Debug)]
pub struct Ray {
    pub point: Vector3D,
    pub direction: Vector3D,
}
