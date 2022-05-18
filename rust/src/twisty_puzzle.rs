use std::collections::HashMap;

use wasm_bindgen::UnwrapThrowExt;
use web_sys::console;

use crate::polyhedron::{Face, Polyhedron};
use crate::vector3d::Vector3D;
use crate::{Plane, Ray};

const CUT_PLANE_THICKNESS: f64 = 0.005;

#[derive(Debug)]
pub struct CutDefinition<'a> {
    name: Option<&'a str>,
    plane: Plane,
    rotation_angle: f64,
}
impl<'a> CutDefinition<'a> {
    pub fn new(name: &'a str, plane: Plane, rotation_angle: f64) -> Self {
        Self {
            name: Some(name),
            plane,
            rotation_angle,
        }
    }
    pub fn new_infer_name(plane: Plane, rotation_angle: f64) -> Self {
        Self {
            name: None,
            plane,
            rotation_angle,
        }
    }
}

type ColorIndex = usize;

#[derive(Debug, Clone)]
pub struct PieceFace {
    pub face: Face,
    pub color_index: ColorIndex,
    /// List of cut names that move this face
    affecting_turn_names: Vec<String>,
}

pub struct TwistyPuzzle {
    faces: Vec<PieceFace>,
    turns: HashMap<String, Turn>,
}

#[derive(Debug)]
struct PhysicalTurn {
    rotation_axis: Vector3D,
    rotation_axis_point: Vector3D,
}

#[derive(Debug)]
struct Turn {
    // The indices of this vector are the new face indexes.
    // The values are the old face indexes to pull colors from.
    face_map: Vec<usize>,
    physical_turn: PhysicalTurn,
}

impl TwistyPuzzle {
    pub fn new(polyhedron: &Polyhedron, cuts: &[CutDefinition]) -> Self {
        let mut physical_turns: Vec<(String, PhysicalTurn)> = vec![];
        let mut inferred_name_i = 'A' as u8;
        let cuts_with_names = cuts.iter().map(|cut| {
            let cut_name = match cut.name {
                Some(name) => name.to_string(),
                None => {
                    let char = inferred_name_i as char;
                    inferred_name_i += 1;
                    char.to_string()
                }
            };
            (cut_name, cut)
        });
        let mut faces: Vec<PieceFace> = polyhedron
            .faces
            .iter()
            .enumerate()
            .map(|(color_index, face)| PieceFace {
                face: face.clone(),
                color_index: color_index as _,
                affecting_turn_names: vec![],
            })
            .collect();
        for (turn_name, cut) in cuts_with_names {
            let inverted_turn_name = format!("{}'", turn_name);
            let rotation_axis = -1.0 * cut.rotation_angle * cut.plane.normal.to_unit_vector();
            physical_turns.push((
                turn_name.clone(),
                PhysicalTurn {
                    rotation_axis,
                    rotation_axis_point: cut.plane.point,
                },
            ));
            physical_turns.push((
                inverted_turn_name.clone(),
                PhysicalTurn {
                    rotation_axis: -1.0 * rotation_axis,
                    rotation_axis_point: cut.plane.point,
                },
            ));
            let mut updated_faces: Vec<PieceFace> = vec![];
            let cut_plane_outer = cut.plane.offset(CUT_PLANE_THICKNESS);
            let cut_plane_inner = cut.plane.offset(-CUT_PLANE_THICKNESS);
            for PieceFace {
                face,
                color_index,
                affecting_turn_names,
            } in &faces
            {
                let mut vertices_above_plane: Vec<Vector3D> = vec![];
                let mut vertices_below_plane: Vec<Vector3D> = vec![];
                // Pairs of (vertex, is_above_cut_plane)
                let vertices_with_status: Vec<_> = face
                    .vertices
                    .iter()
                    // Make the last vertex appear again at the end so all edges are included
                    .chain(std::iter::once(&face.vertices[0]))
                    .map(|vertex| {
                        (
                            vertex,
                            (vertex - &cut.plane.point).dot(&cut.plane.normal) > 0.0,
                        )
                    })
                    .collect();
                let edges = vertices_with_status.windows(2);
                for edge in edges {
                    let (&vertex_a, a_is_above_plane) = edge[0];
                    let (&vertex_b, b_is_above_plane) = edge[1];
                    if a_is_above_plane && b_is_above_plane {
                        vertices_above_plane.push(vertex_a);
                    } else if !a_is_above_plane && !b_is_above_plane {
                        vertices_below_plane.push(vertex_a);
                    } else {
                        // This edge crosses the plane
                        if a_is_above_plane {
                            vertices_above_plane.push(vertex_a);
                        } else {
                            vertices_below_plane.push(vertex_a);
                        }
                        let edge_ray = Ray {
                            point: vertex_a,
                            direction: vertex_a - &vertex_b,
                        };
                        vertices_above_plane.push(cut_plane_outer.intersection(&edge_ray));
                        vertices_below_plane.push(cut_plane_inner.intersection(&edge_ray));
                    }
                }
                if vertices_above_plane.len() > 2 {
                    let mut new_affecting_turn_names = affecting_turn_names.clone();
                    new_affecting_turn_names.push(turn_name.to_string());
                    new_affecting_turn_names.push(inverted_turn_name.to_string());
                    updated_faces.push(PieceFace {
                        face: Face {
                            vertices: vertices_above_plane,
                        },
                        color_index: *color_index,
                        affecting_turn_names: new_affecting_turn_names,
                    });
                }
                if vertices_below_plane.len() > 2 {
                    updated_faces.push(PieceFace {
                        face: Face {
                            vertices: vertices_below_plane,
                        },
                        color_index: *color_index,
                        affecting_turn_names: affecting_turn_names.clone(),
                    });
                }
            }
            faces = updated_faces;
        }

        let face_centers: Vec<Vector3D> = faces
            .iter()
            .map(|face| Vector3D::from_average(&face.face.vertices))
            .collect();

        // try out each of the turns to determine the symmetries between pieces
        // and which faces map to which faces after each turn
        let turns: HashMap<_, _> = physical_turns
            .into_iter()
            .map(|(turn_name, physical_turn)| {
                let face_map: Vec<_> = faces
                    .iter()
                    .enumerate()
                    .map(|(i, face)| {
                        if face.affecting_turn_names.contains(&turn_name) {
                            let original_location = &face_centers[i];
                            let new_location = original_location.rotate_about_axis(
                                physical_turn.rotation_axis,
                                physical_turn.rotation_axis_point,
                            );
                            // Find the index in the old faces array
                            // which corresponds to the new position
                            face_centers
                                .iter()
                                .position(|old_location| old_location.approx_equals(&new_location))
                                .unwrap_or(i)
                        } else {
                            // this turn does not affect this face; map to itself
                            i
                        }
                    })
                    .collect();

                let mut inverted_face_map = vec![0; face_map.len()];
                for (val, i) in face_map.iter().enumerate() {
                    inverted_face_map[*i] = val;
                }
                let turn = Turn {
                    physical_turn,
                    face_map: inverted_face_map,
                };
                (turn_name, turn)
            })
            .collect();

        Self { faces, turns }
    }

