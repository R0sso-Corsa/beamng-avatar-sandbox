//! Engine-neutral Classic-inspired orbit camera; host applies pose and collision.
use crate::{finite, Vec3};
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraPose {
    pub position: Vec3,
    pub forward: Vec3,
    pub first_person: u32,
    pub heading: f64,
    pub requested_distance: f64,
    pub actual_distance: f64,
}
#[derive(Clone)]
pub struct Camera {
    yaw: f64,
    pitch: f64,
    distance: f64,
    first: bool,
    active: bool,
    classic: Option<ClassicZoom>,
}
#[derive(Clone)]
struct ClassicZoom {
    scale: f64,
    position: f64,
    velocity: f64,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            distance: 3.0,
            first: false,
            active: false,
            classic: None,
        }
    }
}
impl Camera {
    /// Opt-in measured R6 camera preset. Existing v1 callers keep instant zoom.
    pub fn enable_classic(&mut self, scale: f64) -> Result<(), &'static str> {
        if !scale.is_finite() || scale <= 0.0 || !(50.0 * scale).is_finite() {
            return Err("invalid camera scale");
        }
        self.distance = 12.5 * scale;
        self.pitch = -15_f64.to_radians();
        self.first = false;
        self.classic = Some(ClassicZoom {
            scale,
            position: self.distance,
            velocity: 0.0,
        });
        Ok(())
    }
    /// Absolute zoom target in metres. The Classic threshold is one stud.
    pub fn set_distance(&mut self, distance: f64) -> Result<(), &'static str> {
        if !self.active || !distance.is_finite() || distance < 0.0 {
            return Err("invalid camera distance or inactive");
        }
        let Some(c) = &self.classic else {
            return Err("Classic preset required");
        };
        self.first = distance < c.scale;
        self.distance = if self.first {
            0.5 * c.scale
        } else {
            distance.min(50.0 * c.scale)
        };
        Ok(())
    }
    /// Advance once per host frame. safe_distance includes the host's camera
    /// shape/near-plane clearance. Obstruction clamps immediately; recovery springs.
    pub fn advance(&mut self, dt: f64, safe_distance: Option<f64>) -> Result<(), &'static str> {
        if !self.active
            || !dt.is_finite()
            || !(0.0..=1.0).contains(&dt)
            || safe_distance.is_some_and(|d| !d.is_finite() || d < 0.0)
        {
            return Err("invalid camera advance or inactive");
        }
        let Some(c) = &mut self.classic else {
            return Err("Classic preset required");
        };
        let target = self.distance.min(safe_distance.unwrap_or(self.distance));
        // Independent analytic critically damped spring, fitted to Studio traces.
        let omega = 2.0 * std::f64::consts::PI * 4.5;
        let offset = c.position - target;
        let b = c.velocity + omega * offset;
        let decay = (-omega * dt).exp();
        let position = target + (offset + b * dt) * decay;
        let velocity = (c.velocity - omega * b * dt) * decay;
        if !position.is_finite() || !velocity.is_finite() {
            return Err("camera overflow");
        }
        c.position = position;
        c.velocity = velocity;
        if let Some(limit) = safe_distance {
            if c.position > limit {
                c.position = limit;
                c.velocity = c.velocity.min(0.0);
            }
        }
        Ok(())
    }
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
    pub fn look(&mut self, dy: f64, dp: f64) -> Result<(), &'static str> {
        if !self.active {
            return Err("camera inactive");
        }
        let (y, p) = (self.yaw + dy, self.pitch + dp);
        if !y.is_finite() || !p.is_finite() {
            return Err("invalid look");
        }
        self.yaw = (y + std::f64::consts::PI).rem_euclid(2.0 * std::f64::consts::PI)
            - std::f64::consts::PI;
        self.pitch = p.clamp(-80_f64.to_radians(), 80_f64.to_radians());
        Ok(())
    }
    pub fn zoom(&mut self, steps: f64) -> Result<(), &'static str> {
        if !self.active {
            return Err("camera inactive");
        }
        if !steps.is_finite() {
            return Err("invalid zoom");
        }
        if let Some(c) = &self.classic {
            let d = self.distance / c.scale;
            let studs = if steps >= 0.0 {
                (d + steps * (1.0 + d * 0.5)).max(1.0)
            } else {
                (d + steps) / (1.0 - steps * 0.5)
            };
            let scale = c.scale;
            return self.set_distance(studs.clamp(0.5, 50.0) * scale);
        }
        self.distance = (self.distance + steps * 0.5).clamp(0.0, 15.0);
        self.first = if self.first {
            self.distance < 0.45
        } else {
            self.distance <= 0.3
        };
        Ok(())
    }
    /// None is a clear boom. A host first queries the desired pose, sweeps its
    /// camera shape from focus, then recomputes with the safe hit fraction.
    pub fn pose(
        &self,
        focus: Vec3,
        eye: Vec3,
        obstruction: Option<f64>,
    ) -> Result<CameraPose, &'static str> {
        if !self.active {
            return Err("camera inactive");
        }
        if !finite(focus)
            || !finite(eye)
            || obstruction.is_some_and(|v| !v.is_finite() || !(0.0..=1.0).contains(&v))
        {
            return Err("invalid camera query");
        }
        let forward = [
            -self.yaw.sin() * self.pitch.cos(),
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
        ];
        let origin = if self.first { eye } else { focus };
        let fraction = obstruction.unwrap_or(1.0);
        let boom = self.classic.as_ref().map_or(self.distance, |c| c.position);
        let actual = if self.first && self.classic.is_none() {
            0.0
        } else {
            (boom * fraction - if fraction < 1.0 { 0.1 } else { 0.0 }).max(0.0)
        };
        let position = std::array::from_fn(|i| origin[i] - forward[i] * actual);
        if !finite(position) {
            return Err("camera overflow");
        }
        Ok(CameraPose {
            position,
            forward,
            first_person: u32::from(self.first),
            heading: self.yaw,
            requested_distance: self.distance,
            actual_distance: actual,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn classic_spring_clipping_and_frame_rate_independence() {
        let mut a = Camera::default();
        a.enable_classic(0.3).unwrap();
        a.set_active(true);
        a.set_distance(0.297).unwrap();
        let mut b = a.clone();
        for _ in 0..30 {
            a.advance(1.0 / 30.0, None).unwrap();
        }
        for _ in 0..144 {
            b.advance(1.0 / 144.0, None).unwrap();
        }
        let pa = a.pose([0.0; 3], [0.0; 3], None).unwrap();
        let pb = b.pose([0.0; 3], [0.0; 3], None).unwrap();
        assert_eq!(pa.first_person, 1);
        assert!((pa.actual_distance - 0.15).abs() < 1e-8);
        assert!((pa.actual_distance - pb.actual_distance).abs() < 1e-10);
        a.set_distance(3.75).unwrap();
        a.advance(1.0 / 30.0, None).unwrap();
        let p = a.pose([0.0; 3], [0.0; 3], None).unwrap();
        assert!(p.actual_distance > 0.15 && p.actual_distance < 3.75);
        a.advance(1.0 / 30.0, Some(0.1)).unwrap();
        assert!(a.pose([0.0; 3], [0.0; 3], None).unwrap().actual_distance <= 0.1);
        let before = a.pose([0.0; 3], [0.0; 3], None).unwrap();
        assert!(a.advance(f64::NAN, None).is_err());
        assert!(a.advance(0.1, Some(-0.1)).is_err());
        assert!(a.enable_classic(f64::INFINITY).is_err());
        assert_eq!(a.pose([0.0; 3], [0.0; 3], None).unwrap(), before);
        a.zoom(-100.0).unwrap();
        assert_eq!(a.pose([0.0; 3], [0.0; 3], None).unwrap().first_person, 1);
        a.zoom(1.0).unwrap();
        assert!((a.distance / 0.3 - 1.75).abs() < 1e-10);
        a.zoom(1.0).unwrap();
        assert!((a.distance / 0.3 - 3.625).abs() < 1e-10);
    }
    #[test]
    fn orbit_zoom_obstruction_and_invalid_rollback() {
        let mut c = Camera::default();
        let focus = [0.0, 0.0, 1.0];
        let eye = [0.0, 0.0, 1.4];
        assert!(c.pose(focus, eye, None).is_err());
        c.set_active(true);
        assert_eq!(c.pose(focus, eye, None).unwrap().position, [0.0, -3.0, 1.0]);
        c.zoom(-6.0).unwrap();
        assert_eq!(c.pose(focus, eye, None).unwrap().position, eye);
        assert_eq!(c.pose(focus, eye, None).unwrap().first_person, 1);
        c.zoom(1.0).unwrap();
        assert_eq!(c.pose(focus, eye, None).unwrap().first_person, 0);
        c.zoom(100.0).unwrap();
        c.look(std::f64::consts::FRAC_PI_2, 100.0).unwrap();
        let p = c.pose(focus, eye, Some(0.2)).unwrap();
        assert!((p.actual_distance - 2.9).abs() < 1e-10);
        assert_eq!(p.requested_distance, 15.0);
        assert!(c.pitch <= 80_f64.to_radians());
        let before = c.pose(focus, eye, None).unwrap();
        assert!(c.look(f64::NAN, 0.0).is_err());
        assert!(c.zoom(f64::INFINITY).is_err());
        assert_eq!(c.pose(focus, eye, None).unwrap(), before);
        assert!(c.pose(focus, eye, Some(-1.0)).is_err());
        c.set_active(false);
        assert!(c.look(0.0, 0.0).is_err());
    }
}
