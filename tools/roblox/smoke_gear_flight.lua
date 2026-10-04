-- Server Command Bar during Play. Isolated flight test, cleans up its objects.
local M=require(game.ReplicatedStorage.AvatarGearModels)
local owner=Instance.new('Folder');owner.Name='RocketFlightTest';owner.Parent=workspace
local wall=Instance.new('Part');wall.Anchored=true;wall.Size=Vector3.new(10,10,0.2)
wall.Position=Vector3.new(120,100,70);wall.Parent=owner
local p,container=M.projectile('rocket',Vector3.new(120,100,100),Vector3.new(0,0,-1))
local impacted
-- Only exclude a dummy owner, not the target wall.
local dummy=Instance.new('Folder');dummy.Parent=owner
M.flyRocket(p,container,Vector3.new(0,0,-1),dummy,function(pos) impacted=pos end)
task.wait(0.2)
local distance=100-p.Position.Z
assert(p.Anchored and math.abs(p.Position.Y-100)<0.001,'Rocket dropped')
assert(distance>8 and distance<20,'Rocket speed outside frame tolerance')
task.wait(0.5)
assert(impacted and math.abs(impacted.Z-70.1)<0.01,'Thin-wall sweep failed')
assert(container.Parent==nil,'Rocket was not removed after impact')
owner:Destroy()
print('ROCKET_FLIGHT_PASS',distance,'studs before impact; no gravity sag; thin wall hit')
