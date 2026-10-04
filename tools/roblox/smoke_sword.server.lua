-- Run in Studio Server during Play; mutates the local test character only.
local c=game.Players:GetPlayers()[1].Character
local m=require(game.ReplicatedStorage.AvatarGearModels)
local dir=Vector3.new(1,0.3,-0.7).Unit
local p=m.projectile('rocket',c:GetPivot().Position+Vector3.new(0,20,0),dir,workspace)
assert(p.CFrame.LookVector:Dot(dir)>.999);p:Destroy()
local t=m.equip(c,'sword');task.wait(.4)
t.AvatarLabActivate:Fire();task.wait(.08)
t.AvatarLabActivate:Fire();task.wait(.3)
assert(t.damage.Value==30 and not t.Enabled,'lunge missing')
assert(t.GripForward:Dot(Vector3.new(0,0,1))>.999,'grip missing')
task.wait(.3)
assert(not c.HumanoidRootPart:FindFirstChildOfClass('BodyVelocity'),'lift not cleaned')
task.wait(.5)
assert(t.Enabled and t.damage.Value==5,'lunge did not reset')
print('SOURCE_SWORD_AND_ROCKET_HEADING_PASS')
