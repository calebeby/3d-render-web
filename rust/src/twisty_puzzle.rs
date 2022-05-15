use web_sys::console;

use crate::polyhedron::{Face, Polyhedron};
use crate::vector3d::Vector3D;
use crate::{Plane, Ray};

pub struct TwistyPuzzle {
    faces: Vec<Face>,
}

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

impl TwistyPuzzle {
    pub fn new(polyhedron: &Polyhedron, cuts: &[Cut]) -> Self {
        let mut faces = polyhedron.faces.clone();
        for cut in cuts {
            let mut faces_above_plane: Vec<Face> = vec![];
            let mut faces_below_plane: Vec<Face> = vec![];
            for face in &faces {
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
                        let intersection_point = cut.plane.intersection(&edge_ray);
                        vertices_above_plane.push(intersection_point);
                        vertices_below_plane.push(intersection_point);
                    }
                }
                if vertices_above_plane.len() > 2 {
                    faces_above_plane.push(Face {
                        vertices: vertices_above_plane,
                    });
                }
                if vertices_below_plane.len() > 2 {
                    faces_below_plane.push(Face {
                        vertices: vertices_below_plane,
                    });
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
    pub fn faces(&self) -> &Vec<Face> {
        &self.faces
    }
}
