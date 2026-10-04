-- Command Bar installer: supply SERVER_SOURCE and LAB_CLIENT_SOURCE strings.
local RS=game:GetService('ReplicatedStorage')
local remote=RS:FindFirstChild('AvatarLab') or Instance.new('RemoteEvent');remote.Name='AvatarLab';remote.Parent=RS
local scripts=game:GetService('ServerScriptService')
local old=scripts:FindFirstChild('AvatarLab');if old then old:Destroy() end
local server=Instance.new('Script');server.Name='AvatarLab';server.Source=SERVER_SOURCE;server.Parent=scripts
local clients=game:GetService('StarterPlayer').StarterPlayerScripts
old=clients:FindFirstChild('AvatarLab');if old then old:Destroy() end
local client=Instance.new('LocalScript');client.Name='AvatarLab';client.Source=LAB_CLIENT_SOURCE;client.Parent=clients
local blocks=workspace:FindFirstChild('AvatarLabBlocks') or Instance.new('Folder');blocks.Name='AvatarLabBlocks';blocks.Parent=workspace
local course=workspace.AvatarFeelCourse
local floor=workspace.Baseplate.Position.Y+workspace.Baseplate.Size.Y/2
for _,name in ipairs({'Ramp','Climb','CameraCorridor','AvatarPreview'}) do old=course:FindFirstChild(name);if old then old:Destroy() end end
local ramp=Instance.new('WedgePart');ramp.Name='Ramp';ramp.Anchored=true;ramp.Size=Vector3.new(12,10,20)
ramp.Position=Vector3.new(35,floor+5,-25);ramp.Parent=course
local truss=Instance.new('TrussPart');truss.Name='Climb';truss.Anchored=true;truss.Size=Vector3.new(2,20,2)
truss.Position=Vector3.new(-10,floor+10,-20);truss.Parent=course
local corridor=Instance.new('Folder');corridor.Name='CameraCorridor';corridor.Parent=course
for _,x in ipairs({-6,6}) do
 local wall=Instance.new('Part');wall.Anchored=true;wall.Size=Vector3.new(2,15,25)
 wall.Position=Vector3.new(x,floor+7.5,40);wall.Parent=corridor
end
local preview=game.StarterPlayer.StarterCharacter:Clone();preview.Name='AvatarPreview';preview.Parent=course
preview:PivotTo(CFrame.new(-20,floor+3,5))
for _,v in ipairs(preview:GetDescendants()) do if v:IsA('BasePart') then v.Anchored=true end end
print('AVATAR_LAB_INSTALLED: native multiplayer replication, no Rust/BeamMP claim')
