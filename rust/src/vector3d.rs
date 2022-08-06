use crate::ray::Ray;

#[derive(Copy, Clone)]
pub struct Vector3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3D {
    pub fn new(x: f64, y: f64, z: f64) -> Vector3D {
        Vector3D { x, y, z }
    }
    pub fn zero() -> Vector3D {
        Vector3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
    pub fn unit() -> Vector3D {
        Vector3D {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        }
    }
    pub fn dot(&self, other: &Vector3D) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    pub fn cross(&self, other: &Vector3D) -> Vector3D {
        Vector3D {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
    pub fn ray_to(&self, other: &Vector3D) -> Ray {
        Ray {
            point: *self,
            direction: other - self,
        }
    }
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    pub fn to_unit_vector(&self) -> Vector3D {
        self / self.magnitude()
    }
    pub fn angle_between(a: &Vector3D, b: &Vector3D) -> f64 {
        (a.dot(b) / (a.magnitude() * b.magnitude())).acos()
    }

    pub fn approx_equals(&self, other: &Vector3D) -> bool {
        approx_equals(self.x, other.x)
            && approx_equals(self.y, other.y)
            && approx_equals(self.z, other.z)
    }

    pub fn from_average(other_vectors: &[Vector3D]) -> Vector3D {
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_z = 0.0;

        for vec in other_vectors {
            sum_x += vec.x;
            sum_y += vec.y;
            sum_z += vec.z;
        }

        let count = other_vectors.len() as f64;
        Vector3D::new(sum_x / count, sum_y / count, sum_z / count)
    }
}

fn approx_equals(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-10
}

impl std::fmt::Debug for Vector3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vector3D ({}, {}, {})", self.x, self.y, self.z)
    }
}

impl std::ops::Sub<&Vector3D> for &Vector3D {
    type Output = Vector3D;

    fn sub(self, rhs: &Vector3D) -> Vector3D {
        Vector3D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::Sub<Vector3D> for &Vector3D {
    type Output = Vector3D;

    fn sub(self, rhs: Vector3D) -> Vector3D {
        Vector3D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::Sub<&Vector3D> for Vector3D {
    type Output = Vector3D;

    fn sub(self, rhs: &Vector3D) -> Vector3D {
        Vector3D {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::Add<&Vector3D> for &Vector3D {
    type Output = Vector3D;

    fn add(self, rhs: &Vector3D) -> Vector3D {
        Vector3D {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::Mul<f64> for &Vector3D {
    type Output = Vector3D;

    fn mul(self, rhs: f64) -> Vector3D {
        Vector3D {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl std::ops::Mul<Vector3D> for f64 {
    type Output = Vector3D;

    fn mul(self, rhs: Vector3D) -> Vector3D {
        Vector3D {
            x: rhs.x * self,
            y: rhs.y * self,
            z: rhs.z * self,
        }
    }
}

impl std::ops::Neg for &Vector3D {
    type Output = Vector3D;

    fn neg(self) -> Vector3D {
        Vector3D {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl std::ops::Div<f64> for &Vector3D {
    type Output = Vector3D;

    fn div(self, rhs: f64) -> Vector3D {
        Vector3D {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl std::cmp::PartialEq<Vector3D> for Vector3D {
    fn eq(&self, rhs: &Vector3D) -> bool {
        self.x == rhs.x && self.y == rhs.y && self.z == rhs.z
    }
}
