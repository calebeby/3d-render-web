use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
pub fn get_points(
    _width: f64,
    _height: f64,
    camera_x_offset: f64,
    camera_y_offset: f64,
    camera_z_offset: f64,
) -> Vec<f64> {
    let mut out = Vec::<f64>::with_capacity(100);
    let front_left_top = Vector3D::new(-1.0, -1.0, 1.0);
    let front_right_top = Vector3D::new(-1.0, 1.0, 1.0);
    let front_left_bottom = Vector3D::new(-1.0, -1.0, -1.0);
    let front_right_bottom = Vector3D::new(-1.0, 1.0, -1.0);
    let back_left_top = Vector3D::new(1.0, -1.0, 1.0);
    let back_right_top = Vector3D::new(1.0, 1.0, 1.0);
    let back_left_bottom = Vector3D::new(1.0, -1.0, -1.0);
    let back_right_bottom = Vector3D::new(1.0, 1.0, -1.0);
    let camera = Camera::new_towards(
        Vector3D::new(camera_x_offset, camera_y_offset, camera_z_offset),
        front_left_top,
    );

    let red = Color::new(255, 0, 0);
    let green = Color::new(0, 255, 0);
    let blue = Color::new(0, 0, 255);
    let yellow = Color::new(255, 255, 0);
    let purple = Color::new(255, 0, 255);
    let cyan = Color::new(0, 255, 255);

    let front_face = Face {
        vertices: vec![
            &front_left_top,
            &front_right_top,
            &front_right_bottom,
            &front_left_bottom,
        ],
        color: red,
    };

    let right_face = Face {
        vertices: vec![
            &front_right_top,
            &back_right_top,
            &back_right_bottom,
            &front_right_bottom,
        ],
        color: blue,
    };

    let top_face = Face {
        vertices: vec![
            &front_right_top,
            &front_left_top,
            &back_left_top,
            &back_right_top,
        ],
        color: green,
    };

    let left_face = Face {
        vertices: vec![
            &front_left_top,
            &front_left_bottom,
            &back_left_bottom,
            &back_left_top,
        ],
        color: yellow,
    };

    let back_face = Face {
        vertices: vec![
            &back_left_top,
            &back_right_top,
            &back_right_bottom,
            &back_left_bottom,
        ],
        color: purple,
    };

    let bottom_face = Face {
        vertices: vec![
            &front_left_bottom,
            &front_right_bottom,
            &back_right_bottom,
            &back_left_bottom,
        ],
        color: cyan,
    };

    let faces = vec![
        &front_face,
        &right_face,
        &top_face,
        &left_face,
        &back_face,
        &bottom_face,
    ];
    let mut seen_faces = faces
        .iter()
        .filter_map(|face| camera.see_face(face))
        .collect::<Vec<_>>();
    seen_faces.sort_by(|a, b| {
        b.distance_from_camera
            .partial_cmp(&a.distance_from_camera)
            .unwrap()
    });

    for face in seen_faces {
        serialize_polygon(face.points, &face.color, &mut out);
    }

    out
}

fn serialize_polygon(polygon: Vec<(f64, f64)>, color: &Color, out: &mut Vec<f64>) {
    // 2D Polygons are returned, in the format:
    // ...(numPoints, color, ...(pointX, pointY))
    out.push(polygon.len() as f64);
    out.push(color.to_int().into());
    for pt in polygon {
        out.push(pt.0);
        out.push(pt.1);
    }
}

#[derive(Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }
    fn to_int(&self) -> u32 {
        ((self.r as u32) << 16) + ((self.g as u32) << 8) + (self.b as u32)
    }
}

struct SeenFace {
    points: Vec<(f64, f64)>,
    color: Color,
    distance_from_camera: f64,
}

#[derive(Debug)]
struct Camera {
    plane: Plane,
    u_right: Vector3D,
    u_up: Vector3D,
    point: Vector3D,
}

