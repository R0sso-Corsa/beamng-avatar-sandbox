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
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            distance: 3.0,
            first: false,
            active: false,
        }
    }
}
impl Camera {
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
        let actual = if self.first {
            0.0
        } else {
            (self.distance * fraction - if fraction < 1.0 { 0.1 } else { 0.0 }).max(0.0)
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
