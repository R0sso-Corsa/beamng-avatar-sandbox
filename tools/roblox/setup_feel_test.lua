-- Run in Studio Command Bar after replacing CLIENT_SOURCE with the client file.
local starter=game:GetService('StarterPlayer')
local old=starter:FindFirstChild('StarterCharacter'); if old then old:Destroy() end
local description=Instance.new('HumanoidDescription')
local yellow=Color3.fromRGB(245,205,48)
description.HeadColor=yellow; description.LeftArmColor=yellow; description.RightArmColor=yellow
description.TorsoColor=Color3.fromRGB(13,105,172)
description.LeftLegColor=Color3.fromRGB(75,151,75); description.RightLegColor=description.LeftLegColor
local rig=game:GetService('Players'):CreateHumanoidModelFromDescription(description,Enum.HumanoidRigType.R6)
rig.Name='StarterCharacter'; rig.Parent=starter
for _,part in ipairs(rig:GetDescendants()) do if part:IsA('BasePart') then part.Anchored=false end end
local reference=workspace:FindFirstChild('R6Reference'); if reference then reference.Parent=game.ServerStorage end
workspace.Gravity=196.2
local scripts=starter.StarterPlayerScripts
local previous=scripts:FindFirstChild('AvatarFeelTest'); if previous then previous:Destroy() end
local client=Instance.new('LocalScript'); client.Name='AvatarFeelTest'; client.Source=CLIENT_SOURCE; client.Parent=scripts
local course=workspace:FindFirstChild('AvatarFeelCourse'); if course then course:Destroy() end
course=Instance.new('Folder'); course.Name='AvatarFeelCourse'; course.Parent=workspace
local floor=workspace.Baseplate.Position.Y+workspace.Baseplate.Size.Y/2
for i,height in ipairs({0.5,1,2,3,4}) do
 local part=Instance.new('Part'); part.Name='Step_'..height; part.Anchored=true
 part.Size=Vector3.new(8,height,8); part.Position=Vector3.new(15,floor+height/2,-i*12)
 part.Color=Color3.fromRGB(80+i*25,140,200); part.Parent=course
end
local wall=Instance.new('Part'); wall.Name='CollisionWall'; wall.Anchored=true
wall.Size=Vector3.new(30,12,2); wall.Position=Vector3.new(-25,floor+6,-30); wall.Parent=course
print('AVATAR_FEEL_TEST_INSTALLED: save locally, then Play')
