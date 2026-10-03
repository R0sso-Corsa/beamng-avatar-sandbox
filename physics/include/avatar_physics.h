#ifndef AVATAR_PHYSICS_H
#define AVATAR_PHYSICS_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
/* ABI v1: metres, seconds, Z-up; each step advances exactly 1/240 second.
   Status: 0 success, -1 unknown handle, -2 invalid input/step failure,
   -3 unavailable registry. Create returns 0 on failure.
   Memory pointers must be valid, aligned and live throughout the call. */
typedef struct { double position[3]; double velocity[3]; uint32_t grounded; } AvatarState;
typedef struct { uint64_t id; double vertices[3][3]; } AvatarTriangle;
/* Frozen v1 layout. Obtain defaults, then change fields before creation. */
typedef struct {
    double metres_per_stud;
    double gravity;
    double walk_speed;
    double jump_speed;
    double ground_acceleration;
    double air_acceleration;
    double mass;
    double radius;
    double height;
    double max_slope_degrees;
    double step_height;
    double recovery_distance;
    double static_friction;
    double dynamic_friction;
} AvatarProfileV1;
typedef struct {
    double position[3]; double forward[3]; uint32_t first_person;
    double heading; double requested_distance; double actual_distance;
} AvatarCameraPoseV1;
int32_t avatar_camera_active_v1(uint64_t handle, uint32_t active);
int32_t avatar_camera_look_v1(uint64_t handle, double yaw_delta, double pitch_delta);
int32_t avatar_camera_zoom_v1(uint64_t handle, double wheel_steps);
/* obstruction=-1 means clear; otherwise safe boom fraction [0,1]. */
int32_t avatar_camera_pose_v1(uint64_t handle, const double focus[3], const double eye[3],
    double obstruction, AvatarCameraPoseV1 *out);
typedef struct { double min[3]; double max[3]; } AvatarBoundsV1;
int32_t avatar_building_enabled_v1(uint64_t handle, uint32_t enabled);
int32_t avatar_building_clear_v1(uint64_t handle);
/* operations: 0 place (centre), 1 move (offset), 2 resize (size), 3 clone
   (offset), 4 remove (value/player pointers still required, unused).
   Output ID is written on success only. Bounds are conservative world AABBs. */
int32_t avatar_building_edit_v1(uint64_t handle, uint32_t operation, uint64_t id,
    const double value[3], const AvatarBoundsV1 *player, const AvatarBoundsV1 *vehicles,
    uint32_t count, uint64_t *out_id);
/* Read-only preview: does not allocate a part; requires enabled building tools. */
int32_t avatar_building_preview_v1(uint64_t handle, const double centre[3],
    const AvatarBoundsV1 *player, const AvatarBoundsV1 *vehicles, uint32_t count, AvatarBoundsV1 *out);
int32_t avatar_building_get_v1(uint64_t handle, uint64_t id, AvatarBoundsV1 *out);
/* Input IDs: 0 forward, 1 backward, 2 left, 3 right, 4 jump; value [0,1]. */
typedef struct { double movement[2]; uint32_t jump; } AvatarInputV1;
typedef struct { double alpha; double dropped_seconds; double step_seconds; } AvatarDriverStatusV1;
int32_t avatar_driver_control_v1(uint64_t handle, uint32_t index, double value);
int32_t avatar_driver_heading_v1(uint64_t handle, double radians);
int32_t avatar_driver_reset_v1(uint64_t handle);
/* output MUST hold 16 elements. Returns inputs only, does not execute physics.
   Host commits each step atomically, resets driver/releases input on failure.
   Reset on exit/unload; dt=0 clears input and interpolation on pause.
   count/status are written on success only; pointers must be disjoint. */
int32_t avatar_driver_batch_v1(uint64_t handle, double dt, AvatarInputV1 *output,
    uint32_t *count, AvatarDriverStatusV1 *status);
uint32_t avatar_abi_version(void);
int32_t avatar_default_profile_v1(AvatarProfileV1 *out);
uint64_t avatar_create_with_profile_v1(const AvatarProfileV1 *profile, double x, double y, double z);
uint64_t avatar_create(double x, double y, double z);
int32_t avatar_destroy(uint64_t handle);
int32_t avatar_set_mesh(uint64_t handle, const AvatarTriangle *triangles, uint32_t count);
int32_t avatar_step(uint64_t handle, double movement_x, double movement_y, uint32_t jump);
/* Platform triangles describe the previous pose and have IDs disjoint from
   the uploaded static mesh. Advance their pose only after a successful step.
   Carrier translation only; no rotation or continuous moving collision. */
int32_t avatar_step_platform_v1(uint64_t handle, double movement_x, double movement_y,
    uint32_t jump, const AvatarTriangle *platform, uint32_t count,
    double displacement_x, double displacement_y, double displacement_z);
int32_t avatar_get_state(uint64_t handle, AvatarState *out);
#ifdef __cplusplus
}
#endif
#endif
