//! C ABI v1. Handles are process-local; calls are serialized by one mutex.
use crate::{
    building::{Bounds, BuildingPlans},
    camera::{Camera, CameraPose},
    mesh::{StaticMesh, Triangle},
    Character, Input, Profile,
};
use std::{
    collections::BTreeMap,
    sync::{Mutex, OnceLock},
};
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct AvatarState {
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub grounded: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AvatarTriangle {
    pub id: u64,
    pub vertices: [[f64; 3]; 3],
}
/// Frozen ABI-v1 profile layout. All lengths/speeds use metres/seconds.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct AvatarProfileV1 {
    pub metres_per_stud: f64,
    pub gravity: f64,
    pub walk_speed: f64,
    pub jump_speed: f64,
    pub ground_acceleration: f64,
    pub air_acceleration: f64,
    pub mass: f64,
    pub radius: f64,
    pub height: f64,
    pub max_slope_degrees: f64,
    pub step_height: f64,
    pub recovery_distance: f64,
    pub static_friction: f64,
    pub dynamic_friction: f64,
}
impl From<Profile> for AvatarProfileV1 {
    fn from(p: Profile) -> Self {
        Self {
            metres_per_stud: p.metres_per_stud,
            gravity: p.gravity,
            walk_speed: p.walk_speed,
            jump_speed: p.jump_speed,
            ground_acceleration: p.ground_acceleration,
            air_acceleration: p.air_acceleration,
            mass: p.mass,
            radius: p.radius,
            height: p.height,
            max_slope_degrees: p.max_slope_degrees,
            step_height: p.step_height,
            recovery_distance: p.recovery_distance,
            static_friction: p.static_friction,
            dynamic_friction: p.dynamic_friction,
        }
    }
}
impl From<AvatarProfileV1> for Profile {
    fn from(p: AvatarProfileV1) -> Self {
        Self {
            metres_per_stud: p.metres_per_stud,
            gravity: p.gravity,
            walk_speed: p.walk_speed,
            jump_speed: p.jump_speed,
            ground_acceleration: p.ground_acceleration,
            air_acceleration: p.air_acceleration,
            mass: p.mass,
            radius: p.radius,
            height: p.height,
            max_slope_degrees: p.max_slope_degrees,
            step_height: p.step_height,
            recovery_distance: p.recovery_distance,
            static_friction: p.static_friction,
            dynamic_friction: p.dynamic_friction,
        }
    }
}
/// # Safety
/// out must point to an aligned writable AvatarProfileV1.
#[no_mangle]
pub unsafe extern "C" fn avatar_default_profile_v1(out: *mut AvatarProfileV1) -> i32 {
    if out.is_null() {
        return -2;
    }
    unsafe {
        out.write(Profile::default().into());
    }
    0
}
/// # Safety
/// profile must point to an aligned readable AvatarProfileV1 throughout this call.
#[no_mangle]
pub unsafe extern "C" fn avatar_create_with_profile_v1(
    profile: *const AvatarProfileV1,
    x: f64,
    y: f64,
    z: f64,
) -> u64 {
    if profile.is_null() {
        return 0;
    }
    create(unsafe { profile.read() }.into(), [x, y, z])
}
struct Instance {
    camera: Camera,
    building: BuildingPlans,
    character: Character,
    mesh: StaticMesh,
}
#[derive(Default)]
struct Registry {
    next: u64,
    instances: BTreeMap<u64, Instance>,
}
// ponytail: serialize instances; use per-instance locks only if profiling warrants it.
fn registry() -> &'static Mutex<Registry> {
    static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(Registry::default()))
}
#[no_mangle]
pub extern "C" fn avatar_abi_version() -> u32 {
    1
}
#[no_mangle]
pub extern "C" fn avatar_create(x: f64, y: f64, z: f64) -> u64 {
    create(Profile::default(), [x, y, z])
}
fn create(profile: Profile, position: [f64; 3]) -> u64 {
    let Ok(character) = Character::new(profile, position) else {
        return 0;
    };
    let Ok(mut r) = registry().lock() else {
        return 0;
    };
    let Some(handle) = r.next.checked_add(1) else {
        return 0;
    };
    r.next = handle;
    r.instances.insert(
        handle,
        Instance {
            camera: Camera::default(),
            building: BuildingPlans::default(),
            character,
            mesh: StaticMesh::new(Vec::new()).unwrap(),
        },
    );
    handle
}
#[no_mangle]
pub extern "C" fn avatar_destroy(handle: u64) -> i32 {
    let Ok(mut r) = registry().lock() else {
        return -3;
    };
    if r.instances.remove(&handle).is_some() {
        0
    } else {
        -1
    }
}
/// Replace an instance's collision mesh atomically. Empty meshes permit free fall.
/// # Safety
/// Nonzero count requires an aligned, readable array of count AvatarTriangles.
/// The caller must keep that memory valid and unmodified throughout this call.
#[no_mangle]
pub unsafe extern "C" fn avatar_set_mesh(
    handle: u64,
    triangles: *const AvatarTriangle,
    count: u32,
) -> i32 {
    if count > 1_000_000 || (count > 0 && triangles.is_null()) {
        return -2;
    }
    let Ok(mut r) = registry().lock() else {
        return -3;
    };
    let Some(instance) = r.instances.get_mut(&handle) else {
        return -1;
    };
    let source = if count == 0 {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(triangles, count as usize) }
    };
    let mesh = source
        .iter()
        .map(|t| Triangle {
            id: t.id,
            vertices: t.vertices,
        })
        .collect();
    let Ok(mesh) = StaticMesh::new(mesh) else {
        return -2;
    };
    instance.mesh = mesh;
    instance.character.contacts.clear();
    0
}
#[no_mangle]
pub extern "C" fn avatar_step(handle: u64, movement_x: f64, movement_y: f64, jump: u32) -> i32 {
    if jump > 1 {
        return -2;
    }
    let Ok(mut r) = registry().lock() else {
        return -3;
    };
    let Some(instance) = r.instances.get_mut(&handle) else {
        return -1;
    };
    match instance.character.step_static_mesh(
        Input {
            movement: [movement_x, movement_y],
            jump: jump == 1,
        },
        &instance.mesh,
    ) {
        Ok(_) => 0,
        Err(_) => -2,
    }
}
/// Step against the uploaded static mesh and one prescribed translating platform.
/// # Safety
/// Nonzero count requires an aligned readable array of count AvatarTriangles,
/// valid and unmodified throughout the call. Triangles describe the OLD pose.
#[no_mangle]
pub unsafe extern "C" fn avatar_step_platform_v1(
    handle: u64,
    movement_x: f64,
    movement_y: f64,
    jump: u32,
    platform: *const AvatarTriangle,
    count: u32,
    displacement_x: f64,
    displacement_y: f64,
    displacement_z: f64,
) -> i32 {
    if jump > 1 || count > 1_000_000 || (count > 0 && platform.is_null()) {
        return -2;
    }
    let Ok(mut r) = registry().lock() else {
        return -3;
    };
    let Some(instance) = r.instances.get_mut(&handle) else {
        return -1;
    };
    let source = if count == 0 {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(platform, count as usize) }
    };
    let triangles: Vec<Triangle> = source
        .iter()
        .map(|t| Triangle {
            id: t.id,
            vertices: t.vertices,
        })
        .collect();
    match instance.character.step_translating_platform(
        Input {
            movement: [movement_x, movement_y],
            jump: jump == 1,
        },
        instance.mesh.triangles(),
        &triangles,
        [displacement_x, displacement_y, displacement_z],
    ) {
        Ok(_) => 0,
        Err(_) => -2,
    }
}
#[no_mangle]
pub extern "C" fn avatar_camera_active_v1(handle: u64, active: u32) -> i32 {
    if active > 1 {
        return -2;
    }
    let Ok(mut r) = registry().lock() else {
        return -3;
    };
    let Some(i) = r.instances.get_mut(&handle) else {
        return -1;
    };
    i.camera.set_active(active == 1);
    0
}
#[no_mangle]
pub extern "C" fn avatar_camera_look_v1(handle: u64, yaw: f64, pitch: f64) -> i32 {
    let Ok(mut r) = registry().lock() else {
        return -3;
    };
    let Some(i) = r.instances.get_mut(&handle) else {
        return -1;
    };
    if i.camera.look(yaw, pitch).is_ok() {
        0
    } else {
        -2
    }
}
#[no_mangle]
pub extern "C" fn avatar_camera_zoom_v1(handle: u64, steps: f64) -> i32 {
    let Ok(mut r) = registry().lock() else {
        return -3;
    };
    let Some(i) = r.instances.get_mut(&handle) else {
        return -1;
    };
    if i.camera.zoom(steps).is_ok() {
        0
    } else {
        -2
    }
}
/// # Safety
/// focus/eye must each point to three readable doubles; out must point to one
/// writable aligned CameraPose. All memory stays valid throughout this call.
#[no_mangle]
pub unsafe extern "C" fn avatar_camera_pose_v1(
    handle: u64,
    focus: *const f64,
    eye: *const f64,
    obstruction: f64,
    out: *mut CameraPose,
) -> i32 {
    if focus.is_null() || eye.is_null() || out.is_null() {
        return -2;
    }
    let Ok(r) = registry().lock() else {
        return -3;
    };
    let Some(i) = r.instances.get(&handle) else {
        return -1;
    };
    let focus = unsafe { std::ptr::read(focus.cast::<[f64; 3]>()) };
    let eye = unsafe { std::ptr::read(eye.cast::<[f64; 3]>()) };
    let hit = if obstruction == -1.0 {
        None
    } else {
        Some(obstruction)
    };
    let Ok(pose) = i.camera.pose(focus, eye, hit) else {
        return -2;
    };
    unsafe {
        out.write(pose);
    }
    0
}
#[no_mangle]
pub extern "C" fn avatar_building_enabled_v1(handle: u64, enabled: u32) -> i32 {
    if enabled > 1 {
        return -2;
    }
    let Ok(mut r) = registry().lock() else {
        return -3;
    };
    let Some(i) = r.instances.get_mut(&handle) else {
        return -1;
    };
    i.building.set_enabled(enabled == 1);
    0
}
#[no_mangle]
pub extern "C" fn avatar_building_clear_v1(handle: u64) -> i32 {
    let Ok(mut r) = registry().lock() else {
        return -3;
    };
    let Some(i) = r.instances.get_mut(&handle) else {
        return -1;
    };
    i.building.clear();
    0
}
/// # Safety
/// value is three readable doubles; player one readable Bounds; vehicles count
/// readable Bounds (null allowed for zero); out_id one writable u64.
#[no_mangle]
pub unsafe extern "C" fn avatar_building_edit_v1(
    handle: u64,
    operation: u32,
    id: u64,
    value: *const f64,
    player: *const Bounds,
    vehicles: *const Bounds,
    count: u32,
    out_id: *mut u64,
) -> i32 {
    if operation > 4
        || value.is_null()
        || player.is_null()
        || out_id.is_null()
        || count > 10000
        || (count > 0 && vehicles.is_null())
    {
        return -2;
    }
    let Ok(mut r) = registry().lock() else {
        return -3;
    };
    let Some(i) = r.instances.get_mut(&handle) else {
        return -1;
    };
    let value = unsafe { value.cast::<[f64; 3]>().read() };
    let player = unsafe { player.read() };
    let vehicles = if count == 0 {
        &[][..]
    } else {
        unsafe { std::slice::from_raw_parts(vehicles, count as usize) }
    };
    let result = match operation {
        0 => i.building.place(value, player, vehicles),
        1 => i
            .building
            .move_part(id, value, player, vehicles)
            .map(|_| id),
        2 => i.building.resize(id, value, player, vehicles).map(|_| id),
        3 => i.building.clone_part(id, value, player, vehicles),
        _ => i.building.remove(id).map(|_| id),
    };
    let Ok(id) = result else {
        return -2;
    };
    unsafe {
        out_id.write(id);
    }
    0
}
/// # Safety
/// out points to one aligned writable Bounds.
#[no_mangle]
pub unsafe extern "C" fn avatar_building_get_v1(handle: u64, id: u64, out: *mut Bounds) -> i32 {
    if out.is_null() {
        return -2;
    }
    let Ok(r) = registry().lock() else {
        return -3;
    };
    let Some(i) = r.instances.get(&handle) else {
        return -1;
    };
    let Some(b) = i.building.get(id) else {
        return -2;
    };
    unsafe {
        out.write(b);
    }
    0
}
/// # Safety
/// out must point to one aligned, writable AvatarState for the duration of the call.
#[no_mangle]
pub unsafe extern "C" fn avatar_get_state(handle: u64, out: *mut AvatarState) -> i32 {
    if out.is_null() {
        return -2;
    }
    let Ok(r) = registry().lock() else {
        return -3;
    };
    let Some(instance) = r.instances.get(&handle) else {
        return -1;
    };
    let s = instance.character.state();
    unsafe {
        out.write(AvatarState {
            position: s.position,
            velocity: s.velocity,
            grounded: u32::from(s.grounded),
        });
    }
    0
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lifecycle_mesh_validation_and_step_rollback() {
        assert_eq!(avatar_abi_version(), 1);
        assert_eq!(avatar_create(f64::NAN, 0.0, 1.0), 0);
        let h = avatar_create(0.0, 0.0, 0.7651);
        assert_ne!(h, 0);
        let tri = AvatarTriangle {
            id: 1,
            vertices: [[-10.0, -10.0, 0.0], [10.0, -10.0, 0.0], [0.0, 10.0, 0.0]],
        };
        assert_eq!(unsafe { avatar_set_mesh(h, &tri, 1) }, 0);
        assert_eq!(unsafe { avatar_set_mesh(h, std::ptr::null(), 1) }, -2);
        assert_eq!(unsafe { avatar_set_mesh(h, [tri, tri].as_ptr(), 2) }, -2);
        for _ in 0..240 {
            assert_eq!(avatar_step(h, 0.0, 0.0, 0), 0);
        }
        let mut state = AvatarState {
            position: [0.0; 3],
            velocity: [0.0; 3],
            grounded: 0,
        };
        assert_eq!(unsafe { avatar_get_state(h, &mut state) }, 0);
        assert_eq!(state.grounded, 1);
        let before = state.position;
        assert_eq!(avatar_step(h, f64::NAN, 0.0, 0), -2);
        assert_eq!(unsafe { avatar_get_state(h, &mut state) }, 0);
        assert_eq!(state.position, before);
        assert_eq!(avatar_destroy(h), 0);
        assert_eq!(avatar_destroy(h), -1);
        assert_eq!(avatar_step(h, 0.0, 0.0, 0), -1);
    }
}
