//! Finite floor and freestanding box: approach, move sideways, pass the box.
use avatar_physics::{mesh::Triangle, Character, Input, Profile, DT};
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
        [-5.0, -5.0, 0.0],
        [10.0, -5.0, 0.0],
        [10.0, 5.0, 0.0],
        [-5.0, 5.0, 0.0],
    );
    let [a, b, c, d, e, f, g, h] = [
        [1.0, -0.5, 0.0],
        [2.0, -0.5, 0.0],
        [2.0, 0.5, 0.0],
        [1.0, 0.5, 0.0],
        [1.0, -0.5, 1.0],
        [2.0, -0.5, 1.0],
        [2.0, 0.5, 1.0],
        [1.0, 0.5, 1.0],
    ];
    for face in [
        [a, b, f, e],
        [b, c, g, f],
        [c, d, h, g],
        [d, a, e, h],
        [e, f, g, h],
    ] {
        quad(&mut mesh, face[0], face[1], face[2], face[3]);
    }
    let p = Profile::default();
    let mut avatar = Character::new(p, [0.0, 0.0, p.height / 2.0 + 0.0001]).unwrap();
    println!("time,x,y,feet_z,grounded");
    for i in 0..480 {
        let movement = if i < 120 {
            [1.0, 0.0]
        } else if i < 240 {
            [0.0, 1.0]
        } else {
            [1.0, 0.0]
        };
        let s = avatar
            .step_mesh(
                Input {
                    movement,
                    jump: false,
                },
                &mesh,
            )
            .unwrap();
        assert!(s.position[2] - p.height / 2.0 > -0.002);
        if i == 119 {
            assert!(s.position[0] < 0.701);
        }
        println!(
            "{:.6},{:.6},{:.6},{:.6},{}",
            (i + 1) as f64 * DT,
            s.position[0],
            s.position[1],
            s.position[2] - p.height / 2.0,
            s.grounded
        );
    }
    assert!(avatar.state().position[0] > 3.0 && avatar.state().position[1] > 1.0);
}
