//! Portable local-joint pose sampling. Host supplies converted animation tracks.
#[derive(Clone, Copy, Debug)]
pub struct Pose {
    pub translation: [f64; 3],
    /// Quaternion x, y, z, w.
    pub rotation: [f64; 4],
}
impl Pose {
    fn valid(self) -> bool {
        self.translation
            .iter()
            .chain(self.rotation.iter())
            .all(|v| v.is_finite())
            && (self.rotation.iter().map(|v| v * v).sum::<f64>() - 1.0).abs() < 1e-6
    }
    pub fn blend(self, other: Self, amount: f64) -> Result<Self, &'static str> {
        if !self.valid() || !other.valid() || !amount.is_finite() || !(0.0..=1.0).contains(&amount)
        {
            return Err("invalid pose blend");
        }
        let mut q = other.rotation;
        let mut dot = self.rotation.iter().zip(q).map(|(a, b)| a * b).sum::<f64>();
        if dot < 0.0 {
            q = q.map(|v| -v);
            dot = -dot;
        }
        let (a, b) = if dot > 0.9995 {
            (1.0 - amount, amount)
        } else {
            let angle = dot.clamp(-1.0, 1.0).acos();
            (
                ((1.0 - amount) * angle).sin() / angle.sin(),
                (amount * angle).sin() / angle.sin(),
            )
        };
        let mut rotation = std::array::from_fn(|i| a * self.rotation[i] + b * q[i]);
        let length = rotation.iter().map(|v| v * v).sum::<f64>().sqrt();
        rotation = rotation.map(|v| v / length);
        Ok(Self {
            translation: std::array::from_fn(|i| {
                self.translation[i] * (1.0 - amount) + other.translation[i] * amount
            }),
            rotation,
        })
    }
}
#[derive(Clone, Copy)]
pub enum Interpolation {
    /// Absolute principal rotation-vector interpolation observed in R6 Animator.
    RobloxLinear,
    Linear,
    Hold,
}
#[derive(Clone, Copy)]
pub struct Key {
    pub time: f64,
    pub pose: Pose,
    pub interpolation: Interpolation,
}
pub struct Track {
    keys: Vec<Key>,
}
impl Track {
    pub fn new(keys: Vec<Key>) -> Result<Self, &'static str> {
        if keys.is_empty()
            || keys.len() > 100000
            || keys
                .iter()
                .any(|k| !k.time.is_finite() || k.time < 0.0 || !k.pose.valid())
            || keys.windows(2).any(|w| w[0].time >= w[1].time)
        {
            return Err("invalid animation track");
        }
        Ok(Self { keys })
    }
    /// Loop duration must cover this track. Nonlooping samples clamp to endpoints.
    pub fn sample(&self, time: f64, loop_duration: Option<f64>) -> Result<Pose, &'static str> {
        if !time.is_finite() || time < 0.0 {
            return Err("invalid animation time");
        }
        let time = match loop_duration {
            Some(d) if d.is_finite() && d > 0.0 && d >= self.keys.last().unwrap().time => {
                time.rem_euclid(d)
            }
            Some(_) => return Err("invalid loop duration"),
            None => time,
        };
        let upper = self.keys.partition_point(|k| k.time <= time);
        if upper == 0 {
            return Ok(self.keys[0].pose);
        }
        let a = self.keys[upper - 1];
        if upper == self.keys.len() || matches!(a.interpolation, Interpolation::Hold) {
            return Ok(a.pose);
        }
        let b = self.keys[upper];
        let amount = (time - a.time) / (b.time - a.time);
        if matches!(a.interpolation, Interpolation::RobloxLinear) {
            a.pose.roblox_blend(b.pose, amount)
        } else {
            a.pose.blend(b.pose, amount)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sampling_rotation_loop_and_rejection() {
        let a = Pose {
            translation: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
        };
        let b = Pose {
            translation: [2.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 1.0, 0.0],
        };
        let track = Track::new(vec![
            Key {
                time: 0.0,
                pose: a,
                interpolation: Interpolation::Linear,
            },
            Key {
                time: 1.0,
                pose: b,
                interpolation: Interpolation::Linear,
            },
        ])
        .unwrap();
        let mid = track.sample(0.5, None).unwrap();
        assert_eq!(mid.translation[0], 1.0);
        assert!((mid.rotation[2] - 0.5_f64.sqrt()).abs() < 1e-10);
        assert_eq!(track.sample(1.0, Some(1.0)).unwrap().translation[0], 0.0);
        assert_eq!(track.sample(2.0, None).unwrap().translation[0], 2.0);
        assert!(track.sample(f64::NAN, None).is_err());
        assert!(track.sample(0.0, Some(0.5)).is_err());
        assert!(Track::new(vec![]).is_err());
        let antipodal = Pose {
            rotation: a.rotation.map(|v| -v),
            ..a
        };
        assert_eq!(a.blend(antipodal, 0.5).unwrap().rotation, a.rotation);
        let hold = Track::new(vec![
            Key {
                time: 0.0,
                pose: a,
                interpolation: Interpolation::Hold,
            },
            Key {
                time: 1.0,
                pose: b,
                interpolation: Interpolation::Linear,
            },
        ])
        .unwrap();
        assert_eq!(hold.sample(0.5, None).unwrap().translation, a.translation);
    }
}

