use std::f64::consts::TAU;

use crate::plane::Plane;
use crate::polyhedron::Polyhedron;
use crate::twisty_puzzle::{CutDefinition, TwistyPuzzle};

fn tetrahedron() -> Polyhedron {
    Polyhedron::generate(3, 3)
}
fn cube() -> Polyhedron {
    Polyhedron::generate(4, 3)
}
fn octahedron() -> Polyhedron {
    Polyhedron::generate(3, 4)
}
fn dodecahedron() -> Polyhedron {
    Polyhedron::generate(5, 3)
}
fn icosahedron() -> Polyhedron {
    Polyhedron::generate(3, 5)
}

const RUBIKS_CUBE_CUT_NAMES: [&str; 6] = ["U", "F", "R", "B", "L", "D"];

pub fn megaminx() -> TwistyPuzzle {
    let dodecahedron = dodecahedron();
    TwistyPuzzle::new(
        &dodecahedron,
        &dodecahedron
            .faces
            .iter()
            .map(|face| CutDefinition::new_infer_name(face.plane().offset(-0.33), TAU / 5.0))
            .collect::<Vec<_>>(),
    )
}

pub fn starminx() -> TwistyPuzzle {
    let dodecahedron = dodecahedron();
    TwistyPuzzle::new(
        &dodecahedron,
        &dodecahedron
            .faces
            .iter()
            .map(|face| CutDefinition::new_infer_name(face.plane().offset(-0.75), TAU / 5.0))
            .collect::<Vec<_>>(),
    )
}

pub fn eitans_star() -> TwistyPuzzle {
    let icosahedron = icosahedron();
    TwistyPuzzle::new(
        &icosahedron,
        &icosahedron
            .faces
            .iter()
            // This cut is positioned to create an extra edge piece that doesn't exist on the real thing
            // To prevent zero-size pieces from being created
            .map(|face| CutDefinition::new_infer_name(face.plane().offset(-0.27), TAU / 3.0))
            .collect::<Vec<_>>(),
    )
}

pub fn rubiks_cube_3x3() -> TwistyPuzzle {
    let cube = cube();
    TwistyPuzzle::new(
        &cube,
        &cube
            .faces
            .iter()
            .enumerate()
            .map(|(i, face)| {
                CutDefinition::new(
                    RUBIKS_CUBE_CUT_NAMES[i],
                    face.plane().offset(-0.33),
                    TAU / 4.0,
                )
            })
            .collect::<Vec<_>>(),
    )
}

pub fn rubiks_cube_2x2() -> TwistyPuzzle {
    let cube = cube();
    TwistyPuzzle::new(
        &cube,
        &cube.faces[0..=2]
            .iter()
            .enumerate()
            .map(|(i, face)| {
                CutDefinition::new(
                    RUBIKS_CUBE_CUT_NAMES[i],
                    face.plane().offset(-0.5),
                    TAU / 4.0,
                )
            })
            .collect::<Vec<_>>(),
    )
}

pub fn compy_cube() -> TwistyPuzzle {
    let cube = cube();
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
}

pub fn pentultimate() -> TwistyPuzzle {
    let dodecahedron = dodecahedron();
    TwistyPuzzle::new(
        &dodecahedron,
        &dodecahedron
            .face_pairs()
            .iter()
            // .take(2)
            .map(|(face, _opposite_face)| {
                CutDefinition::new_infer_name(
                    face.plane().offset(-dodecahedron.inradius),
                    TAU / 5.0,
                )
            })
            .collect::<Vec<_>>(),
    )
}

pub fn master_pentultimate() -> TwistyPuzzle {
    let dodecahedron = dodecahedron();
    TwistyPuzzle::new(
        &dodecahedron,
        &dodecahedron
            .faces
            .iter()
            .map(|face| CutDefinition::new_infer_name(face.plane().offset(-1.03), TAU / 5.0))
            .collect::<Vec<_>>(),
    )
}

