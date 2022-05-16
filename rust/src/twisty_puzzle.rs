use std::collections::HashMap;

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
    affecting_cut_indices: Vec<String>,
}

#[derive(Clone)]
pub struct TwistyPuzzle {
    faces: Vec<PieceFace>,
    cuts: HashMap<String, Cut>,
}

#[derive(Clone)]
struct Cut {
    rotation_axis: Vector3D,
    rotation_axis_point: Vector3D,
}

impl TwistyPuzzle {
    pub fn new(polyhedron: &Polyhedron, cuts: &[CutDefinition]) -> Self {
        let mut cuts_map: HashMap<String, Cut> = HashMap::new();
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
                affecting_cut_indices: vec![],
            })
            .collect();
        for (cut_name, cut) in cuts_with_names {
            cuts_map.insert(
                cut_name.clone(),
                Cut {
                    rotation_axis: cut.rotation_angle * cut.plane.normal.to_unit_vector(),
                    rotation_axis_point: cut.plane.point,
                },
            );
            let mut faces_above_plane: Vec<PieceFace> = vec![];
            let mut faces_below_plane: Vec<PieceFace> = vec![];
            let cut_plane_outer = cut.plane.offset(CUT_PLANE_THICKNESS);
            let cut_plane_inner = cut.plane.offset(-CUT_PLANE_THICKNESS);
            for PieceFace {
                face,
                color_index,
                affecting_cut_indices,
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
                    let mut new_affecting_cut_indices = affecting_cut_indices.clone();
                    new_affecting_cut_indices.push(cut_name.to_string());
                    faces_above_plane.push(PieceFace {
                        face: Face {
                            vertices: vertices_above_plane,
                        },
                        color_index: *color_index,
                        affecting_cut_indices: new_affecting_cut_indices,
                    });
                }
                if vertices_below_plane.len() > 2 {
                    faces_below_plane.push(PieceFace {
                        face: Face {
                            vertices: vertices_below_plane,
                        },
                        color_index: *color_index,
                        affecting_cut_indices: affecting_cut_indices.clone(),
                    });
                }
            }
            faces = faces_above_plane
                .iter()
                .chain(faces_below_plane.iter())
                .cloned()
                .collect();
        }

        Self {
            faces,
            cuts: cuts_map,
        }
    }
    pub fn faces(&self) -> &Vec<PieceFace> {
        &self.faces
    }

    pub fn get_turned_faces(&self, cut_name: &str, interpolate_amount: f64) -> Vec<PieceFace> {
        let cut = &self.cuts[cut_name];
        let new_faces = self
            .faces
            .iter()
            .map(|piece_face| PieceFace {
                face: if piece_face
                    .affecting_cut_indices
                    .contains(&cut_name.to_string())
                {
                    piece_face.face.rotate_about_axis(
                        interpolate_amount * cut.rotation_axis,
                        cut.rotation_axis_point,
                    )
                } else {
                    piece_face.face.clone()
                },
                affecting_cut_indices: piece_face.affecting_cut_indices.clone(),
                color_index: piece_face.color_index,
            })
            .collect();
        new_faces
    }

    pub fn cuts_iter(&self) -> impl Iterator<Item = &String> + '_ {
        self.cuts.iter().map(|cut| cut.0)
    }
}
