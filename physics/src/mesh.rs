//! Two-sided finite triangles and conservative capsule sweeps.
//! Geometry is static. Linear scans suit small offline courses, not whole maps.
use super::{add, dot, extent, finite, scale, Character, Input, Plane, Profile, State, Vec3, DT};
fn sub(a: Vec3, b: Vec3) -> Vec3 {
    add(a, scale(b, -1.0))
}
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn length(v: Vec3) -> f64 {
    dot(v, v).sqrt()
}
#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub id: u64,
    pub vertices: [Vec3; 3],
}
#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub fraction: f64,
    pub normal: Vec3,
    pub triangle_id: u64,
}
const SKIN: f64 = 0.0001;
fn validate(triangles: &[Triangle]) -> Result<(), &'static str> {
    for (i, t) in triangles.iter().enumerate() {
        if t.vertices
            .iter()
            .any(|v| !finite(*v) || v.iter().any(|x| x.abs() > 1e6))
            || length(cross(
                sub(t.vertices[1], t.vertices[0]),
                sub(t.vertices[2], t.vertices[0]),
            )) < 1e-10
            || triangles[..i].iter().any(|p| p.id == t.id)
        {
            return Err("invalid or duplicate triangle");
        }
    }
    Ok(())
}
// Closest point in all seven triangle Voronoi regions (face/edges/vertices).
fn point_triangle(p: Vec3, t: Triangle) -> Vec3 {
    let [a, b, c] = t.vertices;
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = sub(p, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        return add(a, scale(ab, d1 / (d1 - d3)));
    }
    let cp = sub(p, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        return add(a, scale(ac, d2 / (d2 - d6)));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && d4 - d3 >= 0.0 && d5 - d6 >= 0.0 {
        return add(b, scale(sub(c, b), (d4 - d3) / ((d4 - d3) + (d5 - d6))));
    }
    let inv = 1.0 / (va + vb + vc);
    add(a, add(scale(ab, vb * inv), scale(ac, vc * inv)))
}
fn segments(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> (Vec3, Vec3) {
    let u = sub(b, a);
    let v = sub(d, c);
    let w = sub(a, c);
    let aa = dot(u, u);
    let bb = dot(u, v);
    let cc = dot(v, v);
    let dd = dot(u, w);
    let ee = dot(v, w);
    let mut s = if aa < 1e-20 {
        0.0
    } else {
        let denom = aa * cc - bb * bb;
        if denom > 1e-20 {
            ((bb * ee - cc * dd) / denom).clamp(0.0, 1.0)
        } else {
            0.0
        }
    };
    let mut q = if cc < 1e-20 { 0.0 } else { (bb * s + ee) / cc };
    if q < 0.0 {
        q = 0.0;
        s = if aa > 1e-20 {
            (-dd / aa).clamp(0.0, 1.0)
        } else {
            0.0
        };
    } else if q > 1.0 {
        q = 1.0;
        s = if aa > 1e-20 {
            ((bb - dd) / aa).clamp(0.0, 1.0)
        } else {
            0.0
        };
    }
    (add(a, scale(u, s)), add(c, scale(v, q)))
}
// Minimum distance from capsule's central segment to a filled triangle.
fn separation(p: Profile, centre: Vec3, t: Triangle) -> (f64, Vec3) {
    let half = p.height / 2.0 - p.radius;
    let a = add(centre, [0.0, 0.0, -half]);
    let b = add(centre, [0.0, 0.0, half]);
    let raw = cross(
        sub(t.vertices[1], t.vertices[0]),
        sub(t.vertices[2], t.vertices[0]),
    );
    let face = scale(raw, 1.0 / length(raw));
    let da = dot(face, sub(a, t.vertices[0]));
    let db = dot(face, sub(b, t.vertices[0]));
    let mut best = (a, point_triangle(a, t));
    let mut consider = |pair: (Vec3, Vec3)| {
        if dot(sub(pair.0, pair.1), sub(pair.0, pair.1))
            < dot(sub(best.0, best.1), sub(best.0, best.1))
        {
            best = pair;
        }
    };
    consider((b, point_triangle(b, t)));
    if (da - db).abs() > 1e-15 {
        let fraction = da / (da - db);
        if (0.0..=1.0).contains(&fraction) {
            let x = add(a, scale(sub(b, a), fraction));
            let q = point_triangle(x, t);
            consider((x, q));
        }
    }
    for i in 0..3 {
        consider(segments(a, b, t.vertices[i], t.vertices[(i + 1) % 3]));
    }
    let delta = sub(best.0, best.1);
    let distance = length(delta);
    let normal = if distance > 1e-12 {
        scale(delta, 1.0 / distance)
    } else {
        scale(
            face,
            if dot(face, sub(centre, t.vertices[0])) < 0.0 {
                -1.0
            } else {
                1.0
            },
        )
    };
    (distance - p.radius, normal)
}
fn sweep(
    p: Profile,
    from: Vec3,
    to: Vec3,
    triangles: &[Triangle],
) -> Result<Option<Hit>, &'static str> {
    let movement = sub(to, from);
    let speed = length(movement);
    let mut earliest: Option<Hit> = None;
    for triangle in triangles {
        let mut fraction = 0.0;
        let mut finished = false;
        // Distance is Lipschitz under translation. Never advance farther than
        // clearance / path length; exhaustion fails closed rather than tunnelling.
        for _ in 0..256 {
            let (gap, normal) = separation(p, add(from, scale(movement, fraction)), *triangle);
            if gap < -SKIN {
                return Err("capsule starts overlapping mesh");
            }
            if gap <= SKIN + 1e-8 {
                if dot(normal, movement) < -1e-10 {
                    let hit = Hit {
                        fraction,
                        normal,
                        triangle_id: triangle.id,
                    };
                    if earliest.is_none_or(|old| fraction < old.fraction) {
                        earliest = Some(hit);
                    }
                }
                finished = true;
                break;
            }
            if speed < 1e-15 || fraction + (gap - SKIN) / speed > 1.0 {
                finished = true;
                break;
            }
            fraction += (gap - SKIN) / speed;
        }
        if !finished {
            return Err("capsule sweep iteration limit");
        }
    }
    Ok(earliest)
}
/// Validated continuous sweep against two-sided triangles, including edges.
/// Skin is 0.1 mm. Starting penetration and iteration exhaustion return errors.
pub fn sweep_mesh(
    p: Profile,
    from: Vec3,
    to: Vec3,
    triangles: &[Triangle],
) -> Result<Option<Hit>, &'static str> {
    Character::new(p, from)?;
    Character::new(p, to)?;
    validate(triangles)?;
    sweep(p, from, to, triangles)
}
/// Move to each earliest hit, then project the remainder along contact surfaces.
/// Corners retain all hit normals. Five blocking contacts per move; fail closed.
pub fn move_and_slide(
    p: Profile,
    from: Vec3,
    to: Vec3,
    triangles: &[Triangle],
) -> Result<Vec3, &'static str> {
    Character::new(p, from)?;
    Character::new(p, to)?;
    validate(triangles)?;
    let mut x = from;
    let mut remaining = sub(to, from);
    let mut normals = Vec::new();
    for _ in 0..5 {
        match sweep(p, x, add(x, remaining), triangles)? {
            None => return Ok(add(x, remaining)),
            Some(hit) => {
                x = add(x, scale(remaining, hit.fraction));
                remaining = scale(remaining, 1.0 - hit.fraction);
                normals.push(hit.normal);
                for _ in 0..5 {
                    for n in &normals {
                        let into = dot(remaining, *n).min(0.0);
                        remaining = sub(remaining, scale(*n, into));
                    }
                }
                if length(remaining) < 1e-9 {
                    return Ok(x);
                }
            }
        }
    }
    Err("slide contact limit")
}
impl Character {
    /// Static mesh adapter around AVBD local tangent contacts, with a swept
    /// movement guard. This guard is kinematic; it is not an AVBD force solve.
    /// Failed queries leave character state unchanged. No stairs/depenetration.
    pub fn step_mesh(
        &mut self,
        input: Input,
        triangles: &[Triangle],
    ) -> Result<State, &'static str> {
        validate(triangles)?;
        let mut trial = self.clone();
        let p = self.profile;
        let old = self.state.position;
        let mut planes = Vec::new();
        for t in triangles {
            let (gap, normal) = separation(p, old, *t);
            if gap < -SKIN {
                return Err("capsule starts overlapping mesh");
            }
            if gap < 0.002 {
                planes.push(Plane {
                    id: t.id,
                    normal,
                    offset: dot(normal, old) - extent(p, normal) - gap,
                });
            }
        }
        let tentative = trial.step(input, &planes)?;
        let x = move_and_slide(p, old, tentative.position, triangles)?;
        trial.state.position = x;
        trial.state.velocity = scale(sub(x, old), 1.0 / DT);
        trial.state.grounded = triangles.iter().any(|t| {
            let (gap, n) = separation(p, x, *t);
            gap.abs() < 0.002
                && n[2] >= p.max_slope_degrees.to_radians().cos()
                && dot(n, trial.state.velocity) <= 0.1
        });
        *self = trial;
        Ok(self.state)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn wall() -> Triangle {
        Triangle {
            id: 1,
            vertices: [[0.0, -2.0, -2.0], [0.0, 2.0, -2.0], [0.0, 0.0, 4.0]],
        }
    }
    #[test]
    fn thin_wall_sliding_and_finite_edges() {
        let p = Profile::default();
        let mesh = [wall()];
        let hit = sweep_mesh(p, [-10.0, 0.0, 1.0], [10.0, 0.0, 1.0], &mesh)
            .unwrap()
            .unwrap();
        assert!((hit.fraction - 0.485).abs() < 1e-4);
        let x = move_and_slide(p, [-1.0, 0.0, 1.0], [1.0, 0.5, 1.0], &mesh).unwrap();
        assert!(x[0] < -p.radius && (x[1] - 0.5).abs() < 1e-6);
        assert!(sweep_mesh(p, [-1.0, 5.0, 1.0], [1.0, 5.0, 1.0], &mesh)
            .unwrap()
            .is_none());
        let t = Triangle {
            id: 2,
            vertices: [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        };
        let (gap, n) = separation(p, [-0.1, -0.1, p.height / 2.0], t);
        assert!(gap > 0.0 && n[0] < 0.0 && n[1] < 0.0); // rounded vertex response
        assert!(sweep_mesh(p, [1.0, 0.0, 1.0], [-1.0, 0.0, 1.0], &mesh)
            .unwrap()
            .is_some());
    }
    #[test]
    fn corners_edges_and_overlap_errors() {
        let p = Profile::default();
        let side = Triangle {
            id: 2,
            vertices: [[-2.0, 0.0, -2.0], [2.0, 0.0, -2.0], [0.0, 0.0, 4.0]],
        };
        let x = move_and_slide(p, [-1.0, -1.0, 1.0], [1.0, 1.0, 1.0], &[wall(), side]).unwrap();
        assert!(x[0] <= -p.radius && x[1] <= -p.radius);
        let edge = Triangle {
            id: 3,
            vertices: [[0.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 3.0]],
        };
        // Centre path misses face, capsule still hits its finite edge.
        assert!(
            sweep_mesh(p, [-2.0, -0.15, 1.0], [2.0, -0.15, 1.0], &[edge])
                .unwrap()
                .is_some()
        );
        assert!(sweep_mesh(p, [0.0, 0.0, 1.0], [1.0, 0.0, 1.0], &[wall()]).is_err());
        assert!(sweep_mesh(p, [f64::NAN, 0.0, 1.0], [1.0, 0.0, 1.0], &[wall()]).is_err());
    }
    #[test]
    fn mesh_floor_and_invalid_state() {
        let p = Profile::default();
        let floor = Triangle {
            id: 1,
            vertices: [[-20.0, -20.0, 0.0], [20.0, -20.0, 0.0], [0.0, 20.0, 0.0]],
        };
        let mut c = Character::new(p, [0.0, 0.0, 3.0]).unwrap();
        for _ in 0..960 {
            c.step_mesh(
                Input {
                    movement: [0.0; 2],
                    jump: false,
                },
                &[floor],
            )
            .unwrap();
        }
        assert!(c.state().grounded && (c.state().position[2] - p.height / 2.0).abs() < 0.002);
        let before = c.state();
        assert!(c
            .step_mesh(
                Input {
                    movement: [0.0; 2],
                    jump: false
                },
                &[floor, floor]
            )
            .is_err());
        assert_eq!(before, c.state());
    }
}
