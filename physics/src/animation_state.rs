//! Engine-independent clip selection. This selects clips; the host samples poses.
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Clip {
    Idle = 0,
    Walk = 1,
    Jump = 2,
    Fall = 3,
    Climb = 4,
}
#[derive(Clone, Copy)]
pub struct Motion {
    pub active: bool,
    pub grounded: bool,
    pub jumped: bool,
    pub climbing: bool,
    pub speed: f64,
}
#[derive(Clone, Copy, Debug)]
pub struct Selection {
    pub clip: Clip,
    pub time: f64,
    pub rate: f64,
    pub blend_seconds: f64,
    pub changed: bool,
}
pub struct Animator {
    clip: Clip,
    time: f64,
    jump_remaining: f64,
}
impl Default for Animator {
    fn default() -> Self {
        Self {
            clip: Clip::Idle,
            time: 0.0,
            jump_remaining: 0.0,
        }
    }
}
impl Animator {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    /// Nonlooping jump/fall timing is clamped by each clip sampler. For transitions,
    /// retain the last rendered pose and blend to this selection over blend_seconds.
    pub fn update(&mut self, dt: f64, m: Motion) -> Result<Option<Selection>, &'static str> {
        if !dt.is_finite()
            || !(0.0..=1.0).contains(&dt)
            || !m.speed.is_finite()
            || m.speed < 0.0
            || m.speed > 10000.0
        {
            return Err("invalid animation motion");
        }
        if !m.active {
            self.reset();
            return Ok(None);
        }
        if m.jumped && !m.grounded && !m.climbing {
            self.jump_remaining = 0.3;
        }
        let clip = if m.climbing {
            Clip::Climb
        } else if m.grounded {
            if m.speed > 0.03 {
                Clip::Walk
            } else {
                Clip::Idle
            }
        } else if self.jump_remaining > 0.0 {
            Clip::Jump
        } else {
            Clip::Fall
        };
        if m.grounded || m.climbing {
            self.jump_remaining = 0.0;
        }
        self.jump_remaining = (self.jump_remaining - dt).max(0.0);
        let rate = match clip {
            Clip::Walk => (m.speed / 4.8).clamp(0.0, 4.0),
            Clip::Climb => (m.speed / 3.6).clamp(0.0, 4.0),
            _ => 1.0,
        };
        let changed = clip != self.clip;
        if changed {
            self.time = 0.0;
        } else {
            self.time += dt * rate;
        }
        self.clip = clip;
        Ok(Some(Selection {
            clip,
            time: self.time,
            rate,
            blend_seconds: if clip == Clip::Fall { 0.3 } else { 0.1 },
            changed,
        }))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn locomotion_jump_fall_climb_disable_and_invalid_rollback() {
        let mut a = Animator::default();
        let mut m = Motion {
            active: true,
            grounded: true,
            jumped: false,
            climbing: false,
            speed: 0.0,
        };
        assert_eq!(a.update(0.01, m).unwrap().unwrap().clip, Clip::Idle);
        m.speed = 4.8;
        let s = a.update(0.01, m).unwrap().unwrap();
        assert!(s.changed);
        assert_eq!(s.time, 0.0);
        assert_eq!(s.rate, 1.0);
        m.grounded = false;
        m.jumped = true;
        assert_eq!(a.update(0.1, m).unwrap().unwrap().clip, Clip::Jump);
        m.jumped = false;
        a.update(0.3, m).unwrap();
        assert_eq!(a.update(0.01, m).unwrap().unwrap().clip, Clip::Fall);
        m.climbing = true;
        assert_eq!(a.update(0.01, m).unwrap().unwrap().clip, Clip::Climb);
        let time = a.time;
        assert!(a.update(f64::NAN, m).is_err());
        assert_eq!(a.time, time);
        m.active = false;
        assert!(a.update(0.01, m).unwrap().is_none());
        m.active = true;
        m.climbing = false;
        assert_eq!(a.update(0.01, m).unwrap().unwrap().clip, Clip::Fall);
    }
}
