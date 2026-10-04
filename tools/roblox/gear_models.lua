-- Original classic visual references extracted from public place definitions; no source scripts executed.
local M={}
local colors={metal=Color3.fromRGB(180,185,195),dark=Color3.fromRGB(35,40,45),wood=Color3.fromRGB(130,80,40),red=Color3.fromRGB(200,40,40),blue=Color3.fromRGB(40,100,220)}
local function part(model,name,size,offset,color,shape)
 local p=Instance.new('Part');p.Name=name;p.Size=size;p.CFrame=offset;p.Color=colors[color] or color
 p.Shape=shape or Enum.PartType.Block;p.CanCollide=false;p.CanTouch=false;p.Massless=true;p.Parent=model;return p
end
local specs=game:GetService('HttpService'):JSONDecode([=[{"sword":{"size":[1.0,0.8,4.0],"color":6512482,"shape":1,"grip":[0.0,0.0,-1.5,0.0,0.0,1.0,1.0,0.0,0.0,0.0,1.0,0.0],"mesh":"rbxasset://fonts/sword.mesh","texture":"rbxasset://textures/SwordTexture.png","scale":[1.0,1.0,1.0]},"slingshot":{"size":[2.0,2.4,1.0],"color":10724005,"shape":1,"grip":[0.0,-0.7,0.0,1.0,0.0,0.0,0.0,1.0,0.0,0.0,0.0,1.0],"mesh":"rbxasset://fonts/slingshot.mesh","texture":"rbxasset://textures/SlingshotTexture.png","scale":[0.5,0.5,0.5]},"rocket":{"size":[4.0,0.8,1.0],"color":6512482,"shape":1,"grip":[1.0,-0.667,0.25,0.0,0.0,-1.0,0.0,1.0,0.0,1.0,0.0,0.0],"mesh":"rbxasset://fonts/rocketlauncher.mesh","texture":"rbxasset://textures/rocketlaunchertex.png","scale":[0.75,0.75,0.75]},"trowel":{"size":[1.0,4.0,1.0],"color":14140826,"shape":1,"grip":[0.0,-1.3,0.0,1.0,0.0,0.0,0.0,1.0,0.0,0.0,0.0,1.0],"mesh":"rbxasset://fonts/trowel.mesh","texture":"rbxasset://textures/TrowelTexture.png","scale":[1.0,1.0,1.0]},"bomb":{"size":[2.0,2.0,2.0],"color":10724005,"shape":0,"grip":[0.0,0.0,0.0,1.0,0.0,0.0,0.0,0.0,-1.0,0.0,1.0,0.0],"mesh":"rbxasset://fonts/timebomb.mesh","texture":"rbxasset://textures/bombtex.png","scale":[1.0,1.0,1.0]},"superball":{"size":[2.0,2.0,2.0],"color":12855324,"shape":0,"grip":[0.0,0.0,0.0,1.0,0.0,0.0,0.0,1.0,0.0,0.0,0.0,1.0]},"paintball":{"size":[1,3,2],"color":10724005,"shape":1,"grip":[0.0,0.400000006,0.5,1.0,0.0,-0.0,0.0,0.0,1.0,0.0,-1.0,-0.0],"mesh":"rbxasset://fonts/PaintballGun.mesh","texture":"rbxasset://textures/PaintballGunTex128.png","scale":[1,1,1]}}]=])
local function build(name)
 local d=assert(specs[name],'Unknown gear '..tostring(name))
 local model=Instance.new('Model');model.Name='Gear_'..name
 local color=Color3.fromRGB(bit32.band(bit32.rshift(d.color,16),255),bit32.band(bit32.rshift(d.color,8),255),bit32.band(d.color,255))
 local handle=part(model,'Handle',Vector3.new(table.unpack(d.size)),CFrame.new(),color,d.shape==0 and Enum.PartType.Ball or Enum.PartType.Block)
 if d.mesh then
  local mesh=Instance.new('SpecialMesh');mesh.MeshType=Enum.MeshType.FileMesh
  mesh.MeshId=d.mesh;mesh.TextureId=d.texture;mesh.Scale=Vector3.new(table.unpack(d.scale));mesh.Parent=handle
 end
 model.PrimaryPart=handle
 model:SetAttribute('SourceGrip',CFrame.new(table.unpack(d.grip)))
 return model
end
M.build=build
function M.equip(character,name)
 local old=character:FindFirstChild('HeldGear');if old then old:Destroy() end
 local arm=character:FindFirstChild('Right Arm');if not arm then return end
 local model=build(name);model.Name='HeldGear';model:PivotTo(arm.CFrame*CFrame.new(0,-1,0)*CFrame.Angles(math.rad(-90),0,0)*model:GetAttribute('SourceGrip'):Inverse());model.Parent=character
 local grip=Instance.new('Weld');grip.Name='GearGrip';grip.Part0=arm;grip.Part1=model.PrimaryPart
 grip.C0=arm.CFrame:ToObjectSpace(model.PrimaryPart.CFrame);grip.Parent=model
 character:SetAttribute('EquippedGear',name)
 return model
end
function M.projectile(name,position,direction)
 local p=Instance.new('Part');p.Name='Projectile_'..name;p.Size=Vector3.one;p.Shape=Enum.PartType.Ball;p.Color=colors.red
 if name=='bomb' or name=='superball' then
  local visual=build(name);p=visual.PrimaryPart;p.Name='Projectile_'..name;p.Transparency=0;p.Shape=Enum.PartType.Ball
  p.Massless=false;p.CanCollide=true;p.CanTouch=true
  visual:PivotTo(CFrame.new(position));visual.Parent=workspace
  return p,visual
 elseif name=='rocket' then
  p.Size=Vector3.new(2,0.6,0.6);p.Shape=Enum.PartType.Cylinder;p.Color=colors.dark
  p.CFrame=CFrame.lookAt(position,position+direction)*CFrame.Angles(0,math.pi/2,0)
  local mesh=Instance.new('SpecialMesh');mesh.MeshType=Enum.MeshType.FileMesh;mesh.MeshId='rbxassetid://2251534';mesh.Scale=Vector3.new(0.35,0.35,0.25);mesh.Parent=p
 else p.Size=Vector3.one*(name=='slingshot' and 0.35 or 0.45);p.Color=name=='slingshot' and colors.metal or colors.blue;p.Position=position end
 p.Parent=workspace;return p,p
end
return M
