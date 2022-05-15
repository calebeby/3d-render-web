mod polyhedron;
mod quaternion;
mod twisty_puzzle;
mod vector3d;
use crate::twisty_puzzle::{Cut, TwistyPuzzle};
use crate::vector3d::Vector3D;
use polyhedron::{Face, Polyhedron};
use wasm_bindgen::prelude::*;
use web_sys::console;

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
pub fn render(
    canvas_ctx: &web_sys::CanvasRenderingContext2d,
    width: i32,
    height: i32,
    cursor_x: i32,
    cursor_y: i32,
    cursor_down: bool,
) {
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

    let make_tetrahedron = || Polyhedron::generate(3, 3);
    let make_cube = || Polyhedron::generate(4, 3);
    let make_octahedron = || Polyhedron::generate(3, 4);
    let make_dodecahedron = || Polyhedron::generate(5, 3);
    let make_icosahedron = || Polyhedron::generate(3, 5);

    let base_shape = make_dodecahedron();
    let cube_puzzle = TwistyPuzzle::new(
        &base_shape,
        // &[]
        &base_shape
            .faces
            .iter()
            .map(|face| Cut::new("R", face.plane().offset(-0.29)))
            .collect::<Vec<_>>(),
        // &[Cut::new("R", base_shape.faces[0].plane().offset(-0.2))],
    );

    let orange = Color::new(254, 133, 57);
    let white = Color::new(231, 224, 220);
    let blue = Color::new(45, 81, 157);
    let red = Color::new(221, 30, 18);
    let dark_red = Color::new(143, 33, 25);
    let green = Color::new(35, 168, 74);
    let yellow = Color::new(219, 226, 35);
    let purple = Color::new(197, 107, 197);

    let colors = [white, blue, orange, green, red, yellow, purple, dark_red];
    let uncolored_faces = cube_puzzle.faces().iter();
    let faces: Vec<FaceWithColor> = uncolored_faces
        .enumerate()
        .map(|(i, f)| FaceWithColor {
            face: f,
            color: colors[i % colors.len()],
        })
        .collect();

    let mut seen_faces = faces
        .iter()
        .filter_map(|face| camera.see_face(face))
        .collect::<Vec<_>>();
    seen_faces.sort_by(|a, b| {
        b.distance_from_camera
            .partial_cmp(&a.distance_from_camera)
            .unwrap()
    });

    canvas_ctx.set_fill_style(&"black".into());
    canvas_ctx.fill_rect(0.0, 0.0, width.into(), height.into());

    for polygon in seen_faces {
        canvas_ctx.set_fill_style(&polygon.color.to_hex_str().into());
        canvas_ctx.begin_path();
        for point in polygon.points {
            canvas_ctx.line_to(point.0 + width as f64 / 2.0, point.1 + height as f64 / 2.0);
        }
        canvas_ctx.close_path();
        canvas_ctx.fill();
    }
}

#[derive(Debug, Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b }
    }
    fn to_hex_str(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
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
        let scale = 1200.0;
        let point_x_in_camera = scale * point_in_camera_plane.dot(&self.u_right);
        let point_y_in_camera = scale * point_in_camera_plane.dot(&-&self.u_up);

        Some((point_x_in_camera, point_y_in_camera))
    }
    fn see_face(&self, face: &FaceWithColor) -> Option<SeenFace> {
        let mut points = Vec::<(f64, f64)>::new();
        let mut sum_dist = 0.0;
        for &vertex in &face.face.vertices {
            sum_dist += (vertex - &self.point).magnitude();
            match self.see_point(vertex) {
                Some(point) => points.push(point),
                None => return None,
            }
        }
        Some(SeenFace {
            color: face.color,
            points,
            distance_from_camera: sum_dist / face.face.vertices.len() as f64,
        })
    }
}

struct FaceWithColor<'a> {
    face: &'a Face,
    color: Color,
}

#[derive(Debug)]
pub struct Ray {
    point: Vector3D,
    direction: Vector3D,
}

#[derive(Debug, Copy, Clone)]
pub struct Plane {
    point: Vector3D,
    normal: Vector3D,
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
