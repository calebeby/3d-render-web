mod polyhedron;
mod quaternion;
mod solver;
mod twisty_puzzle;
mod vector3d;

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::f64::consts::TAU;
use std::rc::Rc;

use crate::twisty_puzzle::{CutDefinition, TwistyPuzzle};
use crate::vector3d::Vector3D;
use polyhedron::{Face, Polyhedron};
use solver::{LookaheadSolver, Solver};
use twisty_puzzle::{PieceFace, PuzzleState};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
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

struct State {
    solver: LookaheadSolver,
    is_solving: bool,
    puzzle_state: PuzzleState,
    puzzle: TwistyPuzzle,
    turn_queue: VecDeque<String>,
    turn_progress: f64,
}

#[wasm_bindgen]
pub fn start() {
    let result = init();
    match result {
        Err(err) => console::error_1(&err),
        _ => {}
    }
}

fn init() -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let app_el = document.query_selector("#app").unwrap().unwrap();
    let buttons_div = document
        .create_element("div")?
        .dyn_into::<web_sys::HtmlDivElement>()?;
    buttons_div.set_class_name("buttons");
    app_el.append_child(&buttons_div)?;

    let canvas = document
        .create_element("canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    app_el.append_child(&canvas)?;

    let canvas_ctx = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    let canvas = Rc::new(canvas);
    let canvas_ctx = Rc::new(canvas_ctx);

    let tetrahedron = Polyhedron::generate(3, 3);
    let cube = Polyhedron::generate(4, 3);
    let octahedron = Polyhedron::generate(3, 4);
    let dodecahedron = Polyhedron::generate(5, 3);
    let icosahedron = Polyhedron::generate(3, 5);

    let megaminx = || {
        TwistyPuzzle::new(
            &dodecahedron,
            &dodecahedron
                .faces
                .iter()
                .map(|face| CutDefinition::new_infer_name(face.plane().offset(-0.33), TAU / 5.0))
                .collect::<Vec<_>>(),
        )
    };

    let starminx = || {
        TwistyPuzzle::new(
            &dodecahedron,
            &dodecahedron
                .faces
                .iter()
                .map(|face| CutDefinition::new_infer_name(face.plane().offset(-0.75), TAU / 5.0))
                .collect::<Vec<_>>(),
        )
    };

    let eitans_star = || {
        TwistyPuzzle::new(
            &icosahedron,
            &icosahedron
                .faces
                .iter()
                .map(|face| CutDefinition::new_infer_name(face.plane().offset(-0.29), TAU / 3.0))
                .collect::<Vec<_>>(),
        )
    };

    let rubiks_cube_cut_names = ["U", "F", "R", "B", "L", "D"];

    let rubiks_cube_3x3 = || {
        TwistyPuzzle::new(
            &cube,
            &cube
                .faces
                .iter()
                .enumerate()
                .map(|(i, face)| {
                    CutDefinition::new(
                        rubiks_cube_cut_names[i],
                        face.plane().offset(-0.33),
                        TAU / 4.0,
                    )
                })
                .collect::<Vec<_>>(),
        )
    };

    let rubiks_cube_2x2 = || {
        TwistyPuzzle::new(
            &cube,
            &cube.faces[0..=2]
                .iter()
                .enumerate()
                .map(|(i, face)| {
                    CutDefinition::new(
                        rubiks_cube_cut_names[i],
                        face.plane().offset(-0.5),
                        TAU / 4.0,
                    )
                })
                .collect::<Vec<_>>(),
        )
    };

    let compy_cube = || {
        TwistyPuzzle::new(
            &cube,
            &cube
                .vertices
                .iter()
                .map(|vertex| {
                    let plane = Plane {
                        point: *vertex,
                        normal: *vertex,
                    };
                    CutDefinition::new_infer_name(plane.offset(-0.45), TAU / 3.0)
                })
                .collect::<Vec<_>>(),
        )
    };

    let pentultimate = || {
        TwistyPuzzle::new(
            &dodecahedron,
            &dodecahedron
                .vertices
                .iter()
                .map(|vertex| {
                    let plane = Plane {
                        point: *vertex,
                        normal: *vertex,
                    };
                    CutDefinition::new_infer_name(plane.offset(-0.1), TAU / 3.0)
                })
                .collect::<Vec<_>>(),
        )
    };

    let dino_starminx = || {
        TwistyPuzzle::new(
            &dodecahedron,
            &dodecahedron
                .vertices
                .iter()
                .map(|vertex| {
                    let plane = Plane {
                        point: *vertex,
                        normal: *vertex,
                    };
                    CutDefinition::new_infer_name(plane.offset(-0.3), TAU / 3.0)
                })
                .collect::<Vec<_>>(),
        )
    };

    let pyraminx_thing = || {
        TwistyPuzzle::new(
            &tetrahedron,
            &tetrahedron
                .vertices
                .iter()
                .map(|vertex| {
                    let plane = Plane {
                        point: *vertex,
                        normal: *vertex,
                    };
                    CutDefinition::new_infer_name(plane.offset(-0.5), TAU / 3.0)
                })
                .collect::<Vec<_>>(),
        )
    };

    let skewb_diamond = || {
        TwistyPuzzle::new(
            &octahedron,
            &octahedron.faces[0..=3]
                .iter()
                .map(|face| CutDefinition::new_infer_name(face.plane().offset(-0.41), TAU / 3.0))
                .collect::<Vec<_>>(),
        )
    };

    let puzzle = pyraminx_thing();

    let puzzle_state = puzzle.get_initial_state();

    let state = Rc::new(RefCell::new(State {
        solver: LookaheadSolver::new(&puzzle, 7),
        is_solving: false,
        puzzle,
        puzzle_state,
        turn_queue: VecDeque::new(),
        turn_progress: 0.0,
    }));

    let width = Rc::new(Cell::new(canvas.client_width()));
    let height = Rc::new(Cell::new(canvas.client_height()));

    let mut cuts_list: Vec<_> = state.borrow().puzzle.turns_iter().cloned().collect();
    cuts_list.sort();
    for cut_name in cuts_list {
        let button = document
            .create_element("button")?
            .dyn_into::<web_sys::HtmlButtonElement>()?;
        button.set_inner_text(&cut_name);
        buttons_div.append_child(&button)?;
        let cut_name = Rc::new(cut_name.clone());

        {
            let cut_name = cut_name.clone();
            let canvas_ctx = canvas_ctx.clone();
            let state = state.clone();
            let width = width.clone();
            let height = height.clone();
            let handle_click = move || {
                state
                    .borrow_mut()
                    .turn_queue
                    .push_back(cut_name.to_string());
                render(
                    &state.borrow(),
                    &canvas_ctx,
                    width.get(),
                    height.get(),
                    0,
                    0,
                    false,
                );
            };

            let click_listener = Closure::wrap(Box::new(handle_click) as Box<dyn FnMut()>);
            button.add_event_listener_with_callback(
                "click",
                click_listener.as_ref().unchecked_ref(),
            )?;
            click_listener.forget();
        }
    }

    fn solve_next_step(state: &mut State) -> bool {
        let turn_name = state
            .solver
            .get_next_move(&state.puzzle, &state.puzzle_state);
        if let Some(turn_name) = turn_name {
            state.turn_queue.push_back(turn_name);
            true
        } else {
            state.is_solving = false;
            false
        }
    }

    {
        let solve_button = document
            .create_element("button")?
            .dyn_into::<web_sys::HtmlButtonElement>()?;
        solve_button.set_inner_text("Solve Step");
        buttons_div.append_child(&solve_button)?;

        let canvas_ctx = canvas_ctx.clone();
        let state = state.clone();
        let width = width.clone();
        let height = height.clone();
        let handle_click = move || {
            let mut state = state.borrow_mut();
            if solve_next_step(&mut state) {
                render(&state, &canvas_ctx, width.get(), height.get(), 0, 0, false);
            }
        };

        let click_listener = Closure::wrap(Box::new(handle_click) as Box<dyn FnMut()>);
        solve_button
            .add_event_listener_with_callback("click", click_listener.as_ref().unchecked_ref())?;
        click_listener.forget();
    }

    {
        let solve_button = document
            .create_element("button")?
            .dyn_into::<web_sys::HtmlButtonElement>()?;
        solve_button.set_inner_text("Solve");
        buttons_div.append_child(&solve_button)?;

        let canvas_ctx = canvas_ctx.clone();
        let state = state.clone();
        let width = width.clone();
        let height = height.clone();
        let handle_click = move || {
            let mut state = state.borrow_mut();
            state.is_solving = true;
            solve_next_step(&mut state);
            // let turns: VecDeque<_> = state
            //     .solver
            //     .next_move_iter(&state.puzzle, &state.puzzle_state)
            //     .take(200)
            //     .collect();
            // state.turn_queue = turns;
            // render(&state, &canvas_ctx, width.get(), height.get(), 0, 0, false);
        };

        let click_listener = Closure::wrap(Box::new(handle_click) as Box<dyn FnMut()>);
        solve_button
            .add_event_listener_with_callback("click", click_listener.as_ref().unchecked_ref())?;
        click_listener.forget();
    }

    {
        let scramble_button = document
            .create_element("button")?
            .dyn_into::<web_sys::HtmlButtonElement>()?;
        scramble_button.set_inner_text("Scramble");
        buttons_div.append_child(&scramble_button)?;

        let canvas_ctx = canvas_ctx.clone();
        let state = state.clone();
        let width = width.clone();
        let height = height.clone();
        let handle_click = move || {
            let mut state = state.borrow_mut();
            state.turn_queue = VecDeque::new();
            state.puzzle_state = state.puzzle.scramble(&state.puzzle_state, 200);
            render(&state, &canvas_ctx, width.get(), height.get(), 0, 0, false);
        };

        let click_listener = Closure::wrap(Box::new(handle_click) as Box<dyn FnMut()>);
        scramble_button
            .add_event_listener_with_callback("click", click_listener.as_ref().unchecked_ref())?;
        click_listener.forget();
    }

    {
        let reset_button = document
            .create_element("button")?
            .dyn_into::<web_sys::HtmlButtonElement>()?;
        reset_button.set_inner_text("Reset");
        buttons_div.append_child(&reset_button)?;

        let canvas_ctx = canvas_ctx.clone();
        let state = state.clone();
        let width = width.clone();
        let height = height.clone();
        let handle_click = move || {
            let mut state = state.borrow_mut();
            state.turn_queue = VecDeque::new();
            state.puzzle_state = state.puzzle.get_initial_state();
            render(&state, &canvas_ctx, width.get(), height.get(), 0, 0, false);
        };

        let click_listener = Closure::wrap(Box::new(handle_click) as Box<dyn FnMut()>);
        reset_button
            .add_event_listener_with_callback("click", click_listener.as_ref().unchecked_ref())?;
        click_listener.forget();
    }

    {
        let canvas = canvas.clone();
        let canvas_ctx = canvas_ctx.clone();
        let state = state.clone();
        let width = width.clone();
        let height = height.clone();

        let update_width = move || {
            width.set(canvas.client_width());
            height.set(canvas.client_height());
            canvas.set_width(width.get() as _);
            canvas.set_height(height.get() as _);

            render(
                &state.borrow(),
                &canvas_ctx,
                width.get(),
                height.get(),
                0,
                0,
                false,
            );
        };

        update_width();

        let resize_listener = Closure::wrap(Box::new(update_width) as Box<dyn FnMut()>);
        window
            .add_event_listener_with_callback("resize", resize_listener.as_ref().unchecked_ref())?;
        resize_listener.forget();
    }

    {
        let canvas_ctx = canvas_ctx.clone();
        let state = state.clone();
        let width = width.clone();
        let height = height.clone();

        let handle_mouse_event = move |event: web_sys::MouseEvent| {
            let x = event.offset_x();
            let y = event.offset_y();
            render(
                &state.borrow(),
                &canvas_ctx,
                width.get(),
                height.get(),
                x,
                y,
                event.buttons() == 1,
            );
        };

        let mouse_listener = Closure::wrap(Box::new(handle_mouse_event) as Box<dyn FnMut(_)>);
        window.add_event_listener_with_callback(
            "mousedown",
            mouse_listener.as_ref().unchecked_ref(),
        )?;
        window
            .add_event_listener_with_callback("mouseup", mouse_listener.as_ref().unchecked_ref())?;
        window.add_event_listener_with_callback(
            "mousemove",
            mouse_listener.as_ref().unchecked_ref(),
        )?;
        mouse_listener.forget();
    }

    {
        let canvas_ctx = canvas_ctx.clone();
        let state = state.clone();
        let width = width.clone();
        let height = height.clone();

        let rerender = move || {
            let mut state = state.borrow_mut();
            if state.turn_queue.len() > 0 {
                if state.turn_progress > 1.0 {
                    state.puzzle_state = state
                        .puzzle
                        .get_derived_state(&state.puzzle_state, &state.turn_queue[0]);
                    state.turn_queue.pop_front();
                    state.turn_progress = 0.0;
                    if state.is_solving && state.turn_queue.is_empty() {
                        solve_next_step(&mut state);
                    }
                } else {
                    state.turn_progress += 0.02;
                }
            }
            if !unsafe { CURSOR_DOWN } {
                render(&state, &canvas_ctx, width.get(), height.get(), 0, 0, false);
            }
        };

        let time_listener = Closure::wrap(Box::new(rerender) as Box<dyn FnMut()>);
        window.set_interval_with_callback_and_timeout_and_arguments_0(
            time_listener.as_ref().unchecked_ref(),
            1,
        )?;
        time_listener.forget();
    }

    Ok(())
}

