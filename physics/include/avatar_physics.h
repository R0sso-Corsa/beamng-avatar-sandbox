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
uint32_t avatar_abi_version(void);
uint64_t avatar_create(double x, double y, double z);
int32_t avatar_destroy(uint64_t handle);
int32_t avatar_set_mesh(uint64_t handle, const AvatarTriangle *triangles, uint32_t count);
int32_t avatar_step(uint64_t handle, double movement_x, double movement_y, uint32_t jump);
int32_t avatar_get_state(uint64_t handle, AvatarState *out);
#ifdef __cplusplus
}
#endif
#endif
