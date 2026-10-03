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
}
impl GearSystem {
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
            Gear::Sword => (
                Effect::Melee {
                    damage: if mode == UseMode::SwordLunge {
                        30.0
                    } else {
                        10.0
                    },
                },
                0.75,
            ),
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