pub fn dino_starminx() -> TwistyPuzzle {
    let dodecahedron = dodecahedron();
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
}

pub fn pyraminx() -> TwistyPuzzle {
    let tetrahedron = tetrahedron();
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
                CutDefinition::new_infer_name(plane.offset(-0.53), TAU / 3.0)
            })
            .collect::<Vec<_>>(),
    )
}

pub fn skewb_diamond() -> TwistyPuzzle {
    let octahedron = octahedron();
    TwistyPuzzle::new(
        &octahedron,
        &octahedron.faces[0..=3]
            .iter()
            .map(|face| CutDefinition::new_infer_name(face.plane().offset(-0.41), TAU / 3.0))
            .collect::<Vec<_>>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_megaminx() {
        let puzzle = megaminx();
        assert_eq!(puzzle.get_num_faces(), 11 * 12);

        let initial_state = puzzle.get_initial_state();
        let turned_state = puzzle.get_derived_state_turn_index(&initial_state, 0);
        let turned_again_state = puzzle.get_derived_state_turn_index(&turned_state, 1);
        assert_eq!(initial_state, turned_again_state);
    }

    #[test]
    fn test_rubiks_cube_3x3() {
        let puzzle = rubiks_cube_3x3();
        assert_eq!(puzzle.get_num_faces(), 9 * 6);

        let initial_state = puzzle.get_initial_state();
        let turned_state = puzzle.get_derived_state_turn_index(&initial_state, 0);
        let turned_again_state = puzzle.get_derived_state_turn_index(&turned_state, 1);
        assert_eq!(initial_state, turned_again_state);
    }

    #[test]
    fn test_rubiks_cube_2x2() {
        let puzzle = rubiks_cube_2x2();
        assert_eq!(puzzle.get_num_faces(), 4 * 6);

        let initial_state = puzzle.get_initial_state();
        let turned_state = puzzle.get_derived_state_turn_index(&initial_state, 0);
        let turned_again_state = puzzle.get_derived_state_turn_index(&turned_state, 1);
        assert_eq!(initial_state, turned_again_state);
    }

    #[test]
    fn test_pyraminx() {
        let puzzle = pyraminx();
        assert_eq!(puzzle.get_num_faces(), 7 * 4);

        let initial_state = puzzle.get_initial_state();
        let turned_state = puzzle.get_derived_state_turn_index(&initial_state, 0);
        let turned_again_state = puzzle.get_derived_state_turn_index(&turned_state, 1);
        assert_eq!(initial_state, turned_again_state);
    }

    #[test]
    fn test_pentultimate() {
        let puzzle = pentultimate();
        assert_eq!(puzzle.get_num_faces(), 6 * 12);

        let initial_state = puzzle.get_initial_state();
        let turned_state = puzzle.get_derived_state_turn_index(&initial_state, 0);
        let turned_again_state = puzzle.get_derived_state_turn_index(&turned_state, 1);
        assert_eq!(initial_state, turned_again_state);
    }

    #[test]
    fn test_master_pentultimate() {
        let puzzle = master_pentultimate();
        assert_eq!(puzzle.get_num_faces(), 16 * 12);

        let initial_state = puzzle.get_initial_state();
        let turned_state = puzzle.get_derived_state_turn_index(&initial_state, 0);
        let turned_again_state = puzzle.get_derived_state_turn_index(&turned_state, 1);
        assert_eq!(initial_state, turned_again_state);
    }

    #[test]
    fn test_eitans_star() {
        let puzzle = eitans_star();
        assert_eq!(puzzle.get_num_faces(), 16 * 20);

        let initial_state = puzzle.get_initial_state();
        let turned_state = puzzle.get_derived_state_turn_index(&initial_state, 0);
        let turned_again_state = puzzle.get_derived_state_turn_index(&turned_state, 1);
        assert_eq!(initial_state, turned_again_state);
    }
}
