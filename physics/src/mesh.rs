//! Two-sided finite triangles and conservative capsule sweeps.
//! StaticMesh caches an AABB tree; raw slice queries retain linear scans.
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
// Cached bounding-volume tree for immutable world geometry.
#[derive(Clone, Copy)]
struct Bounds {
    lo: Vec3,
    hi: Vec3,
}
impl Bounds {
    fn triangle(t: Triangle) -> Self {
        Self {
            lo: std::array::from_fn(|i| {
                t.vertices
                    .iter()
                    .map(|v| v[i])
                    .fold(f64::INFINITY, f64::min)
            }),
            hi: std::array::from_fn(|i| {
                t.vertices
                    .iter()
                    .map(|v| v[i])
                    .fold(f64::NEG_INFINITY, f64::max)
            }),
        }
    }
    fn union(self, b: Self) -> Self {
        Self {
            lo: std::array::from_fn(|i| self.lo[i].min(b.lo[i])),
            hi: std::array::from_fn(|i| self.hi[i].max(b.hi[i])),
        }
    }
    fn intersects(self, b: Self) -> bool {
        (0..3).all(|i| self.lo[i] <= b.hi[i] && self.hi[i] >= b.lo[i])
    }
}
struct Node {
    bounds: Bounds,
    indices: Vec<usize>,
    children: Option<[Box<Node>; 2]>,
}
impl Node {
    fn build(mut indices: Vec<usize>, bounds: &[Bounds]) -> Self {
        let total = indices
            .iter()
            .map(|i| bounds[*i])
            .reduce(Bounds::union)
            .unwrap();
        if indices.len() <= 8 {
            return Self {
                bounds: total,
                indices,
                children: None,
            };
        }
        let axis = (0..3)
            .max_by(|a, b| (total.hi[*a] - total.lo[*a]).total_cmp(&(total.hi[*b] - total.lo[*b])))
            .unwrap();
        indices.sort_unstable_by(|a, b| {
            (bounds[*a].lo[axis] + bounds[*a].hi[axis])
                .total_cmp(&(bounds[*b].lo[axis] + bounds[*b].hi[axis]))
        });
        let right = indices.split_off(indices.len() / 2);
        Self {
            bounds: total,
            indices: Vec::new(),
            children: Some([
                Box::new(Self::build(indices, bounds)),
                Box::new(Self::build(right, bounds)),
            ]),
        }
    }
    fn query(&self, query: Bounds, bounds: &[Bounds], out: &mut Vec<usize>) {
        if !self.bounds.intersects(query) {
            return;
        }
        if let Some(children) = &self.children {
            for child in children {
                child.query(query, bounds, out);
            }
        } else {
            out.extend(
                self.indices
                    .iter()
                    .filter(|i| bounds[**i].intersects(query))
                    .copied(),
            );
        }
    }
}
/// Immutable static mesh, validated once and indexed by a median-split AABB tree.
/// Rebuild when geometry changes. IDs must remain unique across the mesh.
pub struct StaticMesh {
    triangles: Vec<Triangle>,
    bounds: Vec<Bounds>,
    root: Option<Node>,
}
impl StaticMesh {
    pub fn new(triangles: Vec<Triangle>) -> Result<Self, &'static str> {
        validate(&triangles)?;
        let bounds: Vec<_> = triangles.iter().map(|t| Bounds::triangle(*t)).collect();
        let root = if triangles.is_empty() {
            None
        } else {
            Some(Node::build((0..triangles.len()).collect(), &bounds))
        };
        Ok(Self {
            triangles,
            bounds,
            root,
        })
    }
    fn candidates(&self, query: Bounds) -> Vec<Triangle> {
        let mut indices = Vec::new();
        if let Some(root) = &self.root {
            root.query(query, &self.bounds, &mut indices);
        }
        // Preserve source order: changing broadphase must not change tie-breaking.
        indices.sort_unstable();
        indices.into_iter().map(|i| self.triangles[i]).collect()
    }
}
fn walkable(t: Triangle, normal: Vec3, slope: f64) -> bool {
    let face = cross(
        sub(t.vertices[1], t.vertices[0]),
        sub(t.vertices[2], t.vertices[0]),
    );
    normal[2] > 0.0 && face[2].abs() / length(face) >= slope
}
const SKIN: f64 = 0.0001;
fn validate(triangles: &[Triangle]) -> Result<(), &'static str> {
    let mut ids = std::collections::HashSet::new();
    for t in triangles {
        if t.vertices
            .iter()
            .any(|v| !finite(*v) || v.iter().any(|x| x.abs() > 1e6))
            || length(cross(
                sub(t.vertices[1], t.vertices[0]),
                sub(t.vertices[2], t.vertices[0]),
            )) < 1e-10
            || !ids.insert(t.id)
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
        // Distance to the convex Minkowski difference has a supporting tangent.
        // Its projected closing rate bounds safe advance; exhaustion fails closed.
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
            // Distance to this convex triangle/capsule pair along a line is
            // convex. Once its derivative is nonnegative, it cannot approach
            // later on this segment; skip tangential near-skin stalls.
            let closing = -dot(normal, movement);
            if closing <= 1e-12 * speed || speed < 1e-15 || fraction + (gap - SKIN) / closing > 1.0
            {
                finished = true;
                break;
            }
            fraction += (gap - SKIN) / closing;
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
    /// Query a cached mesh using a conservative envelope including motor, jump,
    /// gravity, stair probes and ground snap. Narrow phase sees only candidates.
    pub fn step_static_mesh(
        &mut self,
        input: Input,
        mesh: &StaticMesh,
    ) -> Result<State, &'static str> {
        let p = self.profile;
        let pos = self.state.position;
        let travel: Vec3 = std::array::from_fn(|i| {
            self.state.velocity[i].abs() * DT
                + p.walk_speed * DT
                + p.jump_speed * DT
                + p.gravity * DT * DT
                + p.step_height
                + 0.04
        });
        let radii = [p.radius, p.radius, p.height / 2.0];
        let query = Bounds {
            lo: std::array::from_fn(|i| pos[i] - radii[i] - travel[i]),
            hi: std::array::from_fn(|i| pos[i] + radii[i] + travel[i]),
        };
        self.step_mesh(input, &mesh.candidates(query))
    }
    /// Static mesh adapter around AVBD local tangent contacts, with a swept
    /// movement guard. This guard is kinematic; it is not an AVBD force solve.
    /// Failed queries leave character state unchanged. No depenetration.
    pub fn step_mesh(
        &mut self,
        input: Input,
        triangles: &[Triangle],
    ) -> Result<State, &'static str> {
        validate(triangles)?;
        let mut trial = self.clone();
        let p = self.profile;
        let old = self.state.position;
        let slope = p.max_slope_degrees.to_radians().cos();
        let mut planes = Vec::new();
        let mut mesh_supported = false;
        for t in triangles {
            let (gap, normal) = separation(p, old, *t);
            if gap < -SKIN {
                return Err("capsule starts overlapping mesh");
            }
            if gap < 0.002 && walkable(*t, normal, slope) {
                mesh_supported |= dot(normal, self.state.velocity) <= 0.1;
                if normal[2] >= slope {
                    planes.push(Plane {
                        id: t.id,
                        normal,
                        offset: dot(normal, old) - extent(p, normal) - gap,
                    });
                }
            }
        }
        let tentative = trial.step(input, &planes)?;
        let mut x = move_and_slide(p, old, tentative.position, triangles)?;
        let supported = planes
            .iter()
            .any(|plane| dot(plane.normal, self.state.velocity) <= 0.1);
        let horizontal = sub(tentative.position, old);
        let wanted = horizontal[0].hypot(horizontal[1]);
        let progress = (x[0] - old[0]).hypot(x[1] - old[1]);
        let walking = (supported || mesh_supported) && !input.jump;
        let mut stepped = false;
        let blocked_steep = matches!(sweep(p,old,tentative.position,triangles),Ok(Some(hit)) if hit.normal[2]<slope);
        if walking
            && blocked_steep
            && p.step_height > 0.0
            && wanted > 1e-6
            && progress + 1e-6 < wanted
        {
            // Up, forward, then down. Every leg is swept, so a low ceiling
            // or wall cannot be bypassed. A failed optional probe keeps the
            // ordinary slide result. Stepping is a gameplay rule, not AVBD.
            let raised = add(old, [0.0, 0.0, p.step_height]);
            if matches!(sweep(p, old, raised, triangles), Ok(None)) {
                let forward = add(raised, [horizontal[0], horizontal[1], 0.0]);
                if let Ok(across) = move_and_slide(p, raised, forward, triangles) {
                    let down = add(across, [0.0, 0.0, -p.step_height - 0.002]);
                    if let Ok(Some(hit)) = sweep(p, across, down, triangles) {
                        let landed = add(across, scale(sub(down, across), hit.fraction));
                        let rise = landed[2] - old[2];
                        let advance = (landed[0] - old[0]).hypot(landed[1] - old[1]);
                        if triangles.iter().any(|t| {
                            let (gap, n) = separation(p, landed, *t);
                            gap.abs() < 0.002 && walkable(*t, n, slope)
                        }) && rise > 1e-6
                            && rise <= p.step_height + SKIN
                            && advance > progress + 1e-6
                        {
                            x = landed;
                            stepped = true;
                        }
                    }
                }
            }
        }
        // Short support probe bridges numerical separation and descending
        // slopes, but deliberately does not snap across stair-sized drops.
        if walking && !stepped {
            let down = add(x, [0.0, 0.0, -0.03]);
            if let Ok(Some(hit)) = sweep(p, x, down, triangles) {
                if hit.normal[2] >= slope {
                    x = add(x, scale(sub(down, x), hit.fraction));
                }
            }
        }
        trial.state.position = x;
        trial.state.velocity = scale(sub(x, old), 1.0 / DT);
        if stepped {
            trial.state.velocity[2] = 0.0;
        }
        trial.state.grounded = triangles.iter().any(|t| {
            let (gap, n) = separation(p, x, *t);
            gap.abs() < 0.002 && walkable(*t, n, slope) && dot(n, trial.state.velocity) <= 0.1
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
    fn quad(mesh: &mut Vec<Triangle>, a: Vec3, b: Vec3, c: Vec3, d: Vec3) {
        for vertices in [[a, b, c], [a, c, d]] {
            mesh.push(Triangle {
                id: mesh.len() as u64,
                vertices,
            });
        }
    }
    fn stair(height: f64, ceiling: Option<f64>) -> Vec<Triangle> {
        let mut mesh = Vec::new();
        quad(
            &mut mesh,
            [-5.0, -3.0, 0.0],
            [8.0, -3.0, 0.0],
            [8.0, 3.0, 0.0],
            [-5.0, 3.0, 0.0],
        );
        quad(
            &mut mesh,
            [1.0, -3.0, 0.0],
            [1.0, 3.0, 0.0],
            [1.0, 3.0, height],
            [1.0, -3.0, height],
        );
        quad(
            &mut mesh,
            [1.0, -3.0, height],
            [8.0, -3.0, height],
            [8.0, 3.0, height],
            [1.0, 3.0, height],
        );
        if let Some(z) = ceiling {
            quad(
                &mut mesh,
                [-5.0, -3.0, z],
                [8.0, -3.0, z],
                [8.0, 3.0, z],
                [-5.0, 3.0, z],
            );
        }
        mesh
    }
    #[test]
    fn stair_limit_and_low_ceiling() {
        let p = Profile::default();
        for (height, ceiling, climbs) in [
            (0.20, None, true),
            (0.65, None, false),
            (0.20, Some(1.65), false),
        ] {
            let mesh = StaticMesh::new(stair(height, ceiling)).unwrap();
            let mut c = Character::new(p, [0.0, 0.0, p.height / 2.0 + SKIN]).unwrap();
            for _ in 0..180 {
                c.step_static_mesh(
                    Input {
                        movement: [1.0, 0.0],
                        jump: false,
                    },
                    &mesh,
                )
                .unwrap();
            }
            if climbs {
                assert!(
                    c.state.position[0] > 2.0
                        && (c.state.position[2] - p.height / 2.0 - height).abs() < 0.003
                );
            } else {
                assert!(c.state.position[0] < 1.05);
            }
        }
    }
    #[test]
    fn walkable_ramp_seams_and_steep_support() {
        let p = Profile::default();
        let angle = 20.0_f64.to_radians();
        let slope = angle.tan();
        let mut mesh = Vec::new();
        for i in 0..2 {
            let x = i as f64 * 2.0;
            let z = x * slope;
            quad(
                &mut mesh,
                [x, -3.0, z],
                [x + 2.0, -3.0, z + 2.0 * slope],
                [x + 2.0, 3.0, z + 2.0 * slope],
                [x, 3.0, z],
            );
        }
        let mut c = Character::new(
            p,
            [
                0.5,
                0.0,
                0.5 * slope + p.height / 2.0 - p.radius + p.radius / angle.cos() + SKIN,
            ],
        )
        .unwrap();
        for _ in 0..150 {
            let s = c
                .step_mesh(
                    Input {
                        movement: [1.0, 0.0],
                        jump: false,
                    },
                    &mesh,
                )
                .unwrap();
            assert!(s.grounded);
        }
        assert!(
            c.state.position[0] > 2.0 && c.state.position[2] > 1.5,
            "{:?}",
            c.state
        );
        let steep = 60.0_f64.to_radians();
        let m = steep.tan();
        let t = Triangle {
            id: 90,
            vertices: [
                [-5.0, -10.0, -5.0 * m],
                [5.0, -10.0, 5.0 * m],
                [0.0, 10.0, 0.0],
            ],
        };
        let mut c = Character::new(
            p,
            [
                0.0,
                0.0,
                p.height / 2.0 - p.radius + p.radius / steep.cos() + SKIN,
            ],
        )
        .unwrap();
        assert!(
            !c.step_mesh(
                Input {
                    movement: [0.0; 2],
                    jump: true
                },
                &[t]
            )
            .unwrap()
            .grounded
        );
        assert!(c.state.velocity[2] < p.jump_speed / 2.0);
    }
    #[test]
    fn indexed_candidates_match_brute_force() {
        let mut triangles = Vec::new();
        for i in 0..1000 {
            let x = i as f64 * 10.0;
            triangles.push(Triangle {
                id: i,
                vertices: [[x - 2.0, -2.0, 0.0], [x + 2.0, -2.0, 0.0], [x, 2.0, 0.0]],
            });
        }
        let mesh = StaticMesh::new(triangles.clone()).unwrap();
        let query = Bounds {
            lo: [-1.0, -1.0, -1.0],
            hi: [1.0, 1.0, 1.0],
        };
        let selected = mesh.candidates(query);
        let expected: Vec<_> = triangles
            .iter()
            .filter(|t| Bounds::triangle(**t).intersects(query))
            .map(|t| t.id)
            .collect();
        assert_eq!(selected.iter().map(|t| t.id).collect::<Vec<_>>(), expected);
        assert_eq!(selected.len(), 1);
        let p = Profile::default();
        let mut a = Character::new(p, [0.0, 0.0, p.height / 2.0 + SKIN]).unwrap();
        let mut b = a.clone();
        for _ in 0..20 {
            let input = Input {
                movement: [0.0; 2],
                jump: false,
            };
            assert_eq!(
                a.step_static_mesh(input, &mesh).unwrap(),
                b.step_mesh(input, &triangles).unwrap()
            );
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
