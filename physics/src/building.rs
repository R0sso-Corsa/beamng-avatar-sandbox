//! Portable axis-aligned building plans. Host supplies fresh blockers/rendering.
use crate::{finite, Vec3};
use std::collections::BTreeMap;
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub min: Vec3,
    pub max: Vec3,
}
fn valid(b: Bounds) -> bool {
    finite(b.min)
        && finite(b.max)
        && (0..3).all(|i| b.min[i].abs() <= 1e6 && b.max[i].abs() <= 1e6 && b.min[i] <= b.max[i])
}
fn overlaps(a: Bounds, b: Bounds) -> bool {
    (0..3).all(|i| a.min[i] < b.max[i] && a.max[i] > b.min[i])
}
#[derive(Default)]
pub struct BuildingPlans {
    enabled: bool,
    next: u64,
    parts: BTreeMap<u64, Bounds>,
}
impl BuildingPlans {
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
    pub fn get(&self, id: u64) -> Option<Bounds> {
        self.parts.get(&id).copied()
    }
    fn check(
        &self,
        candidate: Bounds,
        ignore: Option<u64>,
        player: Bounds,
        vehicles: &[Bounds],
    ) -> Result<(), &'static str> {
        if !self.enabled {
            return Err("building tools disabled");
        }
        if !valid(candidate)
            || (0..3).any(|i| candidate.max[i] - candidate.min[i] < 0.01)
            || !valid(player)
            || vehicles.iter().any(|v| !valid(*v))
        {
            return Err("invalid bounds");
        }
        if overlaps(candidate, player)
            || vehicles.iter().any(|v| overlaps(candidate, *v))
            || self
                .parts
                .iter()
                .any(|(id, b)| Some(*id) != ignore && overlaps(candidate, *b))
        {
            return Err("overlapping part");
        }
        Ok(())
    }
    pub fn preview(
        &self,
        centre: Vec3,
        player: Bounds,
        vehicles: &[Bounds],
    ) -> Result<Bounds, &'static str> {
        if !finite(centre) {
            return Err("invalid centre");
        }
        let min = centre.map(f64::floor);
        let candidate = Bounds {
            min,
            max: min.map(|v| v + 1.0),
        };
        self.check(candidate, None, player, vehicles)?;
        Ok(candidate)
    }
    fn insert(&mut self, b: Bounds) -> Result<u64, &'static str> {
        let id = self.next.checked_add(1).ok_or("id exhausted")?;
        self.next = id;
        self.parts.insert(id, b);
        Ok(id)
    }
    pub fn place(
        &mut self,
        centre: Vec3,
        player: Bounds,
        vehicles: &[Bounds],
    ) -> Result<u64, &'static str> {
        let b = self.preview(centre, player, vehicles)?;
        self.insert(b)
    }
    pub fn move_part(
        &mut self,
        id: u64,
        offset: Vec3,
        player: Bounds,
        vehicles: &[Bounds],
    ) -> Result<(), &'static str> {
        let mut b = self.get(id).ok_or("unknown part")?;
        b.min = std::array::from_fn(|i| b.min[i] + offset[i]);
        b.max = std::array::from_fn(|i| b.max[i] + offset[i]);
        self.check(b, Some(id), player, vehicles)?;
        self.parts.insert(id, b);
        Ok(())
    }
    pub fn resize(
        &mut self,
        id: u64,
        size: Vec3,
        player: Bounds,
        vehicles: &[Bounds],
    ) -> Result<(), &'static str> {
        let old = self.get(id).ok_or("unknown part")?;
        let centre: Vec3 = std::array::from_fn(|i| (old.min[i] + old.max[i]) / 2.0);
        let b = Bounds {
            min: std::array::from_fn(|i| centre[i] - size[i] / 2.0),
            max: std::array::from_fn(|i| centre[i] + size[i] / 2.0),
        };
        self.check(b, Some(id), player, vehicles)?;
        self.parts.insert(id, b);
        Ok(())
    }
    pub fn clone_part(
        &mut self,
        id: u64,
        offset: Vec3,
        player: Bounds,
        vehicles: &[Bounds],
    ) -> Result<u64, &'static str> {
        let b = self.get(id).ok_or("unknown part")?;
        let b = Bounds {
            min: std::array::from_fn(|i| b.min[i] + offset[i]),
            max: std::array::from_fn(|i| b.max[i] + offset[i]),
        };
        self.check(b, None, player, vehicles)?;
        self.insert(b)
    }
    pub fn remove(&mut self, id: u64) -> Result<(), &'static str> {
        if !self.enabled {
            return Err("building tools disabled");
        }
        self.parts.remove(&id).ok_or("unknown part")?;
        Ok(())
    }
    pub fn clear(&mut self) {
        self.parts.clear();
        self.enabled = false;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn edit_validation_and_opt_in() {
        let mut p = BuildingPlans::default();
        let player = Bounds {
            min: [20.0; 3],
            max: [21.0; 3],
        };
        assert!(p.place([0.0; 3], player, &[]).is_err());
        p.set_enabled(true);
        let id = p.place([-0.1, 0.2, 0.5], player, &[]).unwrap();
        assert_eq!(p.get(id).unwrap().min, [-1.0, 0.0, 0.0]);
        p.move_part(id, [3.0, 0.0, 0.0], player, &[]).unwrap();
        p.resize(id, [2.0, 1.0, 1.0], player, &[]).unwrap();
        let cloned = p.clone_part(id, [0.0, 2.0, 0.0], player, &[]).unwrap();
        let before = p.get(cloned);
        assert!(p.move_part(cloned, [0.0, -2.0, 0.0], player, &[]).is_err());
        assert_eq!(p.get(cloned), before);
        assert!(p.resize(id, [f64::NAN, 1.0, 1.0], player, &[]).is_err());
        p.remove(id).unwrap();
        p.clear();
        assert_eq!(p.get(cloned), None);
        assert!(p.place([0.0; 3], player, &[]).is_err());
    }
}
