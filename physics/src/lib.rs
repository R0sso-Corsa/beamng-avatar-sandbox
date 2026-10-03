//! Engine-neutral, Z-up, metre/second character prototype.
//! AVBD subset: one translational body against static, frictionless planes.
//! No collision detection, rigid rotation, ragdolls, or host-engine calls.

pub type Vec3 = [f64; 3];
fn dot(a: Vec3, b: Vec3) -> f64 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
fn add(a: Vec3, b: Vec3) -> Vec3 {
    std::array::from_fn(|i| a[i] + b[i])
}
fn scale(a: Vec3, s: f64) -> Vec3 {
    a.map(|v| v * s)
}
fn finite(v: Vec3) -> bool {
    v.iter().all(|v| v.is_finite())
}

/// Prototype settings; acceleration, mass and collision radius are design choices.
#[derive(Clone, Copy, Debug)]
pub struct Profile {
    pub metres_per_stud: f64,
    pub gravity: f64,
    pub walk_speed: f64,
    pub jump_speed: f64,
    pub ground_acceleration: f64,
    pub air_acceleration: f64,
    pub mass: f64,
    pub radius: f64,
    pub max_slope_degrees: f64,
}
impl Default for Profile {
    fn default() -> Self {
        // .30 matches the existing mesh's body scale; .28 is Roblox's standard.
        Self {
            metres_per_stud: 0.30,
            gravity: 196.2 * 0.30,
            walk_speed: 16.0 * 0.30,
            jump_speed: 50.0 * 1.06 * 0.30,
            ground_acceleration: 80.0,
            air_acceleration: 20.0,
            mass: 60.0,
            radius: 0.30,
            max_slope_degrees: 45.0,
        }
    }
}