fn multiply(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    let [x, y, z, w] = a;
    let [i, j, k, l] = b;
    [
        w * i + x * l + y * k - z * j,
        w * j - x * k + y * l + z * i,
        w * k + x * j - y * i + z * l,
        w * l - x * i - y * j - z * k,
    ]
}
fn rotate(q: [f64; 4], v: [f64; 3]) -> [f64; 3] {
    let r = multiply(
        multiply(q, [v[0], v[1], v[2], 0.0]),
        [-q[0], -q[1], -q[2], q[3]],
    );
    [r[0], r[1], r[2]]
}
impl Pose {
    pub const IDENTITY: Self = Self {
        translation: [0.0; 3],
        rotation: [0.0, 0.0, 0.0, 1.0],
    };
    /// Rigid transform multiplication, parent * child (no scale/shear).
    pub fn compose(self, child: Self) -> Result<Self, &'static str> {
        if !self.valid() || !child.valid() {
            return Err("invalid rigid transform");
        }
        let offset = rotate(self.rotation, child.translation);
        let mut rotation = multiply(self.rotation, child.rotation);
        let norm = rotation.iter().map(|v| v * v).sum::<f64>().sqrt();
        rotation = rotation.map(|v| v / norm);
        let result = Self {
            translation: std::array::from_fn(|i| self.translation[i] + offset[i]),
            rotation,
        };
        if !result.valid() {
            return Err("rigid transform overflow");
        }
        Ok(result)
    }
    pub fn inverse(self) -> Result<Self, &'static str> {
        if !self.valid() {
            return Err("invalid rigid transform");
        }
        let rotation = [
            -self.rotation[0],
            -self.rotation[1],
            -self.rotation[2],
            self.rotation[3],
        ];
        Ok(Self {
            translation: rotate(rotation, self.translation.map(|v| -v)),
            rotation,
        })
    }
}
/// Applies a Roblox Motor6D pose to a target bone's local rest transform.
/// C1/pose translation must already share target units. Alignment maps source
/// child-part axes into target bone axes; it is a pure rotation, supplied by host.
/// Preserves target bind pose for identity input; does not guess source offsets.
pub fn retarget(
    target_bind: Pose,
    source_c1: Pose,
    alignment: Pose,
    delta: Pose,
) -> Result<Pose, &'static str> {
    if alignment.translation != [0.0; 3] {
        return Err("alignment must be a pure rotation");
    }
    let local_delta = source_c1.compose(delta)?.compose(source_c1.inverse()?)?;
    target_bind
        .compose(alignment)?
        .compose(local_delta)?
        .compose(alignment.inverse()?)
}
#[cfg(test)]
mod retarget_tests {
    use super::*;
    fn close(a: Pose, b: Pose) {
        for (x, y) in a.translation.iter().zip(b.translation) {
            assert!((x - y).abs() < 1e-9);
        }
        let dot = a
            .rotation
            .iter()
            .zip(b.rotation)
            .map(|(x, y)| x * y)
            .sum::<f64>();
        assert!((dot.abs() - 1.0).abs() < 1e-9);
    }
    #[test]
    fn bind_preservation_inverse_and_joint_pivot() {
        let bind = Pose {
            translation: [1.0, 2.0, 3.0],
            rotation: [0.0, 0.0, 0.5_f64.sqrt(), 0.5_f64.sqrt()],
        };
        close(
            bind.compose(bind.inverse().unwrap()).unwrap(),
            Pose::IDENTITY,
        );
        let c1 = Pose {
            translation: [1.0, 0.0, 0.0],
            ..Pose::IDENTITY
        };
        close(
            retarget(bind, c1, bind_with_rotation(bind), Pose::IDENTITY).unwrap(),
            bind,
        );
        let halfturn = Pose {
            rotation: [0.0, 0.0, 1.0, 0.0],
            ..Pose::IDENTITY
        };
        let result = retarget(Pose::IDENTITY, c1, Pose::IDENTITY, halfturn).unwrap();
        close(
            result,
            Pose {
                translation: [2.0, 0.0, 0.0],
                ..halfturn
            },
        );
        assert!(retarget(bind, c1, bind, halfturn).is_err());
    }
    fn bind_with_rotation(p: Pose) -> Pose {
        Pose {
            translation: [0.0; 3],
            ..p
        }
    }
}

