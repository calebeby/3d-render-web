use crate::plane::Plane;
use crate::vector3d::Vector3D;
use std::collections::VecDeque;
use std::f64::consts::{PI, TAU};

#[derive(Debug, Clone)]
pub struct Face {
    pub vertices: Vec<Vector3D>,
}

impl Face {
    pub fn edges_iter(&self) -> impl Iterator<Item = Edge> + '_ {
        self.vertices
            .iter()
            .enumerate()
            .map(move |(i, vertex_a)| Edge(*vertex_a, self.vertices[(i + 1) % self.vertices.len()]))
    }

    pub fn rotate_about_axis(
        &self,
        rotation_vector: Vector3D,
        rotation_axis_position: Vector3D,
    ) -> Self {
        Self {
            vertices: self
                .vertices
                .iter()
                .map(|vertex| vertex.rotate_about_axis(rotation_vector, rotation_axis_position))
                .rev() // Need to be reversed because rotation flips the direction of the face, so we need to flip the order back (otherwise the subsequent folds will be the wrong direction)
                .collect(),
        }
    }

    pub fn plane(&self) -> Plane {
        let center = Vector3D::from_average(&self.vertices);

        let edge_1_vector = &self.vertices[1] - &self.vertices[0];
        let edge_2_vector = &self.vertices[2] - &self.vertices[1];

        Plane {
            point: center,
            normal: edge_1_vector.cross(&edge_2_vector),
        }
    }
}

#[derive(Debug)]
pub struct Polyhedron {
    pub faces: Vec<Face>,
    pub vertices: Vec<Vector3D>,
    /// Distance from the origin to the center of a face
    pub inradius: f64,
}

impl Polyhedron {
    // p means the faces are p-sided polygons
    // q means there are q faces on each vertex
    pub fn generate(p: usize, q: usize) -> Polyhedron {
        // Dihedral angle is the angle between adjacent faces
        let dihedral_angle = 2.0 * ((PI / q as f64).cos() / (PI / p as f64).sin()).asin();
        let edge_length = 1.0;
        let dihedral_angle_cos = dihedral_angle.cos();
        // Inradius is the radius of an inscribed sphere (distance from center to face)
        // Used to offset the base face from the origin
        let inradius = edge_length / (2.0 * (PI / p as f64).tan())
            * ((1.0 - dihedral_angle_cos) / (1.0 + dihedral_angle_cos)).sqrt();

        let mut bottom_vertices = vec![];

        let angle_between_vertices = TAU / p as f64;
        // sin(theta/2) = (edge_length / 2) / vertex_to_face_center
        let vertex_to_face_center = (edge_length / 2.0) / (angle_between_vertices / 2.0).sin();

        let mut vertices = vec![];

        // Base polygon
        for i in 0..p {
            let rotation_amount = angle_between_vertices * i as f64;
            let vertex = Vector3D::new(vertex_to_face_center, 0.0, inradius)
                .rotate_about_origin(Vector3D::new(0.0, 0.0, rotation_amount));
            bottom_vertices.push(vertex);
        }

        let bottom_face = Face {
            vertices: bottom_vertices,
        };
        // Edges which have one associated face but not yet two
        let mut incomplete_edges: VecDeque<QueuedEdge> = VecDeque::new();
        let mut faces = vec![bottom_face];
        for edge in faces[0].edges_iter() {
            vertices.push(edge.0);
            incomplete_edges.push_back(QueuedEdge {
                edge,
                face_index: 0,
            });
        }

        while let Some(queued_edge) = incomplete_edges.pop_front() {
            let Edge(vertex_a, vertex_b) = &queued_edge.edge;
            let existing_face = &faces[queued_edge.face_index];
            let rotation_axis_direction = &(vertex_a - vertex_b).to_unit_vector();
            let rotation_vector = rotation_axis_direction * (dihedral_angle);
            let new_face = existing_face.rotate_about_axis(rotation_vector, *vertex_a);
            // It is possible that this face will create an edge that is already queued.
            // If it does, it needs to remove that edge
            // (because now it is a complete edge with two attached faces)

            let new_face_index = faces.len();
            for edge in new_face.edges_iter() {
                if edge.approx_equals(&queued_edge.edge) {
                    continue;
                }
                let matching_existing_edge = incomplete_edges
                    .iter()
                    .position(|incomplete_edge| edge.approx_equals(&incomplete_edge.edge));
                if let Some(matching_existing_edge) = matching_existing_edge {
                    // The edge on the new face matches an existing unmatched edge
                    // Remove the previously-unmatched edge from incomplete_edges
                    incomplete_edges.remove(matching_existing_edge);
                } else {
                    if !vertices.iter().any(|v| v.approx_equals(&edge.0)) {
                        vertices.push(edge.0);
                    }
                    // The edge on the new face does not match an existing unmatched edge
                    incomplete_edges.push_back(QueuedEdge {
                        edge,
                        face_index: new_face_index,
                    });
                }
            }
            faces.push(new_face);
        }

        Polyhedron {
            faces,
            vertices,
            inradius,
        }
    }
    pub fn face_pairs(&self) -> Vec<(&Face, &Face)> {
        let mut face_pairs = vec![None; self.faces.len()];
        let mut paired_faces: Vec<(&Face, &Face)> = vec![];
        for (i, face) in self.faces.iter().enumerate() {
            if face_pairs[i].is_some() {
                continue;
            }
            let opposite_face = self.faces.iter().enumerate().find(|(j, f)| {
                if *j == i || face_pairs[*j].is_some() {
                    return false;
                }
                let cross_product = f.plane().normal.cross(&face.plane().normal);
                cross_product.magnitude().abs() < 1e-8
            });
            if let Some((opposite_face_index, opposite_face)) = opposite_face {
                face_pairs[i] = Some(opposite_face_index);
                face_pairs[opposite_face_index] = Some(i);
                paired_faces.push((face, opposite_face));
            }
        }
        paired_faces
    }
}

#[derive(Debug)]
struct QueuedEdge {
    edge: Edge,
    face_index: usize,
}

#[derive(Debug)]
pub struct Edge(pub Vector3D, pub Vector3D);
impl Edge {
    pub fn approx_equals(&self, other: &Edge) -> bool {
        (self.0.approx_equals(&other.0) && self.1.approx_equals(&other.1))
            || (self.0.approx_equals(&other.1) && self.1.approx_equals(&other.0))
    }
}

