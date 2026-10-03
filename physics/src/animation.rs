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
        a.pose.blend(b.pose, (time - a.time) / (b.time - a.time))
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