fn render(
    state: &State,
    canvas_ctx: &web_sys::CanvasRenderingContext2d,
    width: i32,
    height: i32,
    cursor_x: i32,
    cursor_y: i32,
    cursor_down: bool,
) {
    let mut camera: Camera = (unsafe { ORBIT_START_CAMERA })
        .unwrap_or_else(|| Camera::new_towards(Vector3D::new(4.0, 2.0, 2.0), Vector3D::zero()));

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

    let orange = Color::new(254, 133, 57);
    let white = Color::new(231, 224, 220);
    let blue = Color::new(45, 81, 157);
    let red = Color::new(221, 30, 18);
    let dark_red = Color::new(143, 33, 25);
    let green = Color::new(35, 168, 74);
    let yellow = Color::new(219, 226, 35);
    let purple = Color::new(197, 107, 197);

    let colors = [white, blue, orange, green, red, yellow, purple, dark_red];

    let uncolored_faces = if state.turn_queue.len() > 0 {
        let turned_puzzle = state.puzzle.get_physically_turned_faces(
            &state.turn_queue[0],
            &state.puzzle_state,
            state.turn_progress,
        );
        turned_puzzle
    } else {
        state.puzzle.faces(&state.puzzle_state).clone()
    };
    let faces: Vec<FaceWithColor> = uncolored_faces
        .iter()
        .map(
            |PieceFace {
                 face,
                 color_index: i,
                 ..
             }| FaceWithColor {
                face,
                color: colors[i % colors.len()],
            },
        )
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

    canvas_ctx.set_fill_style(&"#ffffff".into());
    canvas_ctx.set_font("30px Arial");
    canvas_ctx
        .fill_text(
            &format!(
                "{:.1}% solved",
                state.puzzle.get_num_solved_pieces(&state.puzzle_state) as f64
                    / state.puzzle.get_num_pieces() as f64
                    * 100.0
            ),
            10.0,
            50.0,
        )
        .unwrap();
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
