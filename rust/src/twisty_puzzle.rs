use web_sys::console;

use crate::polyhedron::{Face, Polyhedron};
use crate::vector3d::Vector3D;
use crate::{Plane, Ray};

const CUT_PLANE_THICKNESS: f64 = 0.005;

#[derive(Debug)]
pub struct Cut<'a> {
    name: &'a str,
    plane: Plane,
}
impl<'a> Cut<'a> {
    pub fn new(name: &'a str, plane: Plane) -> Self {
        Self { name, plane }
    }
}

type ColorIndex = usize;

#[derive(Clone)]
pub struct FaceWithColorIndex(pub Face, pub ColorIndex);

pub struct TwistyPuzzle {
    faces: Vec<FaceWithColorIndex>,
}

impl TwistyPuzzle {
    pub fn new(polyhedron: &Polyhedron, cuts: &[Cut]) -> Self {
        console::log_1(&"new twisty_puzzle".into());
        let mut faces: Vec<FaceWithColorIndex> = polyhedron
            .faces
            .iter()
            .enumerate()
            .map(|(color_index, face)| FaceWithColorIndex(face.clone(), color_index as _))
            .collect();
        for cut in cuts {
            let mut faces_above_plane: Vec<FaceWithColorIndex> = vec![];
            let mut faces_below_plane: Vec<FaceWithColorIndex> = vec![];
            let cut_plane_outer = cut.plane.offset(CUT_PLANE_THICKNESS);
            let cut_plane_inner = cut.plane.offset(-CUT_PLANE_THICKNESS);
            for FaceWithColorIndex(face, color_index) in &faces {
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
                    faces_above_plane.push(FaceWithColorIndex(
                        Face {
                            vertices: vertices_above_plane,
                        },
                        *color_index,
                    ));
                }
                if vertices_below_plane.len() > 2 {
                    faces_below_plane.push(FaceWithColorIndex(
                        Face {
                            vertices: vertices_below_plane,
                        },
                        *color_index,
                    ));
                }
            }
            faces = faces_above_plane
                .iter()
                .chain(faces_below_plane.iter())
                .cloned()
                .collect();
        }

        Self { faces }
    }
    pub fn faces(&self) -> &Vec<FaceWithColorIndex> {
        &self.faces
    }
}
