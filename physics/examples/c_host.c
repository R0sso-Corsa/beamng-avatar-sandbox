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
    AvatarProfileV1 profile;
    assert(avatar_default_profile_v1(&profile) == 0);
    profile.walk_speed = 2;
    avatar = avatar_create_with_profile_v1(&profile, 0, 0, 0.7651);
    assert(avatar && avatar_set_mesh(avatar, &floor, 1) == 0);
    for (int i = 0; i < 240; ++i) assert(avatar_step(avatar, 1, 0, 0) == 0);
    assert(avatar_get_state(avatar, &state) == 0);
    assert(state.position[0] > 1.9 && state.position[0] < 2.01);
    assert(avatar_destroy(avatar) == 0);
    profile.radius = -1;
    assert(avatar_create_with_profile_v1(&profile, 0, 0, 1) == 0);
    assert(avatar_create_with_profile_v1(NULL, 0, 0, 1) == 0);
    assert(avatar_default_profile_v1(NULL) == -2);
    avatar = avatar_create(0, 0, 0.7651);
    assert(avatar);
    AvatarTriangle platform = floor;
    platform.id = 2;
    /* Zero displacement establishes grounding on the initial deck. */
    assert(avatar_step_platform_v1(avatar,0,0,0,&platform,1,0,0,0)==0);
    for (int i=0; i<240; ++i) {
        assert(avatar_step_platform_v1(avatar,0,0,0,&platform,1,0.002,0,0.001)==0);
        for (int v=0; v<3; ++v) { platform.vertices[v][0]+=0.002; platform.vertices[v][2]+=0.001; }
    }
    assert(avatar_get_state(avatar,&state)==0);
    assert(state.grounded && fabs(state.position[0]-0.48)<0.001);
    assert(fabs(state.position[2]-1.0051)<0.001);
    AvatarState before = state;
    assert(avatar_step_platform_v1(avatar,0,0,0,NULL,1,0,0,0)==-2);
    assert(avatar_step_platform_v1(avatar,0,0,0,&platform,1,NAN,0,0)==-2);
    assert(avatar_get_state(avatar,&state)==0 && state.position[0]==before.position[0]);
    assert(avatar_set_mesh(avatar,&platform,1)==0);
    assert(avatar_step_platform_v1(avatar,0,0,0,&platform,1,0,0,0)==-2);
    assert(avatar_set_mesh(avatar,NULL,0)==0);
    assert(avatar_step_platform_v1(avatar,0,0,1,&platform,1,0.002,0,0.001)==0);
    assert(avatar_get_state(avatar,&state)==0 && !state.grounded);
    assert(state.velocity[0]>0.4 && state.velocity[2]>15.8);
    assert(avatar_destroy(avatar)==0);
    assert(avatar_step_platform_v1(avatar,0,0,0,NULL,0,0,0,0)==-1);
    puts("C host: profiles, platform carry/jump, validation and lifecycle passed");
    return 0;
}
