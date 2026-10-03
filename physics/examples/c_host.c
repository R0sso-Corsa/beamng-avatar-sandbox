/* Compile/link instructions are in docs/CROSS_GAME_SUPPORT.md. */
#include "avatar_physics.h"
#include <assert.h>
#include <math.h>
#include <stdio.h>
int main(void) {
    assert(avatar_abi_version() == 1);
    uint64_t avatar = avatar_create(0, 0, 0.7651);
    assert(avatar);
    AvatarTriangle floor = {1, {{-10,-10,0}, {10,-10,0}, {0,10,0}}};
    assert(avatar_set_mesh(avatar, &floor, 1) == 0);
    for (int i = 0; i < 240; ++i) assert(avatar_step(avatar, 1, 0, 0) == 0);
    AvatarState state;
    assert(avatar_get_state(avatar, &state) == 0);
    assert(state.grounded && state.position[0] > 4 && fabs(state.position[2]-0.765) < 0.002);
    assert(avatar_step(avatar, 0, 0, 1) == 0);
    assert(avatar_get_state(avatar, &state) == 0);
    assert(!state.grounded && state.velocity[2] > 15);
    assert(avatar_destroy(avatar) == 0);
    assert(avatar_step(avatar, 0, 0, 0) == -1);
    puts("C host: walking, grounding, jump and lifecycle passed");
    return 0;
}
