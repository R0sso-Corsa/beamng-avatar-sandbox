//! Additive ABI v1 entry points. Pointer validity remains the caller's contract.
use super::*;
use crate::{
    animation::{Interpolation, Key, Track},
    animation_state::Motion,
    catalogue::{Client, Server, Snapshot},
};
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AvatarKeyV1 {
    pub time: f64,
    pub pose: AvatarPoseV1,
    pub interpolation: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AvatarMotionV1 {
    pub active: u32,
    pub grounded: u32,
    pub jumped: u32,
    pub climbing: u32,
    pub speed: f64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AvatarSelectionV1 {
    pub active: u32,
    pub clip: u32,
    pub changed: u32,
    pub time: f64,
    pub rate: f64,
    pub blend_seconds: f64,
}
/// # Safety
/// keys references count aligned readable elements; count in 1..10000.
#[no_mangle]
pub unsafe extern "C" fn avatar_track_upload_v1(
    handle: u64,
    id: u64,
    keys: *const AvatarKeyV1,
    count: u32,
) -> i32 {
    if keys.is_null() || count == 0 || count > 10000 {
        return -2;
    }
    let mut converted = Vec::with_capacity(count as usize);
    for k in unsafe { std::slice::from_raw_parts(keys, count as usize) } {
        let interpolation = match k.interpolation {
            0 => Interpolation::Linear,
            1 => Interpolation::Hold,
            _ => return -2,
        };
        converted.push(Key {
            time: k.time,
            pose: k.pose.into(),
            interpolation,
        });
    }
    let Ok(track) = Track::new(converted) else {
        return -2;
    };
    let Ok(mut r) = registry().lock() else {
        return -3;
    };
    let Some(i) = r.instances.get_mut(&handle) else {
        return -1;
    };
    if !i.tracks.contains_key(&id) && i.tracks.len() >= 64 {
        return -2;
    }
    i.tracks.insert(id, track);
    0
}
/// # Safety
/// out references one writable aligned pose. duration=0 means nonlooping.
#[no_mangle]
pub unsafe extern "C" fn avatar_track_sample_v1(
    handle: u64,
    id: u64,
    time: f64,
    duration: f64,
    out: *mut AvatarPoseV1,
) -> i32 {
    if out.is_null() {
        return -2;
    }
    let Ok(r) = registry().lock() else {
        return -3;
    };
    let Some(i) = r.instances.get(&handle) else {
        return -1;
    };
    let Some(track) = i.tracks.get(&id) else {
        return -2;
    };
    let Ok(p) = track.sample(
        time,
        if duration == 0.0 {
            None
        } else {
            Some(duration)
        },
    ) else {
        return -2;
    };
    unsafe {
        out.write(p.into());
    }
    0
}
#[no_mangle]
pub extern "C" fn avatar_animation_reset_v1(handle: u64) -> i32 {
    let Ok(mut r) = registry().lock() else {
        return -3;
    };
    let Some(i) = r.instances.get_mut(&handle) else {
        return -1;
    };
    i.animator.reset();
    i.tracks.clear();
    0
}
/// # Safety
/// motion readable and out writable aligned objects.
#[no_mangle]
pub unsafe extern "C" fn avatar_animation_update_v1(
    handle: u64,
    dt: f64,
    motion: *const AvatarMotionV1,
    out: *mut AvatarSelectionV1,
) -> i32 {
    if motion.is_null() || out.is_null() {
        return -2;
    }
    let m = unsafe { motion.read() };
    if [m.active, m.grounded, m.jumped, m.climbing]
        .iter()
        .any(|v| *v > 1)
    {
        return -2;
    }
    let Ok(mut r) = registry().lock() else {
        return -3;
    };
    let Some(i) = r.instances.get_mut(&handle) else {
        return -1;
    };
    let Ok(selection) = i.animator.update(
        dt,
        Motion {
            active: m.active == 1,
            grounded: m.grounded == 1,
            jumped: m.jumped == 1,
            climbing: m.climbing == 1,
            speed: m.speed,
        },
    ) else {
        return -2;
    };
    let output = match selection {
        Some(s) => AvatarSelectionV1 {
            active: 1,
            clip: s.clip as u32,
            changed: s.changed as u32,
            time: s.time,
            rate: s.rate,
            blend_seconds: s.blend_seconds,
        },
        None => AvatarSelectionV1 {
            active: 0,
            clip: 0,
            changed: 0,
            time: 0.0,
            rate: 0.0,
            blend_seconds: 0.0,
        },
    };
    unsafe {
        out.write(output);
    }
    0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AvatarIdV1 {
    pub bytes: [u8; 65],
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AvatarAssignmentV1 {
    pub player: u32,
    pub avatar: AvatarIdV1,
}
fn decode(id: AvatarIdV1) -> Result<String, ()> {
    let end = id.bytes.iter().position(|v| *v == 0).ok_or(())?;
    if id.bytes[end..].iter().any(|v| *v != 0) {
        return Err(());
    }
    let s = std::str::from_utf8(&id.bytes[..end]).map_err(|_| ())?;
    if s != "noob"
        && !(s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
    {
        return Err(());
    }
    Ok(s.to_owned())
}
fn encode(s: &str) -> AvatarIdV1 {
    let mut bytes = [0; 65];
    bytes[..s.len()].copy_from_slice(s.as_bytes());
    AvatarIdV1 { bytes }
}
unsafe fn ids(pointer: *const AvatarIdV1, count: u32) -> Result<Vec<String>, ()> {
    if count > 256 || (count > 0 && pointer.is_null()) {
        return Err(());
    }
    if count == 0 {
        return Ok(vec![]);
    }
    unsafe { std::slice::from_raw_parts(pointer, count as usize) }
        .iter()
        .map(|v| decode(*v))
        .collect()
}
/// # Safety
/// Arrays reference their respective counts of readable aligned IDs (null if 0).
/// Replaces server/client catalogues and resets sessions; use only at session setup.
#[no_mangle]
pub unsafe extern "C" fn avatar_catalogue_configure_v1(
    handle: u64,
    approved: *const AvatarIdV1,
    approved_count: u32,
    installed: *const AvatarIdV1,
    installed_count: u32,
) -> i32 {
    let Ok(approved) = (unsafe { ids(approved, approved_count) }) else {
        return -2;
    };
    let Ok(installed) = (unsafe { ids(installed, installed_count) }) else {
        return -2;
    };
    let Ok(server) = Server::new(&approved) else {
        return -2;
    };
    let Ok(client) = Client::new(&installed) else {
        return -2;
    };
    let Ok(mut r) = registry().lock() else {
        return -3;
    };
    let Some(i) = r.instances.get_mut(&handle) else {
        return -1;
    };
    i.catalogue_server = server;
    i.catalogue_client = client;
    0
}
/// # Safety
/// avatar points to a readable ID for select (operation 1); ignored otherwise.
/// Host must derive player from authenticated connection context, never payload.
#[no_mangle]
pub unsafe extern "C" fn avatar_catalogue_change_v1(
    handle: u64,
    operation: u32,
    player: u32,
    avatar: *const AvatarIdV1,
) -> i32 {
    let selection = if operation == 1 {
        if avatar.is_null() {
            return -2;
        }
        let Ok(s) = decode(unsafe { avatar.read() }) else {
            return -2;
        };
        Some(s)
    } else {
        None
    };
    let Ok(mut r) = registry().lock() else {
        return -3;
    };
    let Some(i) = r.instances.get_mut(&handle) else {
        return -1;
    };
    let result = match operation {
        0 => i.catalogue_server.join(player),
        1 => i
            .catalogue_server
            .select(player, selection.as_ref().unwrap()),
        2 => i.catalogue_server.leave(player),
        _ => return -2,
    };
    if result.is_ok() {
        0
    } else {
        -2
    }
}
/// # Safety
/// output MUST hold 256 writable assignments; count/revision writable disjoint.
#[no_mangle]
pub unsafe extern "C" fn avatar_catalogue_snapshot_v1(
    handle: u64,
    output: *mut AvatarAssignmentV1,
    count: *mut u32,
    revision: *mut u64,
) -> i32 {
    if output.is_null() || count.is_null() || revision.is_null() {
        return -2;
    }
    let Ok(r) = registry().lock() else {
        return -3;
    };
    let Some(i) = r.instances.get(&handle) else {
        return -1;
    };
    let s = i.catalogue_server.snapshot();
    for (n, (p, a)) in s.assignments.iter().enumerate() {
        unsafe {
            output.add(n).write(AvatarAssignmentV1 {
                player: *p,
                avatar: encode(a),
            });
        }
    }
    unsafe {
        count.write(s.assignments.len() as u32);
        revision.write(s.revision);
    }
    0
}
/// # Safety
/// assignments references count readable records; host authenticates server source.
#[no_mangle]
pub unsafe extern "C" fn avatar_catalogue_apply_v1(
    handle: u64,
    version: u32,
    revision: u64,
    assignments: *const AvatarAssignmentV1,
    count: u32,
) -> i32 {
    if count > 256 || (count > 0 && assignments.is_null()) {
        return -2;
    }
    let records = if count == 0 {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(assignments, count as usize) }
    };
    let mut converted = vec![];
    for a in records {
        let Ok(id) = decode(a.avatar) else {
            return -2;
        };
        converted.push((a.player, id));
    }
    let Ok(mut r) = registry().lock() else {
        return -3;
    };
    let Some(i) = r.instances.get_mut(&handle) else {
        return -1;
    };
    if i.catalogue_client
        .apply(&Snapshot {
            version,
            revision,
            assignments: converted,
        })
        .is_ok()
    {
        0
    } else {
        -2
    }
}
/// # Safety
/// requested/resolved are disjoint writable aligned ID objects.
#[no_mangle]
pub unsafe extern "C" fn avatar_catalogue_get_v1(
    handle: u64,
    player: u32,
    requested: *mut AvatarIdV1,
    resolved: *mut AvatarIdV1,
) -> i32 {
    if requested.is_null() || resolved.is_null() {
        return -2;
    }
    let Ok(r) = registry().lock() else {
        return -3;
    };
    let Some(i) = r.instances.get(&handle) else {
        return -1;
    };
    let Some((a, b)) = i.catalogue_client.get(player) else {
        return -2;
    };
    unsafe {
        requested.write(encode(a));
        resolved.write(encode(b));
    }
    0
}
#[no_mangle]
pub extern "C" fn avatar_catalogue_client_reset_v1(handle: u64) -> i32 {
    let Ok(mut r) = registry().lock() else {
        return -3;
    };
    let Some(i) = r.instances.get_mut(&handle) else {
        return -1;
    };
    i.catalogue_client.reset();
    0
}
