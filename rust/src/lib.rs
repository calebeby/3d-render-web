mod vector3d;
use crate::vector3d::Vector3D;
use wasm_bindgen::prelude::*;

static mut CURSOR_START_POSITION: (i32, i32) = (0, 0);
static mut CURSOR_DOWN: bool = false;
static mut ORBIT_START_CAMERA: Option<Camera> = None;

fn cursor_location_to_3d(width: i32, height: i32, (cursor_x, cursor_y): (i32, i32)) -> Vector3D {
    // Assuming: camera is on the z-axis pointing towards the origin, with y as the up direction
    let min_viewport_dimension = std::cmp::min(width, height);
    let sphere_radius = 0.4 * min_viewport_dimension as f64;

    // x and y relative to the center of the canvas
    let x = cursor_x as f64 - (width as f64 / 2.0);
    let y = cursor_y as f64 - (height as f64 / 2.0);
    // Sphere: z^2 + x^2 + y^2 = radius^2
    let z_eq_sq = sphere_radius * sphere_radius - x * x - y * y;
    let z = if z_eq_sq >= 0.0 { z_eq_sq.sqrt() } else { 0.0 };
    Vector3D { x, y, z }
}

fn compute_camera_position_from_orbit(
    width: i32,
    height: i32,
    initial_camera: Camera,
    orbit_start_cursor: (i32, i32),
    orbit_end_cursor: (i32, i32),
) -> Camera {
    let start_cursor_vector = cursor_location_to_3d(width, height, orbit_start_cursor);
    let end_cursor_vector = cursor_location_to_3d(width, height, orbit_end_cursor);

    if start_cursor_vector == end_cursor_vector {
        return initial_camera;
    }

    let rotation_axis = start_cursor_vector
        .cross(&end_cursor_vector)
        .to_unit_vector();
    let rotation_d_theta = start_cursor_vector.dot(&end_cursor_vector)
        / (start_cursor_vector.magnitude() * end_cursor_vector.magnitude());

    let rotation_vector = &rotation_axis * rotation_d_theta;

    Camera {
        u_up: initial_camera.u_up.rotate_about_origin(rotation_vector),
        u_right: initial_camera.u_right.rotate_about_origin(rotation_vector),
        plane: Plane {
            point: initial_camera
                .plane
                .point
                .rotate_about_origin(rotation_vector),
            normal: initial_camera
                .plane
                .normal
                .rotate_about_origin(rotation_vector),
        },
        point: initial_camera.point.rotate_about_origin(rotation_vector),
    }
}

#[wasm_bindgen]
pub fn get_points(
    canvas_ctx: &web_sys::CanvasRenderingContext2d,
    width: i32,
    height: i32,
    cursor_x: i32,
    cursor_y: i32,
    cursor_down: bool,
) -> Vec<f64> {
    let mut camera: Camera = (unsafe { ORBIT_START_CAMERA }).unwrap_or(Camera::new_towards(
        Vector3D::new(4.0, 4.0, 4.0),
        Vector3D::zero(),
    ));

    unsafe {
        if !CURSOR_DOWN && cursor_down {
            // Just pressed cursor
            CURSOR_DOWN = true;
            CURSOR_START_POSITION = (cursor_x, cursor_y);
            ORBIT_START_CAMERA = Some(compute_camera_position_from_orbit(
                width,
                height,
                camera,
                CURSOR_START_POSITION,
                (cursor_x, cursor_y),
            ));
        }
        if CURSOR_DOWN && !cursor_down {
            // Just released cursor
            CURSOR_DOWN = false;
            ORBIT_START_CAMERA = Some(compute_camera_position_from_orbit(
                width,
                height,
                camera,
                CURSOR_START_POSITION,
                (cursor_x, cursor_y),
            ));
        }
        if CURSOR_DOWN {
            camera = compute_camera_position_from_orbit(
                width,
                height,
                camera,
                CURSOR_START_POSITION,
                (cursor_x, cursor_y),
            )
        }
    }
    let mut out = Vec::<f64>::with_capacity(100);
    let front_left_top = Vector3D::new(-1.0, -1.0, 1.0);
    let front_right_top = Vector3D::new(-1.0, 1.0, 1.0);
    let front_left_bottom = Vector3D::new(-1.0, -1.0, -1.0);
    let front_right_bottom = Vector3D::new(-1.0, 1.0, -1.0);
    let back_left_top = Vector3D::new(1.0, -1.0, 1.0);
    let back_right_top = Vector3D::new(1.0, 1.0, 1.0);
    let back_left_bottom = Vector3D::new(1.0, -1.0, -1.0);
    let back_right_bottom = Vector3D::new(1.0, 1.0, -1.0);

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

#[derive(Debug, Copy, Clone)]
struct Camera {
    plane: Plane,
    u_right: Vector3D,
    u_up: Vector3D,
    point: Vector3D,
}

impl Camera {
    fn new_towards(camera_point: Vector3D, target: Vector3D) -> Camera {
        let camera_to_target = target - &camera_point;
        let u_camera_to_target = camera_to_target.to_unit_vector();
        let u_z = Vector3D::new(0.0, 0.0, 1.0);
        let u_right = u_camera_to_target.cross(&u_z);
        let u_up = u_right.cross(&u_camera_to_target);
        Camera {
            plane: Plane {
                normal: u_camera_to_target,
                point: &camera_point + &u_camera_to_target,
            },
            u_right,
            u_up,
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
        let scale = 500.0;
        let point_x_in_camera = scale * point_in_camera_plane.dot(&self.u_right);
        let point_y_in_camera = scale * point_in_camera_plane.dot(&-&self.u_up);

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

#[derive(Debug)]
pub struct Ray {
    point: Vector3D,
    direction: Vector3D,
}

#[derive(Debug, Copy, Clone)]
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
