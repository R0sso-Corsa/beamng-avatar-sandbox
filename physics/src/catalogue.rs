//! Approved appearance IDs and snapshots; hosts authenticate players/transport.
use std::collections::{BTreeMap, BTreeSet};
fn avatar(id: &str) -> bool {
    id == "noob"
        || (id.len() == 64
            && id
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
}
fn catalogue(ids: &[String]) -> Result<BTreeSet<String>, &'static str> {
    if ids.len() > 256 || ids.iter().any(|s| !avatar(s)) {
        return Err("invalid catalogue");
    }
    let mut set: BTreeSet<_> = ids.iter().cloned().collect();
    set.insert("noob".into());
    Ok(set)
}
#[derive(Clone)]
pub struct Snapshot {
    pub version: u32,
    pub revision: u64,
    pub assignments: Vec<(u32, String)>,
}
pub struct Server {
    allowed: BTreeSet<String>,
    active: BTreeMap<u32, String>,
    revision: u64,
}
impl Server {
    pub fn new(ids: &[String]) -> Result<Self, &'static str> {
        Ok(Self {
            allowed: catalogue(ids)?,
            active: BTreeMap::new(),
            revision: 0,
        })
    }
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            version: 1,
            revision: self.revision,
            assignments: self.active.iter().map(|(p, a)| (*p, a.clone())).collect(),
        }
    }
    pub fn join(&mut self, p: u32) -> Result<Snapshot, &'static str> {
        if p > i32::MAX as u32 || self.active.contains_key(&p) || self.active.len() >= 256 {
            return Err("invalid player");
        }
        self.bump()?;
        self.active.insert(p, "noob".into());
        Ok(self.snapshot())
    }
    fn bump(&mut self) -> Result<(), &'static str> {
        if self.revision >= 9007199254740991 {
            return Err("revision exhausted");
        }
        self.revision += 1;
        Ok(())
    }
    pub fn select(&mut self, p: u32, a: &str) -> Result<Snapshot, &'static str> {
        if !self.active.contains_key(&p) || !self.allowed.contains(a) {
            return Err("unapproved selection");
        }
        if self.active[&p] != a {
            self.bump()?;
            self.active.insert(p, a.into());
        }
        Ok(self.snapshot())
    }
    pub fn leave(&mut self, p: u32) -> Result<Snapshot, &'static str> {
        if !self.active.contains_key(&p) {
            return Err("unknown player");
        }
        self.bump()?;
        self.active.remove(&p);
        Ok(self.snapshot())
    }
}
pub struct Client {
    installed: BTreeSet<String>,
    active: BTreeMap<u32, (String, String)>,
    revision: Option<u64>,
}
impl Client {
    pub fn new(ids: &[String]) -> Result<Self, &'static str> {
        Ok(Self {
            installed: catalogue(ids)?,
            active: BTreeMap::new(),
            revision: None,
        })
    }
    pub fn apply(&mut self, s: &Snapshot) -> Result<(), &'static str> {
        if s.version != 1
            || s.revision > 9007199254740991
            || self.revision.is_some_and(|r| s.revision <= r)
            || s.assignments.len() > 256
        {
            return Err("invalid or stale snapshot");
        }
        let mut next = BTreeMap::new();
        for (p, a) in &s.assignments {
            if *p > i32::MAX as u32 || !avatar(a) || next.contains_key(p) {
                return Err("invalid assignment");
            }
            let resolved = if self.installed.contains(a) {
                a.clone()
            } else {
                "noob".into()
            };
            next.insert(*p, (a.clone(), resolved));
        }
        self.active = next;
        self.revision = Some(s.revision);
        Ok(())
    }
    pub fn get(&self, p: u32) -> Option<&(String, String)> {
        self.active.get(&p)
    }
    pub fn reset(&mut self) {
        self.active.clear();
        self.revision = None;
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn approval_fallback_stale_and_rollback() {
        let hash = "a".repeat(64);
        let mut s = Server::new(&[hash.clone()]).unwrap();
        let mut c = Client::new(&[]).unwrap();
        c.apply(&s.join(1).unwrap()).unwrap();
        let selected = s.select(1, &hash).unwrap();
        c.apply(&selected).unwrap();
        assert_eq!(c.get(1).unwrap().1, "noob");
        assert!(c.apply(&selected).is_err());
        assert!(s.select(2, &hash).is_err());
        let bad = Snapshot {
            version: 1,
            revision: 99,
            assignments: vec![(1, hash.clone()), (1, hash)],
        };
        assert!(c.apply(&bad).is_err());
        c.apply(&s.leave(1).unwrap()).unwrap();
        assert!(c.get(1).is_none());
        c.reset();
        c.apply(&Snapshot {
            version: 1,
            revision: 0,
            assignments: vec![],
        })
        .unwrap();
    }
}
