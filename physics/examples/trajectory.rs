use avatar_physics::{Character, Input, Plane, Profile, DT};
fn main() {
    let p = Profile::default();
    let mut c = Character::new(p, [0.0, 0.0, p.height / 2.0]).unwrap();
    let floor = Plane {
        id: 1,
        normal: [0.0, 0.0, 1.0],
        offset: 0.0,
    };
    println!("time,x,y,z,vx,vy,vz,grounded");
    for i in 0..480 {
        let s = c
            .step(
                Input {
                    movement: [1.0, 0.0],
                    jump: i == 120,
                },
                &[floor],
            )
            .unwrap();
        println!(
            "{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{}",
            (i + 1) as f64 * DT,
            s.position[0],
            s.position[1],
            s.position[2],
            s.velocity[0],
            s.velocity[1],
            s.velocity[2],
            s.grounded
        );
    }
}