/// One supporting half-space: dot(normal, centre) >= offset + radius.
/// Supply all relevant planes before stepping. IDs must uniquely and stably
/// identify unchanged world surfaces; replacement geometry gets a new ID.
#[derive(Clone, Copy, Debug)]
pub struct Plane {
    pub id: u64,
    pub normal: Vec3,
    pub offset: f64,
}
#[derive(Clone, Copy, Debug)]
pub struct Input {
    pub movement: [f64; 2],
    pub jump: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct State {
    pub position: Vec3,
    pub velocity: Vec3,
    pub grounded: bool,
}
#[derive(Clone, Debug)]
struct Contact {
    plane: Plane,
    lambda: f64,
    penalty: f64,
    initial_error: f64,
}

/// A fixed 240 Hz step, independent of rendering. Host owns elapsed-time batching.
pub const DT: f64 = 1.0 / 240.0;
const ITERATIONS: usize = 20;
const ALPHA: f64 = 0.95;
const GAMMA: f64 = 0.99;
const START: f64 = 1.0e6;
const BETA: f64 = 1.0e7;
const MAX_PENALTY: f64 = 1.0e10;
const MARGIN: f64 = 0.002;

pub struct Character {
    profile: Profile,
    state: State,
    contacts: Vec<Contact>,
    jump_held: bool,
}
impl Character {
    pub fn new(profile: Profile, position: Vec3) -> Result<Self, &'static str> {
        let positive = [
            profile.metres_per_stud,
            profile.gravity,
            profile.mass,
            profile.radius,
        ];
        let nonnegative = [
            profile.walk_speed,
            profile.jump_speed,
            profile.ground_acceleration,
            profile.air_acceleration,
        ];
        if !finite(position)
            || positive.iter().any(|v| !v.is_finite() || *v <= 0.0)
            || nonnegative.iter().any(|v| !v.is_finite() || *v < 0.0)
            || !profile.max_slope_degrees.is_finite()
            || !(0.0..90.0).contains(&profile.max_slope_degrees)
        {
            return Err("invalid profile or position");
        }
        Ok(Self {
            profile,
            state: State {
                position,
                velocity: [0.0; 3],
                grounded: false,
            },
            contacts: Vec::new(),
            jump_held: false,
        })
    }
    pub fn state(&self) -> State {
        self.state
    }
    /// Validate before mutation. Normal must be unit length; duplicate IDs rejected.
    pub fn step(&mut self, input: Input, planes: &[Plane]) -> Result<State, &'static str> {
        if input.movement.iter().any(|v| !v.is_finite()) {
            return Err("invalid input");
        }
        for (i, p) in planes.iter().enumerate() {
            if !finite(p.normal)
                || !p.offset.is_finite()
                || (dot(p.normal, p.normal) - 1.0).abs() > 1e-6
                || planes[..i].iter().any(|q| q.id == p.id)
            {
                return Err("invalid planes");
            }
        }
        let p = self.profile;
        let old = self.state.position;
        let mut velocity = self.state.velocity;
        let slope = p.max_slope_degrees.to_radians().cos();
        let supported = planes.iter().any(|plane| {
            plane.normal[2] >= slope
                && (dot(plane.normal, old) - plane.offset - p.radius).abs() <= MARGIN
                && dot(plane.normal, velocity) <= 0.1
        });
        let jump = input.jump && !self.jump_held && supported;
        self.jump_held = input.jump;
        if jump {
            velocity[2] = p.jump_speed;
        }
        // Separate gameplay motor; these acceleration limits are not AVBD or
        // reverse-engineered Humanoid constants. Analog magnitude is preserved.
        let length = input.movement[0].hypot(input.movement[1]).max(1.0);
        let target = input.movement.map(|v| v / length * p.walk_speed);
        let delta = [target[0] - velocity[0], target[1] - velocity[1]];
        let d = delta[0].hypot(delta[1]);
        let limit = if supported && !jump {
            p.ground_acceleration
        } else {
            p.air_acceleration
        } * DT;
        if d > 0.0 {
            for i in 0..2 {
                velocity[i] += delta[i] * (limit / d).min(1.0);
            }
        }
        // Inertial target (Eq. 2); initial guess uses that target, rather than
        // the reference implementation's adaptive initialization.
        let target = add(
            add(old, scale(velocity, DT)),
            [0.0, 0.0, -p.gravity * DT * DT],
        );
        let mut x = target;
        let previous = std::mem::take(&mut self.contacts);
        // ponytail: linear scan for one avatar; spatial queries belong to adapter.
        for plane in planes {
            let c0 = dot(plane.normal, old) - plane.offset - p.radius;
            let predicted = dot(plane.normal, target) - plane.offset - p.radius;
            if c0.min(predicted) > MARGIN {
                continue;
            }
            let cached = previous.iter().find(|c| {
                c.plane.id == plane.id
                    && c.plane.normal == plane.normal
                    && c.plane.offset == plane.offset
            });
            self.contacts.push(Contact {
                plane: *plane,
                initial_error: c0.min(0.0),
                lambda: cached.map_or(0.0, |c| c.lambda * ALPHA * GAMMA),
                penalty: cached.map_or(START, |c| (c.penalty * GAMMA).max(START)),
            });
        }
        let inertia = p.mass / (DT * DT);
        for _ in 0..ITERATIONS {
            let mut h = [[0.0; 3]; 3];
            for (i, row) in h.iter_mut().enumerate() {
                row[i] = inertia;
            }
            let mut gradient = scale(add(x, scale(target, -1.0)), inertia);
            for c in &self.contacts {
                let error =
                    dot(c.plane.normal, x) - c.plane.offset - p.radius - ALPHA * c.initial_error;
                // Outward normal convention -> negative dual, repulsive force.
                let raw = c.lambda + c.penalty * error;
                let force = raw.min(0.0);
                // Eq. 14: Hessian-only rescaling at force bound (zero).
                let k = if raw > 0.0 && error != 0.0 {
                    (c.lambda / error).abs()
                } else {
                    c.penalty
                };
                gradient = add(gradient, scale(c.plane.normal, force));
                for (i, row) in h.iter_mut().enumerate() {
                    for (j, v) in row.iter_mut().enumerate() {
                        *v += k * c.plane.normal[i] * c.plane.normal[j];
                    }
                }
                // Linear plane C has exactly zero geometric Hessian (Sec. 3.5).
            }
            x = add(x, solve_spd(h, scale(gradient, -1.0)));
            // Dual update after the body block (Eqs. 11-12, bounded Sec. 3.2).
            for c in &mut self.contacts {
                let error =
                    dot(c.plane.normal, x) - c.plane.offset - p.radius - ALPHA * c.initial_error;
                let raw = c.lambda + c.penalty * error;
                c.lambda = raw.min(0.0);
                if raw < 0.0 {
                    c.penalty = (c.penalty + BETA * error.abs()).min(MAX_PENALTY);
                }
            }
        }
        self.state = State {
            position: x,
            velocity: scale(add(x, scale(old, -1.0)), 1.0 / DT),
            grounded: !jump
                && self.contacts.iter().any(|c| {
                    c.lambda < 0.0
                        && c.plane.normal[2] >= slope
                        && (dot(c.plane.normal, x) - c.plane.offset - p.radius).abs() <= MARGIN
                }),
        };
        Ok(self.state)
    }
}

