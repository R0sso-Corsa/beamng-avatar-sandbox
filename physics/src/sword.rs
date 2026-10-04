//! Behaviour transcribed from the downloaded Doomspire sword Server/onDied scripts.
//! Host owns contact events, sounds, clip sampling and the vertical velocity motor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attack {
    Slash,
    Lunge,
}
impl Attack {
    pub fn animation_id(self) -> u64 {
        match self {
            Self::Slash => 129967390,
            Self::Lunge => 129967478,
        }
    }
    pub fn damage(self) -> f64 {
        match self {
            Self::Slash => 10.0,
            Self::Lunge => 30.0,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grip {
    Up,
    Out,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwordPose {
    pub grip: Grip,
    pub contact_damage: f64,
    /// Z-up target speed: 14.5 studs/s at 0.30 metres/stud; only for first 0.5s.
    pub lift_speed: Option<f64>,
    pub enabled: bool,
    pub attack: Option<Attack>,
    pub animation_time: f64,
}
#[derive(Default)]
pub struct Sword {
    time: f64,
    last_attack: f64,
    lunge_started: Option<f64>,
    animation: Option<(Attack, f64)>,
}
impl Sword {
    pub fn advance(&mut self, dt: f64) -> Result<(), &'static str> {
        if !dt.is_finite() || dt < 0.0 || !(self.time + dt).is_finite() {
            return Err("invalid sword delta");
        }
        self.time += dt;
        if self
            .lunge_started
            .is_some_and(|start| self.time - start >= 1.0)
        {
            self.lunge_started = None;
        }
        Ok(())
    }
    /// Primary reproduces the source's strict <0.2s double-click test.
    /// The source initializes last_attack=0; an activation before 0.2s lunges.
    pub fn activate(&mut self, force_lunge: bool) -> Result<Attack, &'static str> {
        if self.lunge_started.is_some() {
            return Err("sword lunging");
        }
        let attack = if force_lunge || self.time - self.last_attack < 0.2 {
            Attack::Lunge
        } else {
            Attack::Slash
        };
        self.last_attack = self.time;
        self.animation = Some((attack, self.time));
        if attack == Attack::Lunge {
            self.lunge_started = Some(self.time);
        }
        Ok(attack)
    }
    pub fn pose(&self) -> SwordPose {
        let age = self.lunge_started.map(|start| self.time - start);
        SwordPose {
            grip: if age.is_some_and(|t| t >= 0.25) {
                Grip::Out
            } else {
                Grip::Up
            },
            // Source attack() sets slash damage then resets to 5 without yielding.
            contact_damage: if age.is_some() { 30.0 } else { 5.0 },
            lift_speed: if age.is_some_and(|t| t < 0.5) {
                Some(4.35)
            } else {
                None
            },
            enabled: age.is_none(),
            attack: self.animation.map(|(a, _)| a),
            animation_time: self.animation.map_or(0.0, |(_, t)| self.time - t),
        }
    }
    /// Call only for host-authorized sword contact with a living target humanoid.
    pub fn contact_damage(
        &self,
        attacker_alive: bool,
        same_humanoid: bool,
        anti_team_kill: bool,
        both_non_neutral: bool,
        same_team: bool,
    ) -> Option<f64> {
        if !attacker_alive || same_humanoid || (anti_team_kill && both_non_neutral && same_team) {
            None
        } else {
            Some(self.pose().contact_damage)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_double_click_grip_lift_and_contact_timing() {
        let mut sword = Sword::default();
        sword.advance(1.0).unwrap();
        assert_eq!(sword.activate(false).unwrap(), Attack::Slash);
        assert_eq!(sword.pose().contact_damage, 5.0);
        sword.advance(0.1).unwrap();
        assert_eq!(sword.activate(false).unwrap(), Attack::Lunge);
        assert_eq!(sword.pose().lift_speed, Some(4.35));
        assert!(sword.activate(false).is_err());
        sword.advance(0.25).unwrap();
        assert_eq!(sword.pose().grip, Grip::Out);
        assert_eq!(
            sword.contact_damage(true, false, false, false, false),
            Some(30.0)
        );
        assert_eq!(sword.contact_damage(true, false, true, true, true), None);
        sword.advance(0.25).unwrap();
        assert_eq!(sword.pose().lift_speed, None);
        sword.advance(0.5).unwrap();
        assert_eq!(sword.pose().contact_damage, 5.0);
        assert!(sword.pose().enabled);
        assert_eq!(sword.activate(false).unwrap(), Attack::Slash);
        let before = sword.pose();
        assert!(sword.advance(f64::NAN).is_err());
        assert_eq!(sword.pose(), before);
        let mut edge = Sword::default();
        edge.advance(0.2).unwrap();
        assert_eq!(edge.activate(false).unwrap(), Attack::Slash);
        assert_eq!(Sword::default().activate(false).unwrap(), Attack::Lunge);
    }
}