impl Camera {
    fn new_towards(camera_point: Vector3D, target: Vector3D) -> Camera {
        let camera_to_target = &target - &camera_point;
        let u_camera_to_target = camera_to_target.to_unit_vector();
        let u_z = Vector3D::new(0.0, 0.0, 1.0);
        let u_right = u_camera_to_target.cross(&u_z);
        let u_up = u_right.cross(&u_camera_to_target);
        Camera {
            plane: Plane {
                normal: u_camera_to_target,
                point: &camera_point + &u_camera_to_target,
            },
            u_right: &u_right * 600.0,
            u_up: &u_up * 600.0,
            point: camera_point,
        }
    }
    // This returns an option in case the point is behind the camera
    fn see_point(&self, point: Vector3D) -> Option<(f64, f64)> {
        let ray_to_camera = point.ray_to(&self.point);
        // Point is behind camera; ignore
        if ray_to_camera.direction.dot(&self.plane.normal) >= 0.0 {
            return None;
        }
        let camera_plane_intersection = self.plane.intersection(&ray_to_camera);
        let point_in_camera_plane = &camera_plane_intersection - &self.point;
        let point_x_in_camera = point_in_camera_plane.dot(&self.u_right);
        let point_y_in_camera = point_in_camera_plane.dot(&-&self.u_up);

        Some((point_x_in_camera, point_y_in_camera))
    }
    fn see_face(&self, face: &Face) -> Option<SeenFace> {
        let mut points = Vec::<(f64, f64)>::new();
        let mut sum_dist = 0.0;
        for &vertex in &face.vertices {
            sum_dist += (vertex - &self.point).magnitude();
            match self.see_point(*vertex) {
                Some(point) => points.push(point),
                None => return None,
            }
        }
        Some(SeenFace {
            color: face.color,
            points,
            distance_from_camera: sum_dist / face.vertices.len() as f64,
        })
    }
}

struct Face<'a> {
    vertices: Vec<&'a Vector3D>,
    color: Color,
}

#[derive(Copy, Clone)]
struct Vector3D {
    x: f64,
    y: f64,
    z: f64,
}

impl Vector3D {
    fn new(x: f64, y: f64, z: f64) -> Vector3D {
        Vector3D { x, y, z }
    }
    fn dot(&self, other: &Vector3D) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    fn cross(&self, other: &Vector3D) -> Vector3D {
        Vector3D {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
    fn ray_to(&self, other: &Vector3D) -> Ray {
        Ray {
            point: *self,
            direction: other - self,
        }
    }
    fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    fn to_unit_vector(&self) -> Vector3D {
        self / self.magnitude()
    }
    /// The magnitude of the rotation vector is the angle of rotation (radians)
    fn rotate_about_origin(&self, rotation_vector: Vector3D) -> Vector3D {
        let rotation_amount = rotation_vector.magnitude();
        if rotation_amount == 0.0 {
            return *self;
        }
        let rotation_q_imaginary =
            &rotation_vector.to_unit_vector() * (rotation_amount / 2.0).sin();
        let q = Quaternion::new(
            (rotation_amount / 2.0).cos(),
            rotation_q_imaginary.x,
            rotation_q_imaginary.y,
            rotation_q_imaginary.z,
        );
        let result_quaternion = &(&q * &Quaternion::from_vector(self)) * &q.conjugate();
        result_quaternion.to_vector()
    }
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

#[derive(Debug)]
struct Ray {
    point: Vector3D,
    direction: Vector3D,
}

#[derive(Debug)]
struct Plane {
    point: Vector3D,
    normal: Vector3D,
}

impl Plane {
    fn intersection(&self, ray: &Ray) -> Vector3D {
        let diff = &ray.point - &self.point;
        let prod1 = diff.dot(&self.normal);
        let prod2 = ray.direction.dot(&self.normal);
        let prod3 = prod1 / prod2;
        &ray.point - &(&ray.direction * prod3)
    }
}

#[derive(Debug)]
struct Quaternion {
    real: f64,
    i: f64,
    j: f64,
    k: f64,
}

impl Quaternion {
    fn new(real: f64, i: f64, j: f64, k: f64) -> Quaternion {
        Quaternion { real, i, j, k }
    }
    fn from_vector(vector: &Vector3D) -> Quaternion {
        Quaternion {
            real: 0.0,
            i: vector.x,
            j: vector.y,
            k: vector.z,
        }
    }
    fn to_vector(&self) -> Vector3D {
        Vector3D {
            x: self.i,
            y: self.j,
            z: self.k,
        }
    }
    fn conjugate(&self) -> Quaternion {
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
