-- Client Command Bar during Play: tests server-owned edits and avatar fallback.
local p=game.Players.LocalPlayer
local c=p.Character;local root=c.HumanoidRootPart
local remote=game.ReplicatedStorage.AvatarLab
local before=#workspace.AvatarLabBlocks:GetChildren()
remote:FireServer('build',{op='place',position=root.Position+Vector3.new(10,0,0)})
task.wait(0.4)
assert(#workspace.AvatarLabBlocks:GetChildren()==before+1,'Place did not replicate')
local block=workspace.AvatarLabBlocks:GetChildren()[before+1]
assert(block:GetAttribute('Owner')==p.UserId,'Owner not replicated')
for _,op in ipairs({'move','rotate','resize'}) do remote:FireServer('build',{op=op,target=block});task.wait(0.2) end
assert(block.Size.X==5,'Resize failed')
remote:FireServer('avatar','alternate');task.wait(0.3);assert(c:FindFirstChild('LabHat'),'Accessory not replicated')
remote:FireServer('avatar','missing');task.wait(0.3);assert(not c:FindFirstChild('LabHat'),'Fallback failed')
remote:FireServer('build',{op='delete',target=block});task.wait(0.3)
assert(#workspace.AvatarLabBlocks:GetChildren()==before,'Delete failed')
print('AVATAR_LAB_SMOKE_PASS: place/move/rotate/resize/delete/accessory/fallback')