impl Pose {
    /// Principal axis-angle vector lerp. Keep separate from generic slerp blending.
    /// Captured R6 clips validate this choice; not a universal Roblox engine claim.
    pub fn roblox_blend(self, other: Self, amount: f64) -> Result<Self, &'static str> {
        // Reuse validation and translation interpolation; replace rotation only.
        let mut result = self.blend(other, amount)?;
        fn vector(mut q: [f64; 4]) -> [f64; 3] {
            if q[3] < 0.0 {
                q = q.map(|v| -v);
            }
            let length = q[..3].iter().map(|v| v * v).sum::<f64>().sqrt();
            if length < 1e-12 {
                return [0.0; 3];
            }
            let angle = 2.0 * length.atan2(q[3]);
            std::array::from_fn(|i| q[i] * angle / length)
        }
        let a = vector(self.rotation);
        let b = vector(other.rotation);
        let v: [f64; 3] = std::array::from_fn(|i| a[i] * (1.0 - amount) + b[i] * amount);
        let angle = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        result.rotation = if angle < 1e-12 {
            [0.0, 0.0, 0.0, 1.0]
        } else {
            [
                v[0] * (angle / 2.0).sin() / angle,
                v[1] * (angle / 2.0).sin() / angle,
                v[2] * (angle / 2.0).sin() / angle,
                (angle / 2.0).cos(),
            ]
        };
        Ok(result)
    }
}

#[cfg(test)]
mod roblox_rotation_tests {
    use super::*;
    #[test]
    fn principal_vector_endpoints_and_antipodal_identity() {
        let a = Pose::IDENTITY;
        let b = Pose {
            rotation: [0.0, 0.0, 1.0, 0.0],
            ..a
        };
        let mid = a.roblox_blend(b, 0.5).unwrap();
        assert!((mid.rotation[2] - 0.5_f64.sqrt()).abs() < 1e-10);
        assert_eq!(a.roblox_blend(a, 0.2).unwrap().rotation, a.rotation);
        assert_eq!(
            a.roblox_blend(
                Pose {
                    rotation: [0.0, 0.0, 0.0, -1.0],
                    ..a
                },
                0.2
            )
            .unwrap()
            .rotation,
            a.rotation
        );
        assert!(a.roblox_blend(b, f64::NAN).is_err());
    }
}
