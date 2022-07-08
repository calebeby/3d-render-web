use std::collections::HashMap;

use crate::face_map::FaceMap;
use rand::distributions::Uniform;
use rand::Rng;

use crate::plane::Plane;
use crate::polyhedron::{Face, Polyhedron};
use crate::ray::Ray;
use crate::vector3d::Vector3D;

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
    /// List of turn indices that move this face
    affecting_turn_indices: Vec<usize>,
}

#[derive(Debug)]
struct PhysicalTurn {
    rotation_axis: Vector3D,
    rotation_axis_point: Vector3D,
}

#[derive(Debug)]
pub(crate) struct Turn {
    // The indices of this vector are the new face indexes.
    // The values are the old face indexes to pull colors from.
    pub(crate) face_map: FaceMap,
    physical_turn: PhysicalTurn,
}

pub struct TwistyPuzzle {
    pub faces: Vec<PieceFace>,
    pub(crate) turns: Vec<Turn>,
    pub turn_names: Vec<String>,
    // Each piece is a vector of its face indexes
    pieces: Vec<Vec<usize>>,
}

impl TwistyPuzzle {
    pub fn new(polyhedron: &Polyhedron, cuts: &[CutDefinition]) -> Self {
        let mut physical_turns: Vec<(String, PhysicalTurn)> = vec![];
        let mut inferred_name_i = b'A';
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
                affecting_turn_indices: vec![],
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
            let forwards_turn_index = physical_turns.len() - 1;
            let reverse_turn_index = physical_turns.len() - 2;
            let mut updated_faces: Vec<PieceFace> = vec![];
            let cut_plane_outer = cut.plane.offset(CUT_PLANE_THICKNESS);
            let cut_plane_inner = cut.plane.offset(-CUT_PLANE_THICKNESS);
            for PieceFace {
                face,
                color_index,
                affecting_turn_indices,
            } in &faces
            {
                let mut vertices_above_plane = VertexList::new();
                let mut vertices_below_plane = VertexList::new();
                // Pairs of (vertex, is_above_cut_plane)
                let vertices_with_status: Vec<_> = face
                    .vertices
                    .iter()
                    // Make the last vertex appear again at the end so all edges are included
                    .chain(std::iter::once(&face.vertices[0]))
                    .map(|vertex| {
                        let is_above_cut_plane =
                            (vertex - cut.plane.point).dot(&cut.plane.normal) > 0.0;
                        (vertex, is_above_cut_plane)
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
                        let above_intersection = cut_plane_outer.intersection(&edge_ray);
                        let below_intersection = cut_plane_inner.intersection(&edge_ray);
                        vertices_above_plane.push(above_intersection);
                        vertices_below_plane.push(below_intersection);
                    }
                }
                let vertices_above_plane = vertices_above_plane.to_vec();
                let vertices_below_plane = vertices_below_plane.to_vec();

                if vertices_above_plane.len() > 2 {
                    let mut new_affecting_turn_indices = affecting_turn_indices.clone();
                    new_affecting_turn_indices.push(forwards_turn_index);
                    new_affecting_turn_indices.push(reverse_turn_index);
                    updated_faces.push(PieceFace {
                        face: Face {
                            vertices: vertices_above_plane,
                        },
                        color_index: *color_index,
                        affecting_turn_indices: new_affecting_turn_indices,
                    });
                }
                if vertices_below_plane.len() > 2 {
                    updated_faces.push(PieceFace {
                        face: Face {
                            vertices: vertices_below_plane,
                        },
                        color_index: *color_index,
                        affecting_turn_indices: affecting_turn_indices.clone(),
                    });
                }
            }
            faces = updated_faces;
        }

        // Pieces decides which physical faces are attached together
        let mut pieces_map: HashMap<_, Vec<usize>> = HashMap::new();
        for (face_i, face) in faces.iter().enumerate() {
            let mut affecting_turn_names = face.affecting_turn_indices.clone();
            affecting_turn_names.sort_unstable();
            match pieces_map.get_mut(&affecting_turn_names) {
                Some(faces) => faces.push(face_i),
                None => {
                    pieces_map.insert(affecting_turn_names, vec![face_i]);
                }
            }
        }
        let pieces: Vec<_> = pieces_map.into_values().collect();

        let face_centers: Vec<Vector3D> = faces
            .iter()
            .map(|face| Vector3D::from_average(&face.face.vertices))
            .collect();

        // try out each of the turns to determine the symmetries between pieces
        // and which faces map to which faces after each turn
        let (turn_names, turns): (Vec<_>, Vec<_>) = physical_turns
            .into_iter()
            .enumerate()
            .map(|(turn_index, (turn_name, physical_turn))| {
                let face_map = FaceMap(
                    faces
                        .iter()
                        .enumerate()
                        .map(|(i, face)| {
                            if face.affecting_turn_indices.contains(&turn_index) {
                                let original_location = &face_centers[i];
                                let new_location = original_location.rotate_about_axis(
                                    physical_turn.rotation_axis,
                                    physical_turn.rotation_axis_point,
                                );
                                // Find the index in the old faces array
                                // which corresponds to the new position
                                face_centers
                                    .iter()
                                    .position(|old_location| {
                                        old_location.approx_equals(&new_location)
                                    })
                                    .unwrap_or(i)
                                // .unwrap_or(usize::MAX)
                            } else {
                                // this turn does not affect this face; map to itself
                                i
                            }
                        })
                        .collect(),
                );

                let turn = Turn {
                    physical_turn,
                    face_map: face_map.invert(),
                };
                (turn_name, turn)
            })
            .unzip();

