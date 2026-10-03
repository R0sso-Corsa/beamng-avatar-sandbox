//! Offline half-space course: floor, corridor walls and low ceiling.
//! This exercises capsule clearance, not triangle mesh collision or stairs.
use avatar_physics::{Character, Input, Plane, Profile, DT};
fn main() {
    let p = Profile::default();
    let surfaces = [
        Plane {
            id: 1,
            normal: [0.0, 0.0, 1.0],
            offset: 0.0,
        },
        Plane {
            id: 2,
            normal: [-1.0, 0.0, 0.0],
            offset: -3.0,
        },
        Plane {
            id: 3,
            normal: [0.0, -1.0, 0.0],
            offset: -1.0,
        },
        Plane {
            id: 4,
            normal: [0.0, 0.0, -1.0],
            offset: -2.0,
        },
    ];
    let mut avatar = Character::new(p, [0.0, 0.0, p.height / 2.0]).unwrap();
    println!("time,x,y,feet_z,head_z,grounded");
    for i in 0..960 {
        let s = avatar
            .step(
                Input {
                    movement: [1.0, 1.0],
                    jump: i == 120,
                },
                &surfaces,
            )
            .unwrap();
        let feet = s.position[2] - p.height / 2.0;
        let head = s.position[2] + p.height / 2.0;
        assert!(feet > -0.002 && head < 2.002);
        assert!(s.position[0] + p.radius < 3.002 && s.position[1] + p.radius < 1.002);
        println!(
            "{:.6},{:.6},{:.6},{:.6},{:.6},{}",
            (i + 1) as f64 * DT,
            s.position[0],
            s.position[1],
            feet,
            head,
            s.grounded
        );
    }
    let s = avatar.state();
    assert!(s.grounded && s.position[0] > 2.69 && s.position[1] > 0.69);
}
