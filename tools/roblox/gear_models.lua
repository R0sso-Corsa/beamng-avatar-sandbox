-- Independently authored primitive Doomspire-style reference models.
local M={}
local colors={metal=Color3.fromRGB(180,185,195),dark=Color3.fromRGB(35,40,45),wood=Color3.fromRGB(130,80,40),red=Color3.fromRGB(200,40,40),blue=Color3.fromRGB(40,100,220)}
local function part(model,name,size,offset,color,shape)
 local p=Instance.new('Part');p.Name=name;p.Size=size;p.CFrame=offset;p.Color=colors[color] or color
 p.Shape=shape or Enum.PartType.Block;p.CanCollide=false;p.CanTouch=false;p.Massless=true;p.Parent=model;return p
end
local function build(name)
 local m=Instance.new('Model');m.Name='Gear_'..name
 local function add(n,x,y,z,px,py,pz,color,shape)
  return part(m,n,Vector3.new(x,y,z),CFrame.new(px,py,pz),color,shape)
 end
 local handle=add('Handle',0.3,1,0.3,0,0,0,'dark');m.PrimaryPart=handle
 if name=='sword' then
  handle.Color=colors.wood;add('Guard',1.4,0.18,0.35,0,0.55,0,'dark')
  add('Blade',0.5,3.4,0.12,0,2.25,0,'metal');add('Tip',0.25,0.4,0.12,0,4.1,0,'metal')
 elseif name=='slingshot' then
  handle.Color=colors.wood
  for _,x in ipairs({-0.55,0.55}) do add('Fork',0.22,1.1,0.22,x,0.85,0,'wood') end
  add('Crosspiece',1.3,0.22,0.22,0,0.4,0,'wood');add('Elastic',1.1,0.06,0.06,0,1.35,0,'dark')
 elseif name=='rocket' then
  local tube=add('LauncherTube',2.9,0.9,0.9,0,0.65,-0.7,'dark',Enum.PartType.Cylinder)
  tube.CFrame=CFrame.new(0,0.65,-0.7)*CFrame.Angles(0,math.pi/2,0)
  add('Sight',0.15,0.45,0.15,0,1.3,-0.8,'metal');add('ShoulderRest',0.8,0.7,0.25,0,0.7,0.85,'wood')
 elseif name=='trowel' then
  handle.Color=colors.wood;add('Neck',0.18,0.6,0.18,0,0.8,0,'metal')
  add('Spade',1.1,0.85,0.12,0,1.5,0,'metal')
 elseif name=='bomb' then
  handle.Transparency=1;add('Bomb',1.5,1.5,1.5,0,0.3,0,'dark',Enum.PartType.Ball)
  add('Fuse',0.12,0.55,0.12,0,1.2,0,'wood');add('Spark',0.18,0.18,0.18,0,1.5,0,Color3.fromRGB(255,180,30),Enum.PartType.Ball)
 elseif name=='superball' then
  handle.Transparency=1;add('Ball',1.4,1.4,1.4,0,0.25,0,'blue',Enum.PartType.Ball)
 elseif name=='paintball' then
  add('Body',0.55,0.65,1.6,0,0.65,-0.4,'dark')
  add('Barrel',0.2,0.2,1.2,0,0.7,-1.65,'metal')
  add('Hopper',0.9,0.8,0.9,0,1.35,-0.4,'blue',Enum.PartType.Ball)
  add('Tank',0.6,0.6,0.8,0,0.65,0.8,'metal')
 else error('Unknown gear '..tostring(name)) end
 for _,p in ipairs(m:GetChildren()) do
  if p~=handle then local w=Instance.new('WeldConstraint',p);w.Part0=handle;w.Part1=p end
 end
 return m
end
M.build=build
function M.equip(character,name)
 local old=character:FindFirstChild('HeldGear');if old then old:Destroy() end
 local arm=character:FindFirstChild('Right Arm');if not arm then return end
 local model=build(name);model.Name='HeldGear';model:PivotTo(arm.CFrame*CFrame.new(0,-1,0)*CFrame.Angles(math.rad(-90),0,0));model.Parent=character
 local grip=Instance.new('Weld');grip.Name='GearGrip';grip.Part0=arm;grip.Part1=model.PrimaryPart
 grip.C0=arm.CFrame:ToObjectSpace(model.PrimaryPart.CFrame);grip.Parent=model
 character:SetAttribute('EquippedGear',name)
 return model
end
function M.projectile(name,position,direction)
 local p=Instance.new('Part');p.Name='Projectile_'..name;p.Size=Vector3.one;p.Shape=Enum.PartType.Ball;p.Color=colors.red
 if name=='bomb' or name=='superball' then
  local visual=build(name);p=visual.PrimaryPart;p.Name='Projectile_'..name;p.Transparency=0;p.Shape=Enum.PartType.Ball
  p.Size=Vector3.one*1.5;p.Color=name=='bomb' and colors.dark or colors.blue;p.Massless=false;p.CanCollide=true;p.CanTouch=true
  visual:PivotTo(CFrame.new(position));visual.Parent=workspace
  return p,visual
 elseif name=='rocket' then
  p.Size=Vector3.new(2,0.6,0.6);p.Shape=Enum.PartType.Cylinder;p.Color=colors.dark
  p.CFrame=CFrame.lookAt(position,position+direction)*CFrame.Angles(0,math.pi/2,0)
  local fin=part(p,'Fins',Vector3.new(0.15,1,1),p.CFrame*CFrame.new(-0.7,0,0),'red')
  local w=Instance.new('WeldConstraint',fin);w.Part0=p;w.Part1=fin
 else p.Size=Vector3.one*(name=='slingshot' and 0.35 or 0.45);p.Color=name=='slingshot' and colors.metal or colors.blue;p.Position=position end
 p.Parent=workspace;return p,p
end
return M
