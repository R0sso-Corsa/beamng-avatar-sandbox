//! Local snapshot relay. No character simulation or game-specific engine calls.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{Duration, Instant};

pub const MAX_BYTES: usize = 8 * 1024;
pub const LEASE: Duration = Duration::from_secs(1);
const MAX_SEQ: u64 = (1_u64 << 53) - 1;

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    pub token: String,
    pub session: String,
    pub http_port: u16,
    pub udp_port: u16,
    pub metres_per_stud: f64,
    pub host_origin: [f64; 3],
}
impl Config {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.token.len() < 32
            || self.token.len() > 128
            || self.session.is_empty()
            || self.session.len() > 64
            || self.http_port == 0
            || self.udp_port == 0
            || !self.metres_per_stud.is_finite()
            || !(0.01..=10.0).contains(&self.metres_per_stud)
            || !vector(self.host_origin, 1e6)
        {
            return Err("invalid configuration");
        }
        Ok(())
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Message {
    version: u32,
    token: String,
    session: String,
    sequence: u64,
    kind: String,
    payload: Value,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Collider {
    pub id: u32,
    pub position: [f64; 3],
    pub size: [f64; 3],
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Host {
    pub enabled: bool,
    pub movement: [f64; 2],
    pub jump: bool,
    #[serde(deserialize_with = "collider_array")]
    pub colliders: Vec<Collider>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Part {
    pub id: String,
    pub position: [f64; 3],
    pub size: [f64; 3],
    pub right: [f64; 3],
    pub up: [f64; 3],
    pub back: [f64; 3],
    pub color: [f64; 3],
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Camera {
    pub position: [f64; 3],
    pub forward: [f64; 3],
    pub up: [f64; 3],
    pub fov: f64,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Guest {
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub state: String,
    pub parts: Vec<Part>,
    pub camera: Camera,
}
// Lua encoders disagree on an empty table's JSON representation.
fn collider_array<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Collider>, D::Error> {
    let value = Value::deserialize(d)?;
    if value.as_object().is_some_and(|o| o.is_empty()) {
        return Ok(Vec::new());
    }
    serde_json::from_value(value).map_err(serde::de::Error::custom)
}
fn vector(v: [f64; 3], bound: f64) -> bool {
    v.iter().all(|x| x.is_finite() && x.abs() <= bound)
}
fn size(v: [f64; 3]) -> bool {
    vector(v, 1000.0) && v.iter().all(|x| *x > 0.0)
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}
fn unit(v: [f64; 3]) -> bool {
    vector(v, 1.001) && (dot(v, v) - 1.0).abs() < 0.002
}
fn basis(r: [f64; 3], u: [f64; 3], b: [f64; 3]) -> bool {
    let cross = [
        r[1] * u[2] - r[2] * u[1],
        r[2] * u[0] - r[0] * u[2],
        r[0] * u[1] - r[1] * u[0],
    ];
    unit(r) && unit(u) && unit(b) && dot(r, u).abs() < 0.002 && dot(cross, b) > 0.998
}
impl Host {
    fn valid(&self) -> bool {
        let mut ids = std::collections::HashSet::new();
        self.movement.iter().all(|x| x.is_finite())
            && self.movement.iter().map(|x| x * x).sum::<f64>() <= 1.00001
            && self.colliders.len() <= 32
            && self
                .colliders
                .iter()
                .all(|b| b.id > 0 && ids.insert(b.id) && vector(b.position, 1e6) && size(b.size))
    }
}
impl Guest {
    fn valid(&self) -> bool {
        let mut ids = std::collections::HashSet::new();
        vector(self.position, 1e6)
            && vector(self.velocity, 1e4)
            && !self.state.is_empty()
            && self.state.len() <= 64
            && !self.parts.is_empty()
            && self.parts.len() <= 16
            && self.parts.iter().all(|p| {
                !p.id.is_empty()
                    && p.id.len() <= 64
                    && ids.insert(&p.id)
                    && vector(p.position, 1e6)
                    && size(p.size)
                    && basis(p.right, p.up, p.back)
                    && p.color
                        .iter()
                        .all(|x| x.is_finite() && (0.0..=1.0).contains(x))
            })
            && vector(self.camera.position, 1e6)
            && unit(self.camera.forward)
            && unit(self.camera.up)
            && dot(self.camera.forward, self.camera.up).abs() < 0.002
            && self.camera.fov.is_finite()
            && (1.0..=120.0).contains(&self.camera.fov)
    }
}
struct Snapshot {
    sequence: u64,
    received: Instant,
    payload: Value,
}
pub struct Relay {
    config: Config,
    host: Option<Snapshot>,
    guest: Option<Snapshot>,
}
impl Relay {
    pub fn new(config: Config) -> Result<Self, &'static str> {
        config.validate()?;
        Ok(Self {
            config,
            host: None,
            guest: None,
        })
    }
    pub fn exchange(
        &mut self,
        bytes: &[u8],
        kind: &str,
        now: Instant,
    ) -> Result<Value, &'static str> {
        if bytes.len() > MAX_BYTES {
            return Err("message too large");
        }
        let m: Message = serde_json::from_slice(bytes).map_err(|_| "invalid message")?;
        if m.version != 1
            || m.session != self.config.session
            || m.token != self.config.token
            || m.kind != kind
        {
            return Err("session, version or role mismatch");
        }
        if m.sequence == 0 || m.sequence > MAX_SEQ {
            return Err("invalid sequence");
        }
        let valid = match kind {
            "host" => serde_json::from_value::<Host>(m.payload.clone()).is_ok_and(|p| p.valid()),
            "guest" => serde_json::from_value::<Guest>(m.payload.clone()).is_ok_and(|p| p.valid()),
            _ => false,
        };
        if !valid || serde_json::to_vec(&m.payload).map_or(true, |p| p.len() > MAX_BYTES - 512) {
            return Err("invalid payload");
        }
        let slot = if kind == "host" {
            &mut self.host
        } else {
            &mut self.guest
        };
        if slot.as_ref().is_some_and(|s| m.sequence <= s.sequence) {
            return Err("old sequence");
        }
        *slot = Some(Snapshot {
            sequence: m.sequence,
            received: now,
            payload: m.payload,
        });
        let fresh = |s: &Snapshot| now.saturating_duration_since(s.received) < LEASE;
        let active = self
            .host
            .as_ref()
            .is_some_and(|h| fresh(h) && h.payload["enabled"] == true)
            && self.guest.as_ref().is_some_and(fresh);
        let peer = if kind == "host" {
            &self.guest
        } else {
            &self.host
        };
        let peer = peer.as_ref().filter(|s| fresh(s));
        Ok(
            json!({"version":1,"session":self.config.session,"sequence":m.sequence,"ok":true,
            "active":active,"peerSequence":peer.map(|s|s.sequence),
            "peerAgeMs":peer.map(|s|now.saturating_duration_since(s.received).as_millis() as u64),
            "payload":peer.map(|s|&s.payload)}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> Config {
        Config {
            token: "a".repeat(48),
            session: "unit-test".into(),
            http_port: 28741,
            udp_port: 28742,
            metres_per_stud: 0.3,
            host_origin: [0.0; 3],
        }
    }
    fn guest() -> Value {
        json!({"position":[1,2,3],"velocity":[4,0,0],"state":"Running",
          "parts":[{"id":"Head","position":[1,2,4],"size":[0.6,0.3,0.3],
          "right":[1,0,0],"up":[0,0,1],"back":[0,-1,0],"color":[1,1,0]}],
          "camera":{"position":[1,-2,3],"forward":[0,1,0],"up":[0,0,1],"fov":70}})
    }
    fn message(role: &str, seq: u64, payload: Value) -> Vec<u8> {
        serde_json::to_vec(
            &json!({"version":1,"token":config().token,"session":config().session,
            "sequence":seq,"kind":role,"payload":payload}),
        )
        .unwrap()
    }
    fn host(enabled: bool) -> Value {
        json!({"enabled":enabled,"movement":[1,0],"jump":false,"colliders":[]})
    }
    #[test]
    fn ownership_freshness_and_disable() {
        let mut r = Relay::new(config()).unwrap();
        let t = Instant::now();
        assert_eq!(
            r.exchange(&message("host", 1, host(true)), "host", t)
                .unwrap()["active"],
            false
        );
        assert_eq!(
            r.exchange(&message("guest", 1, guest()), "guest", t)
                .unwrap()["active"],
            true
        );
        let response = r
            .exchange(&message("host", 2, host(true)), "host", t + LEASE)
            .unwrap();
        assert_eq!(response["active"], false);
        assert!(response["payload"].is_null());
        assert_eq!(
            r.exchange(&message("guest", 2, guest()), "guest", t + LEASE)
                .unwrap()["active"],
            true
        );
        assert_eq!(
            r.exchange(&message("host", 3, host(false)), "host", t + LEASE)
                .unwrap()["active"],
            false
        );
    }
    #[test]
    fn invalid_or_replayed_snapshots_preserve_state() {
        let mut r = Relay::new(config()).unwrap();
        let t = Instant::now();
        r.exchange(&message("guest", 1, guest()), "guest", t)
            .unwrap();
        assert!(r
            .exchange(&message("guest", 1, guest()), "guest", t)
            .is_err());
        let mut bad = guest();
        bad["parts"][0]["back"] = json!([0, 1, 0]);
        assert!(r.exchange(&message("guest", 2, bad), "guest", t).is_err());
        assert!(r
            .exchange(&message("host", 1, host(true)), "guest", t)
            .is_err());
        let valid = r
            .exchange(&message("host", 1, host(true)), "host", t)
            .unwrap();
        assert_eq!(valid["peerSequence"], 1);
        assert_eq!(valid["payload"]["position"], json!([1, 2, 3]));
        let mut bad = host(true);
        bad["movement"] = json!([2, 0]);
        assert!(r.exchange(&message("host", 2, bad), "host", t).is_err());
        assert!(r.exchange(&vec![b'x'; MAX_BYTES + 1], "host", t).is_err());
        assert!(r
            .exchange(&message("guest", MAX_SEQ + 1, guest()), "guest", t)
            .is_err());
    }
    #[test]
    fn lua_empty_table_interoperability_and_duplicate_boxes() {
        let mut r = Relay::new(config()).unwrap();
        let t = Instant::now();
        let mut h = host(false);
        h["colliders"] = json!({});
        assert!(r.exchange(&message("host", 1, h), "host", t).is_ok());
        let mut h = host(true);
        let b = json!({"id":1,"position":[0,0,0],"size":[1,1,1]});
        h["colliders"] = json!([b, b]);
        assert!(r.exchange(&message("host", 2, h), "host", t).is_err());
    }
}