    pub fn get_percent_solved(&self, puzzle_state: &PuzzleState) -> f64 {
        let mut num_solved_faces = 0;
        for (i, color_index) in puzzle_state.iter().enumerate() {
            if *color_index == self.faces[i].color_index {
                num_solved_faces += 1;
            }
        }
        num_solved_faces as f64 / self.faces.len() as f64
    }

    pub fn faces(&self, puzzle_state: &PuzzleState) -> Vec<PieceFace> {
        self.faces
            .iter()
            .enumerate()
            .map(|(i, piece_face)| PieceFace {
                face: piece_face.face.clone(),
                affecting_turn_names: piece_face.affecting_turn_names.clone(),
                color_index: puzzle_state[i],
            })
            .collect()
    }

    pub fn get_physically_turned_faces(
        &self,
        turn_name: &str,
        puzzle_state: &PuzzleState,
        interpolate_amount: f64,
    ) -> Vec<PieceFace> {
        let cut = &self.turns[turn_name];
        let new_faces = self
            .faces
            .iter()
            .enumerate()
            .map(|(i, piece_face)| PieceFace {
                face: if piece_face
                    .affecting_turn_names
                    .contains(&turn_name.to_string())
                {
                    piece_face.face.rotate_about_axis(
                        interpolate_amount * cut.physical_turn.rotation_axis,
                        cut.physical_turn.rotation_axis_point,
                    )
                } else {
                    piece_face.face.clone()
                },
                affecting_turn_names: piece_face.affecting_turn_names.clone(),
                color_index: puzzle_state[i],
            })
            .collect();
        new_faces
    }

    pub fn get_initial_state(&self) -> PuzzleState {
        self.faces.iter().map(|face| face.color_index).collect()
    }

    pub fn get_derived_state(&self, previous_state: &PuzzleState, turn_name: &str) -> PuzzleState {
        let face_map = &self.turns.get(turn_name).unwrap_throw().face_map;
        face_map
            .iter()
            .map(|old_face_index| previous_state[*old_face_index])
            .collect()
    }

    pub fn turns_iter(&self) -> impl Iterator<Item = &String> + '_ {
        self.turns.iter().map(|turn| turn.0)
    }
}

pub type PuzzleState = Vec<usize>;
