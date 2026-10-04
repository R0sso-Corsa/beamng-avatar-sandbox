//! Portable gear commands. Hosts own projectiles, hit tests, damage and visuals.
use crate::{finite, Vec3};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Gear {
    Sword = 0,
    Slingshot = 1,
    Rocket = 2,
    Trowel = 3,
    Bomb = 4,
    Superball = 5,
    Paintball = 6,
    BuildingTools = 7,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UseMode {
    Primary,
    SwordLunge,
    RemoveBlock(u64),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Effect {
    Melee {
        damage: f64,
    },
    Projectile {
        damage: f64,
        speed: Option<f64>,
        gravity_factor: Option<f64>,
        bounce_damage_factor: f64,
        explosion_radius: Option<f64>,
    },
    Wall {
        size: Vec3,
    },
    Bomb {
        damage: f64,
        fuse_seconds: f64,
    },
    PlaceBlock,
    RemoveBlock {
        id: u64,
    },
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Command {
    pub gear: Gear,
    pub origin: Vec3,
    pub direction: Vec3,
    pub effect: Effect,
}
#[derive(Default)]
pub struct GearSystem {
    equipped: Option<Gear>,
    building_tools: bool,
    remaining: [f64; 8],
    sword: crate::sword::Sword,
}
impl GearSystem {
    pub fn sword_pose(&self) -> crate::sword::SwordPose {
        self.sword.pose()
    }
    pub fn equipped(&self) -> Option<Gear> {
        self.equipped
    }
    pub fn set_building_tools_enabled(&mut self, enabled: bool) {
        self.building_tools = enabled;
        if !enabled && self.equipped == Some(Gear::BuildingTools) {
            self.equipped = None;
        }
    }
    pub fn equip(&mut self, gear: Gear) -> Result<(), &'static str> {
        if gear == Gear::BuildingTools && !self.building_tools {
            return Err("building tools disabled");
        }
        self.equipped = Some(gear);
        Ok(())
    }
    pub fn advance(&mut self, seconds: f64) -> Result<(), &'static str> {
        if !seconds.is_finite() || seconds < 0.0 {
            return Err("invalid elapsed time");
        }
        self.sword.advance(seconds)?;
        for t in &mut self.remaining {
            *t = (*t - seconds).max(0.0);
        }
        Ok(())
    }
    pub fn activate(
        &mut self,
        origin: Vec3,
        direction: Vec3,
        mode: UseMode,
    ) -> Result<Command, &'static str> {
        let gear = self.equipped.ok_or("no equipped gear")?;
        if !finite(origin) || !finite(direction) {
            return Err("invalid aim");
        }
        let length = direction.iter().map(|v| v * v).sum::<f64>().sqrt();
        if !length.is_finite() || length < 1e-9 {
            return Err("invalid aim");
        }
        if self.remaining[gear as usize] > 1e-9 {
            return Err("gear cooling down");
        }
        if mode != UseMode::Primary
            && !matches!(
                (gear, mode),
                (Gear::Sword, UseMode::SwordLunge) | (Gear::BuildingTools, UseMode::RemoveBlock(_))
            )
        {
            return Err("unsupported gear mode");
        }
        let (effect, cooldown) = match gear {
            Gear::Sword => {
                let attack = self.sword.activate(mode == UseMode::SwordLunge)?;
                (
                    Effect::Melee {
                        damage: attack.damage(),
                    },
                    0.0,
                )
            }
            Gear::Slingshot => (
                Effect::Projectile {
                    damage: 16.0,
                    speed: None,
                    gravity_factor: Some(1.0),
                    bounce_damage_factor: 0.0,
                    explosion_radius: None,
                },
                0.2,
            ),
            Gear::Rocket => (
                Effect::Projectile {
                    damage: 100.0,
                    speed: Some(18.0),
                    gravity_factor: Some(0.0),
                    bounce_damage_factor: 0.0,
                    explosion_radius: Some(1.2),
                },
                7.0,
            ),
            Gear::Trowel => (
                Effect::Wall {
                    size: [1.2, 0.3, 0.9],
                },
                4.0,
            ),
            Gear::Bomb => (
                Effect::Bomb {
                    damage: 100.0,
                    fuse_seconds: 3.8,
                },
                5.0,
            ),
            Gear::Superball => (
                Effect::Projectile {
                    damage: 55.0,
                    speed: None,
                    gravity_factor: Some(1.0),
                    bounce_damage_factor: 0.5,
                    explosion_radius: None,
                },
                2.0,
            ),
            Gear::Paintball => (
                Effect::Projectile {
                    damage: 20.0,
                    speed: None,
                    gravity_factor: None,
                    bounce_damage_factor: 0.0,
                    explosion_radius: None,
                },
                0.5,
            ),
            Gear::BuildingTools => (
                match mode {
                    UseMode::RemoveBlock(id) if id > 0 => Effect::RemoveBlock { id },
                    UseMode::Primary => Effect::PlaceBlock,
                    _ => return Err("invalid block id"),
                },
                0.0,
            ),
        };
        self.remaining[gear as usize] = cooldown;
        Ok(Command {
            gear,
            origin,
            direction: direction.map(|v| v / length),
            effect,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn loadout_cooldowns_modes_and_opt_in() {
        let mut g = GearSystem::default();
        assert!(g.equip(Gear::BuildingTools).is_err());
        for gear in [
            Gear::Sword,
            Gear::Slingshot,
            Gear::Rocket,
            Gear::Trowel,
            Gear::Bomb,
            Gear::Superball,
            Gear::Paintball,
        ] {
            g.equip(gear).unwrap();
            assert!(g.activate([0.0; 3], [0.0; 3], UseMode::Primary).is_err());
            let command = g
                .activate([0.0; 3], [2.0, 0.0, 0.0], UseMode::Primary)
                .unwrap();
            assert_eq!(command.direction, [1.0, 0.0, 0.0]);
            assert!(g
                .activate([0.0; 3], [1.0, 0.0, 0.0], UseMode::Primary)
                .is_err());
            g.advance(7.0).unwrap();
        }
        g.equip(Gear::Sword).unwrap();
        assert_eq!(
            g.activate([0.0; 3], [1.0, 0.0, 0.0], UseMode::SwordLunge)
                .unwrap()
                .effect,
            Effect::Melee { damage: 30.0 }
        );
        g.set_building_tools_enabled(true);
        g.equip(Gear::BuildingTools).unwrap();
        assert_eq!(
            g.activate([0.0; 3], [1.0, 0.0, 0.0], UseMode::RemoveBlock(42))
                .unwrap()
                .effect,
            Effect::RemoveBlock { id: 42 }
        );
        g.set_building_tools_enabled(false);
        assert_eq!(g.equipped(), None);
        assert!(g.advance(f64::NAN).is_err());
    }
}

/// Host-configured projectile parameters for values not established by the wiki.
#[derive(Clone, Copy)]
pub struct ProjectileConfig {
    pub speed: f64,
    pub gravity: f64,
    pub lifetime: f64,
}
#[derive(Clone, Copy)]
pub struct Impact {
    pub fraction: f64,
    pub normal: Vec3,
    pub player: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProjectileEvent {
    Damage(f64),
    Explosion { damage: f64, radius: f64 },
    Bounce,
    Expired,
}
#[derive(Clone, Copy)]
pub struct Projectile {
    pub position: Vec3,
    pub velocity: Vec3,
    damage: f64,
    bounce: f64,
    explosion: Option<f64>,
    gravity: f64,
    remaining: f64,
    active: bool,
    stop_on_surface: bool,
}
/// Z-up render frame; host maps the mesh's local nose axis onto forward.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlightFrame {
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
}
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
impl Projectile {
    /// Recompute after each step/impact so bounced projectiles face their velocity.
    pub fn facing(&self) -> Result<FlightFrame, &'static str> {
        let length = self.velocity.iter().map(|v| v * v).sum::<f64>().sqrt();
        if !length.is_finite() || length < 1e-9 {
            return Err("invalid projectile heading");
        }
        let forward = self.velocity.map(|v| v / length);
        let reference = if forward[2].abs() > 0.999 {
            [0.0, 1.0, 0.0]
        } else {
            [0.0, 0.0, 1.0]
        };
        let right = cross(forward, reference);
        let norm = right.iter().map(|v| v * v).sum::<f64>().sqrt();
        let right = right.map(|v| v / norm);
        let up = cross(right, forward);
        Ok(FlightFrame { forward, right, up })
    }

    pub fn new(command: Command, config: ProjectileConfig) -> Result<Self, &'static str> {
        let Effect::Projectile {
            damage,
            speed,
            gravity_factor,
            bounce_damage_factor,
            explosion_radius,
        } = command.effect
        else {
            return Err("not a projectile");
        };
        if !finite(command.origin)
            || !finite(command.direction)
            || !config.speed.is_finite()
            || config.speed <= 0.0
            || !config.gravity.is_finite()
            || config.gravity < 0.0
            || !config.lifetime.is_finite()
            || config.lifetime <= 0.0
            || !damage.is_finite()
            || damage < 0.0
            || !bounce_damage_factor.is_finite()
            || !(0.0..=1.0).contains(&bounce_damage_factor)
            || speed.is_some_and(|v| !v.is_finite() || v <= 0.0)
            || gravity_factor.is_some_and(|v| !v.is_finite() || v < 0.0)
            || explosion_radius.is_some_and(|v| !v.is_finite() || v <= 0.0)
        {
            return Err("invalid projectile configuration");
        }
        let length = command.direction.iter().map(|v| v * v).sum::<f64>().sqrt();
        if !length.is_finite() || length < 1e-9 {
            return Err("invalid aim");
        }
        let velocity = command
            .direction
            .map(|v| v / length * speed.unwrap_or(config.speed));
        let gravity = config.gravity * gravity_factor.unwrap_or(1.0);
        if !finite(velocity) || !gravity.is_finite() {
            return Err("projectile overflow");
        }
        Ok(Self {
            position: command.origin,
            velocity,
            damage,
            bounce: bounce_damage_factor,
            explosion: explosion_radius,
            gravity,
            remaining: config.lifetime,
            active: true,
            stop_on_surface: command.gear == Gear::Paintball,
        })
    }
    /// Host sweeps a projectile shape over this segment, then supplies its first hit.
    pub fn segment(&self, dt: f64) -> Result<(Vec3, Vec3), &'static str> {
        if !dt.is_finite() || dt <= 0.0 {
            return Err("invalid delta");
        }
        let t = dt.min(self.remaining);
        let mut end = std::array::from_fn(|i| self.position[i] + self.velocity[i] * t);
        end[2] -= 0.5 * self.gravity * t * t;
        if !finite(end) {
            return Err("projectile overflow");
        }
        Ok((self.position, end))
    }
    pub fn advance(
        &mut self,
        dt: f64,
        impact: Option<Impact>,
    ) -> Result<Option<ProjectileEvent>, &'static str> {
        let (start, end) = self.segment(dt)?;
        if !self.active {
            return Ok(None);
        }
        if let Some(hit) = impact {
            let norm = hit.normal.iter().map(|v| v * v).sum::<f64>();
            if !hit.fraction.is_finite()
                || !(0.0..=1.0).contains(&hit.fraction)
                || !finite(hit.normal)
                || (norm - 1.0).abs() > 1e-6
            {
                return Err("invalid impact");
            }
        }
        let t = dt.min(self.remaining);
        self.remaining -= t;
        self.velocity[2] -= self.gravity * t * impact.map_or(1.0, |h| h.fraction);
        self.position = end;
        if let Some(hit) = impact {
            self.position = std::array::from_fn(|i| start[i] + (end[i] - start[i]) * hit.fraction);
            if let Some(radius) = self.explosion {
                self.active = false;
                return Ok(Some(ProjectileEvent::Explosion {
                    damage: self.damage,
                    radius,
                }));
            }
            if hit.player {
                self.active = false;
                return Ok(Some(ProjectileEvent::Damage(self.damage)));
            }
            if self.stop_on_surface {
                self.active = false;
                return Ok(Some(ProjectileEvent::Expired));
            }
            let dot = self
                .velocity
                .iter()
                .zip(hit.normal)
                .map(|(v, n)| v * n)
                .sum::<f64>();
            if dot < 0.0 {
                self.velocity =
                    std::array::from_fn(|i| self.velocity[i] - 2.0 * dot * hit.normal[i]);
            }
            self.damage *= self.bounce;
            return Ok(Some(ProjectileEvent::Bounce));
        }
        if self.remaining <= 0.0 {
            self.active = false;
            return Ok(Some(ProjectileEvent::Expired));
        }
        Ok(None)
    }
}
pub struct BombTimer {
    remaining: f64,
    damage: f64,
    fired: bool,
}
impl BombTimer {
    pub fn new(command: Command) -> Result<Self, &'static str> {
        let Effect::Bomb {
            damage,
            fuse_seconds,
        } = command.effect
        else {
            return Err("not a bomb");
        };
        if !damage.is_finite() || damage < 0.0 || !fuse_seconds.is_finite() || fuse_seconds <= 0.0 {
            return Err("invalid bomb");
        }
        Ok(Self {
            remaining: fuse_seconds,
            damage,
            fired: false,
        })
    }
    /// Radius/impulse are host settings; return nominal damage exactly once.
    pub fn advance(&mut self, dt: f64) -> Result<Option<f64>, &'static str> {
        if !dt.is_finite() || dt < 0.0 {
            return Err("invalid delta");
        }
        if self.fired {
            return Ok(None);
        }
        self.remaining = (self.remaining - dt).max(0.0);
        if self.remaining == 0.0 {
            self.fired = true;
            Ok(Some(self.damage))
        } else {
            Ok(None)
        }
    }
}
#[cfg(test)]
mod projectile_tests {
    use super::*;
    #[test]
    fn trajectories_impacts_expiry_and_bomb_fuse() {
        let mut gears = GearSystem::default();
        let config = ProjectileConfig {
            speed: 10.0,
            gravity: 10.0,
            lifetime: 2.0,
        };
        gears.equip(Gear::Superball).unwrap();
        let cmd = gears
            .activate([0.0; 3], [1.0, 0.0, 0.0], UseMode::Primary)
            .unwrap();
        let mut ball = Projectile::new(cmd, config).unwrap();
        ball.advance(0.1, None).unwrap();
        assert!((ball.position[2] + 0.05).abs() < 1e-10);
        let before = ball.position;
        assert!(ball
            .advance(
                0.1,
                Some(Impact {
                    fraction: 2.0,
                    normal: [-1.0, 0.0, 0.0],
                    player: false
                })
            )
            .is_err());
        assert_eq!(ball.position, before);
        assert_eq!(
            ball.advance(
                0.1,
                Some(Impact {
                    fraction: 0.5,
                    normal: [-1.0, 0.0, 0.0],
                    player: false
                })
            )
            .unwrap(),
            Some(ProjectileEvent::Bounce)
        );
        assert!(ball.velocity[0] < 0.0);
        assert_eq!(
            ball.advance(
                0.1,
                Some(Impact {
                    fraction: 0.5,
                    normal: [1.0, 0.0, 0.0],
                    player: true
                })
            )
            .unwrap(),
            Some(ProjectileEvent::Damage(27.5))
        );
        assert_eq!(ball.advance(0.1, None).unwrap(), None);
        let mut expired = Projectile::new(cmd, config).unwrap();
        assert_eq!(
            expired.advance(3.0, None).unwrap(),
            Some(ProjectileEvent::Expired)
        );
        gears.equip(Gear::Rocket).unwrap();
        let mut rocket = Projectile::new(
            gears
                .activate([0.0; 3], [1.0, 0.0, 0.0], UseMode::Primary)
                .unwrap(),
            config,
        )
        .unwrap();
        rocket.advance(0.1, None).unwrap();
        assert_eq!(rocket.position[2], 0.0);
        assert_eq!(
            rocket
                .advance(
                    0.1,
                    Some(Impact {
                        fraction: 1.0,
                        normal: [-1.0, 0.0, 0.0],
                        player: false
                    })
                )
                .unwrap(),
            Some(ProjectileEvent::Explosion {
                damage: 100.0,
                radius: 1.2
            })
        );
        gears.equip(Gear::Bomb).unwrap();
        let mut bomb = BombTimer::new(
            gears
                .activate([0.0; 3], [1.0, 0.0, 0.0], UseMode::Primary)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(bomb.advance(3.0).unwrap(), None);
        assert_eq!(bomb.advance(1.0).unwrap(), Some(100.0));
        assert_eq!(bomb.advance(1.0).unwrap(), None);
    }
    #[test]
    fn projectile_facing_tracks_velocity_including_vertical_and_bounce() {
        for direction in [[1.0, 2.0, 3.0], [0.0, 0.0, 1.0], [0.0, 0.0, -1.0]] {
            let mut gears = GearSystem::default();
            gears.equip(Gear::Rocket).unwrap();
            let mut rocket = Projectile::new(
                gears
                    .activate([0.0; 3], direction, UseMode::Primary)
                    .unwrap(),
                ProjectileConfig {
                    speed: 18.0,
                    gravity: 58.86,
                    lifetime: 6.0,
                },
            )
            .unwrap();
            rocket.advance(0.2, None).unwrap();
            let frame = rocket.facing().unwrap();
            let norm = direction.iter().map(|v| v * v).sum::<f64>().sqrt();
            for i in 0..3 {
                assert!((frame.forward[i] - direction[i] / norm).abs() < 1e-9);
            }
            for axis in [frame.forward, frame.right, frame.up] {
                assert!((axis.iter().map(|v| v * v).sum::<f64>() - 1.0).abs() < 1e-9);
            }
            assert!(
                frame
                    .right
                    .iter()
                    .zip(frame.forward)
                    .map(|(a, b)| a * b)
                    .sum::<f64>()
                    .abs()
                    < 1e-9
            );
        }
        let mut gears = GearSystem::default();
        gears.equip(Gear::Superball).unwrap();
        let mut ball = Projectile::new(
            gears
                .activate([0.0; 3], [1.0, 0.0, 0.0], UseMode::Primary)
                .unwrap(),
            ProjectileConfig {
                speed: 18.0,
                gravity: 58.86,
                lifetime: 6.0,
            },
        )
        .unwrap();
        ball.advance(
            0.1,
            Some(Impact {
                fraction: 0.5,
                normal: [-1.0, 0.0, 0.0],
                player: false,
            }),
        )
        .unwrap();
        assert!(ball.facing().unwrap().forward[0] < 0.0);
    }
}