        Self {
            faces,
            turns,
            turn_names,
            pieces,
        }
    }

    #[inline]
    pub fn get_num_faces(&self) -> usize {
        self.faces.len()
    }

    #[inline]
    pub fn get_num_pieces(&self) -> usize {
        self.pieces.len()
    }

    pub fn get_num_solved_pieces(&self, puzzle_state: &PuzzleState) -> usize {
        let faces_solved_states: Vec<bool> = puzzle_state
            .iter()
            .enumerate()
            .map(|(i, color_index)| *color_index == self.faces[i].color_index)
            .collect();

        self.pieces
            .iter()
            .fold(0, |num_solved_pieces, piece_faces| {
                let every_face_solved = piece_faces
                    .iter()
                    .all(|face_index| faces_solved_states[*face_index]);
                if every_face_solved {
                    num_solved_pieces + 1
                } else {
                    num_solved_pieces
                }
            })
    }

    pub fn faces(&self, puzzle_state: &PuzzleState) -> Vec<PieceFace> {
        self.faces
            .iter()
            .enumerate()
            .map(|(i, piece_face)| PieceFace {
                face: piece_face.face.clone(),
                affecting_turn_indices: piece_face.affecting_turn_indices.clone(),
                color_index: puzzle_state[i],
            })
            .collect()
    }

    pub fn get_physically_turned_faces(
        &self,
        turn_index: usize,
        puzzle_state: &PuzzleState,
        interpolate_amount: f64,
    ) -> Vec<PieceFace> {
        let cut = &self.turns[turn_index];
        let new_faces = self
            .faces
            .iter()
            .enumerate()
            .map(|(i, piece_face)| PieceFace {
                face: if piece_face.affecting_turn_indices.contains(&turn_index) {
                    piece_face.face.rotate_about_axis(
                        interpolate_amount * cut.physical_turn.rotation_axis,
                        cut.physical_turn.rotation_axis_point,
                    )
                } else {
                    piece_face.face.clone()
                },
                affecting_turn_indices: piece_face.affecting_turn_indices.clone(),
                color_index: puzzle_state[i],
            })
            .collect();
        new_faces
    }

    pub fn get_initial_state(&self) -> PuzzleState {
        self.faces.iter().map(|face| face.color_index).collect()
    }

    pub fn get_derived_state(
        &self,
        previous_state: &PuzzleState,
        face_map: &FaceMap,
    ) -> PuzzleState {
        face_map
            .0
            .iter()
            .map(|old_face_index| previous_state[*old_face_index])
            .collect()
    }

    pub fn get_derived_state_turn_index(
        &self,
        previous_state: &PuzzleState,
        turn_index: usize,
    ) -> PuzzleState {
        let face_map = &self.turns.get(turn_index).unwrap().face_map;
        self.get_derived_state(previous_state, face_map)
    }

    #[allow(dead_code)]
    pub fn get_derived_state_from_turns_iter(
        &self,
        previous_state: &PuzzleState,
        turns: impl Iterator<Item = usize>,
    ) -> PuzzleState {
        turns.fold(previous_state.clone(), |state, turn_index| {
            self.get_derived_state_turn_index(&state, turn_index)
        })
    }

    // TODO: remove this, turn_names is public?
    #[inline]
    pub fn turn_names_iter(&self) -> impl Iterator<Item = &String> + '_ {
        self.turn_names.iter()
    }

    pub fn scramble(&self, initial_state: &PuzzleState, limit: u64) -> PuzzleState {
        let mut state = initial_state.clone();

        let num_turns = self.turns.len();

        let mut rng = rand::thread_rng();
        let range = Uniform::new(0, num_turns);

        for _ in 0..limit {
            state = self.get_derived_state_turn_index(&state, rng.sample(range));
        }

        state
    }
}

pub type PuzzleState = Vec<usize>;

// A Vec<Vector3D> but it prevents two adjacent items from being equal or approx equal
// Also prevents the first and last from being equal or approx equal
struct VertexList {
    vec: Vec<Vector3D>,
}

impl VertexList {
    fn new() -> Self {
        VertexList { vec: vec![] }
    }
    fn push(&mut self, vertex: Vector3D) {
        match self.vec.last() {
            Some(last) if last.approx_equals(&vertex) => {}
            _ => {
                self.vec.push(vertex);
            }
        };
    }
    fn to_vec(mut self) -> Vec<Vector3D> {
        match (self.vec.first(), self.vec.last()) {
            (Some(first), Some(last)) if first.approx_equals(last) => {
                self.vec.pop();
            }
            _ => {}
        };
        self.vec
    }
}