// Tiny positive-definite LDL^T solve: no matrix dependency required.
fn solve_spd(a: [[f64; 3]; 3], b: Vec3) -> Vec3 {
    let mut l = [[0.0; 3]; 3];
    let mut d = [0.0; 3];
    for i in 0..3 {
        l[i][i] = 1.0;
        d[i] = a[i][i] - (0..i).map(|k| l[i][k] * l[i][k] * d[k]).sum::<f64>();
        for j in i + 1..3 {
            l[j][i] = (a[j][i] - (0..i).map(|k| l[j][k] * l[i][k] * d[k]).sum::<f64>()) / d[i];
        }
    }
    let mut y = [0.0; 3];
    for i in 0..3 {
        y[i] = b[i] - (0..i).map(|k| l[i][k] * y[k]).sum::<f64>();
    }
    let mut x = [0.0; 3];
    for i in (0..3).rev() {
        x[i] = y[i] / d[i] - (i + 1..3).map(|k| l[k][i] * x[k]).sum::<f64>();
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;
    const FLOOR: Plane = Plane {
        id: 1,
        normal: [0.0, 0.0, 1.0],
        offset: 0.0,
    };
    const IDLE: Input = Input {
        movement: [0.0, 0.0],
        jump: false,
    };
    #[test]
    fn fall_land_walk_wall_and_jump() {
        let p = Profile::default();
        let mut c = Character::new(p, [0.0, 0.0, 3.0]).unwrap();
        let wall = Plane {
            id: 2,
            normal: [-1.0, 0.0, 0.0],
            offset: -2.0,
        };
        for _ in 0..1200 {
            c.step(IDLE, &[FLOOR, wall]).unwrap();
        }
        assert!((c.state.position[2] - p.radius).abs() < 1e-4);
        assert!(c.state.grounded);
        for _ in 0..480 {
            c.step(
                Input {
                    movement: [1.0, 0.0],
                    jump: false,
                },
                &[FLOOR, wall],
            )
            .unwrap();
        }
        assert!(c.state.position[0] <= 2.0 - p.radius + MARGIN);
        assert!(c.state.position[0] > 1.69);
        let start = c.state.position[2];
        let mut peak = start;
        for _ in 0..240 {
            let s = c
                .step(
                    Input {
                        movement: [0.0; 2],
                        jump: true,
                    },
                    &[FLOOR, wall],
                )
                .unwrap();
            peak = peak.max(s.position[2]);
        }
        let expected = p.jump_speed * p.jump_speed / (2.0 * p.gravity);
        assert!(((peak - start) - expected).abs() < 0.04);
        assert!(c.state.grounded); // holding jump must not auto-repeat
    }
    #[test]
    fn free_fall_and_diagonal_speed() {
        let p = Profile::default();
        let mut c = Character::new(p, [0.0, 0.0, 100.0]).unwrap();
        for _ in 0..240 {
            c.step(
                Input {
                    movement: [1.0, 1.0],
                    jump: false,
                },
                &[],
            )
            .unwrap();
        }
        assert!((c.state.velocity[2] + p.gravity).abs() < 1e-7);
        assert!((c.state.velocity[0].hypot(c.state.velocity[1]) - p.walk_speed).abs() < 1e-7);
        assert!(!c.state.grounded);
    }
    #[test]
    fn invalid_input_does_not_mutate() {
        let mut c = Character::new(Profile::default(), [0.0; 3]).unwrap();
        let before = c.state();
        assert!(c
            .step(
                Input {
                    movement: [f64::NAN, 0.0],
                    jump: false
                },
                &[]
            )
            .is_err());
        assert!(c.step(IDLE, &[FLOOR, FLOOR]).is_err());
        assert_eq!(c.state(), before);
    }
}
