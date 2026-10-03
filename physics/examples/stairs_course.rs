//! Offline three-riser stair course; input walks forward without jumping.
use avatar_physics::{
    mesh::{StaticMesh, Triangle},
    Character, Input, Profile, DT,
};
fn quad(mesh: &mut Vec<Triangle>, a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3]) {
    for vertices in [[a, b, c], [a, c, d]] {
        mesh.push(Triangle {
            id: mesh.len() as u64,
            vertices,
        });
    }
}
fn main() {
    let mut mesh = Vec::new();
    quad(
        &mut mesh,
        [-5.0, -3.0, 0.0],
        [8.0, -3.0, 0.0],
        [8.0, 3.0, 0.0],
        [-5.0, 3.0, 0.0],
    );
    for i in 0..3 {
        let x = 1.0 + i as f64 * 0.8;
        let z = 0.15 * (i + 1) as f64;
        quad(
            &mut mesh,
            [x, -3.0, z - 0.15],
            [x, 3.0, z - 0.15],
            [x, 3.0, z],
            [x, -3.0, z],
        );
        quad(
            &mut mesh,
            [x, -3.0, z],
            [x + 0.8, -3.0, z],
            [x + 0.8, 3.0, z],
            [x, 3.0, z],
        );
    }
    let mesh = StaticMesh::new(mesh).unwrap();
    let p = Profile::default();
    let mut avatar = Character::new(p, [0.0, 0.0, p.height / 2.0 + 0.0001]).unwrap();
    println!("time,x,feet_z,grounded");
    for i in 0..230 {
        let s = avatar
            .step_static_mesh(
                Input {
                    movement: if avatar.state().position[0] < 3.0 {
                        [1.0, 0.0]
                    } else {
                        [0.0; 2]
                    },
                    jump: false,
                },
                &mesh,
            )
            .unwrap();
        println!(
            "{:.6},{:.6},{:.6},{}",
            (i + 1) as f64 * DT,
            s.position[0],
            s.position[2] - p.height / 2.0,
            s.grounded
        );
    }
    assert!(avatar.state().position[0] > 2.8);
    assert!((avatar.state().position[2] - p.height / 2.0 - 0.45).abs() < 0.003);
    assert!(avatar.state().grounded);
}
