use crate::quaternion::Quaternion;
use crate::vector3d::Vector3D;

pub struct Rotation3D {
    q: Quaternion,
}

impl Rotation3D {
    pub fn new(axis: &Vector3D, rotation_amount: f64) -> Self {
        let rotation_q_imaginary = &axis.to_unit_vector() * (rotation_amount / 2.0).sin();
        let q = Quaternion::new(
            (rotation_amount / 2.0).cos(),
            rotation_q_imaginary.x,
            rotation_q_imaginary.y,
            rotation_q_imaginary.z,
        );

        Self { q }
    }

    pub fn rotate_point_about_origin(&self, point: &Vector3D) -> Vector3D {
        let result_quaternion = &(&self.q * &Quaternion::from_vector(point)) * &self.q.conjugate();
        result_quaternion.to_vector()
    }

    pub fn rotate_point_about_positioned_axis(
        &self,
        point: &Vector3D,
        axis_position: &Vector3D,
    ) -> Vector3D {
        // Move the point so it works as though the rotation axis is at the origin
        let rebased_point = point - axis_position;
        let rotated_point = self.rotate_point_about_origin(&rebased_point);
        // Move the point back
        &rotated_point + axis_position
    }

    /// Combine rotation axes into a single rotation axis,
    /// as if rotation_a was applied and then rotation_b
    pub fn combine_rotations(rotation_a: &Rotation3D, rotation_b: &Rotation3D) -> Rotation3D {
        Self {
            q: &rotation_b.q * &rotation_a.q,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_rotations() {
        let rotate_about_x_axis_90 = Rotation3D::new(
            // shouldn't matter what the values are here, this gets converted into a unit vector
            &Vector3D::new(1.1, 0.0, 0.0),
            std::f64::consts::FRAC_PI_2,
        );

        assert!(rotate_about_x_axis_90
            .rotate_point_about_origin(&Vector3D::new(1.5, 0.0, 0.0))
            .approx_equals(&Vector3D::new(1.5, 0.0, 0.0)));
        assert!(rotate_about_x_axis_90
            .rotate_point_about_origin(&Vector3D::new(0.0, 1.5, 0.0))
            .approx_equals(&Vector3D::new(0.0, 0.0, 1.5)));
        assert!(rotate_about_x_axis_90
            .rotate_point_about_origin(&Vector3D::new(0.0, 0.0, 1.5))
            .approx_equals(&Vector3D::new(0.0, -1.5, 0.0)));

        let rotate_about_x_axis_0 = Rotation3D::new(
            // shouldn't matter what the values are here, this gets converted into a unit vector
            &Vector3D::new(1.1, 0.0, 0.0),
            0.0,
        );
        assert!(rotate_about_x_axis_0
            .rotate_point_about_origin(&Vector3D::new(3.4, 2.5, 1.7))
            .approx_equals(&Vector3D::new(3.4, 2.5, 1.7)));
    }

    #[test]
    fn test_combining_rotations() {
        let rotation_a = Rotation3D::new(&Vector3D::new(-2.3, -2.4, 95.1), 1.64);
        let rotation_b = Rotation3D::new(&Vector3D::new(-5.1, 5.4, -10.0), -0.3);
        let combined_rotation = Rotation3D::combine_rotations(&rotation_a, &rotation_b);

        let pt1 = Vector3D::new(0.2, -0.5, 92.0);

        assert!(rotation_b
            .rotate_point_about_origin(&rotation_a.rotate_point_about_origin(&pt1))
            .approx_equals(&combined_rotation.rotate_point_about_origin(&pt1)))
    }
}
