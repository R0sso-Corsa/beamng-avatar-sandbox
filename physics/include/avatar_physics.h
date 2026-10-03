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
uint32_t avatar_abi_version(void);
int32_t avatar_default_profile_v1(AvatarProfileV1 *out);
uint64_t avatar_create_with_profile_v1(const AvatarProfileV1 *profile, double x, double y, double z);
uint64_t avatar_create(double x, double y, double z);
int32_t avatar_destroy(uint64_t handle);
int32_t avatar_set_mesh(uint64_t handle, const AvatarTriangle *triangles, uint32_t count);
int32_t avatar_step(uint64_t handle, double movement_x, double movement_y, uint32_t jump);
int32_t avatar_get_state(uint64_t handle, AvatarState *out);
#ifdef __cplusplus
}
#endif
#endif
