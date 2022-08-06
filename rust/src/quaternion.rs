use crate::vector3d::Vector3D;

#[derive(Clone)]
pub struct Quaternion {
    real: f64,
    i: f64,
    j: f64,
    k: f64,
}

impl Quaternion {
    #[inline]
    pub fn new(real: f64, i: f64, j: f64, k: f64) -> Quaternion {
        Quaternion { real, i, j, k }
    }
    #[inline]
    pub fn from_vector(vector: &Vector3D) -> Quaternion {
        Quaternion {
            real: 0.0,
            i: vector.x,
            j: vector.y,
            k: vector.z,
        }
    }
    #[inline]
    pub fn to_vector(&self) -> Vector3D {
        Vector3D {
            x: self.i,
            y: self.j,
            z: self.k,
        }
    }
    #[inline]
    pub fn conjugate(&self) -> Quaternion {
        Quaternion {
            real: self.real,
            i: -self.i,
            j: -self.j,
            k: -self.k,
        }
    }
}

impl std::ops::Mul<&Quaternion> for &Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: &Quaternion) -> Quaternion {
        Quaternion {
            real: self.real * rhs.real - self.i * rhs.i - self.j * rhs.j - self.k * rhs.k,
            i: self.real * rhs.i + self.i * rhs.real + self.j * rhs.k - self.k * rhs.j,
            j: self.real * rhs.j - self.i * rhs.k + self.j * rhs.real + self.k * rhs.i,
            k: self.real * rhs.k + self.i * rhs.j - self.j * rhs.i + self.k * rhs.real,
        }
    }
}
