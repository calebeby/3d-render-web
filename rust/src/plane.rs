use crate::ray::Ray;
use crate::vector3d::Vector3D;

#[derive(Debug, Copy, Clone)]
pub struct Plane {
    pub point: Vector3D,
    pub normal: Vector3D,
}

impl Plane {
    pub fn intersection(&self, ray: &Ray) -> Vector3D {
        let diff = &ray.point - &self.point;
        let prod1 = diff.dot(&self.normal);
        let prod2 = ray.direction.dot(&self.normal);
        let prod3 = prod1 / prod2;
        &ray.point - &(&ray.direction * prod3)
    }
    pub fn offset(&self, offset: f64) -> Plane {
        let offset_vector = offset * self.normal.to_unit_vector();
        Plane {
            point: &self.point + &offset_vector,
            normal: self.normal,
        }
    }
}
