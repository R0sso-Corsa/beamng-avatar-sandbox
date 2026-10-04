-- Local Studio reference lab. Server owns edits/projectiles so clients see them.
local RS=game:GetService('ReplicatedStorage')
local Players=game:GetService('Players')
local Debris=game:GetService('Debris')
local remote=RS:WaitForChild('AvatarLab')
local blocks=workspace:WaitForChild('AvatarLabBlocks')
local last={}
local function spawn(player)
 local c=player.Character
 if c then c:WaitForChild('Humanoid').WalkSpeed=16;c.Humanoid.UseJumpPower=true;c.Humanoid.JumpPower=53;c.Humanoid.MaxSlopeAngle=45 end
end
Players.PlayerAdded:Connect(function(p) p.CharacterAdded:Connect(function() spawn(p) end) end)
Players.PlayerRemoving:Connect(function(p) for key in pairs(last) do if key:match('^'..p.UserId..':') then last[key]=nil end end end)
remote.OnServerEvent:Connect(function(p,action,data)
 local c=p.Character;local root=c and c:FindFirstChild('HumanoidRootPart');if not root then return end
 if action~='reset' and action~='outfit' and action~='avatar' and action~='build' and action~='fire' then return end
 local now=os.clock();local key=p.UserId..':'..tostring(action)
 if now-(last[key] or -100)<0.15 then return end;last[key]=now
 if action=='reset' then p:LoadCharacter();return end
 if action=='outfit' then
  local face=c.Head:FindFirstChildOfClass('Decal')
  if face then face:SetAttribute('OriginalTexture',face:GetAttribute('OriginalTexture') or face.Texture);face.Texture=face.Texture=='' and face:GetAttribute('OriginalTexture') or '' end
  local shirt=c:FindFirstChild('LabShirt')
  if shirt then shirt:Destroy() else shirt=Instance.new('ShirtGraphic');shirt.Name='LabShirt';shirt.Graphic='rbxasset://textures/face.png';shirt.Parent=c end
  return
 end
 if action=='avatar' then
  local green=data=='alternate' and Color3.fromRGB(120,80,220) or Color3.fromRGB(75,151,75)
  for _,part in ipairs(c:GetChildren()) do
   if part:IsA('BasePart') and part.Name:find('Leg') then part.Color=green end
  end
  local hat=c:FindFirstChild('LabHat');if hat then hat:Destroy() end
  if data=='alternate' then
   hat=Instance.new('Part');hat.Name='LabHat';hat.Size=Vector3.new(2.3,0.3,2.3);hat.CanCollide=false;hat.Massless=true
   hat.CFrame=c.Head.CFrame*CFrame.new(0,0.65,0);hat.Color=green;hat.Parent=c
   local weld=Instance.new('WeldConstraint',hat);weld.Part0=c.Head;weld.Part1=hat
  end
  return
 end
 if action=='build' then
  if type(data)~='table' then return end
  local target=data.target
  if data.op=='place' then
   local pos=data.position
   if typeof(pos)~='Vector3' or pos.X~=pos.X or pos.Y~=pos.Y or pos.Z~=pos.Z or (pos-root.Position).Magnitude>40 then return end
   if #blocks:GetChildren()>=100 then return end
   local b=Instance.new('Part');b.Size=Vector3.new(4,4,4);b.Anchored=true;b.Position=pos
   b.Color=Color3.fromRGB(240,180,50);b:SetAttribute('Owner',p.UserId);b.Parent=blocks
  elseif typeof(target)=='Instance' and target.Parent==blocks and target:GetAttribute('Owner')==p.UserId and (target.Position-root.Position).Magnitude<40 then
   if data.op=='delete' then target:Destroy()
   elseif data.op=='move' then target.Position+=Vector3.new(0,2,0)
   elseif data.op=='rotate' then target.CFrame*=CFrame.Angles(0,math.rad(15),0)
   elseif data.op=='resize' then target.Size=Vector3.new(math.min(12,target.Size.X+1),target.Size.Y,target.Size.Z) end
  end
  return
 end
 if action~='fire' or type(data)~='table' then return end
 local gear=data.gear
 local cooldown={rocket=1.5,slingshot=0.25,superball=0.6,bomb=2,sword=0.6,trowel=1,paintball=0.2}
 if not cooldown[gear] then return end
 local firekey=p.UserId..':gear';if now-(last[firekey] or -100)<cooldown[gear] then return end;last[firekey]=now
 local aim=data.position
 if typeof(aim)~='Vector3' or aim.X~=aim.X or aim.Y~=aim.Y or aim.Z~=aim.Z or (aim-root.Position).Magnitude<0.1 or (aim-root.Position).Magnitude>1000 then return end
 if gear=='trowel' then
  if #blocks:GetChildren()>=100 then return end
  local b=Instance.new('Part');b.Size=Vector3.new(8,4,2);b.Anchored=true;b.CFrame=root.CFrame*CFrame.new(0,0,-8)
  b:SetAttribute('Owner',p.UserId);b.Parent=blocks;return
 end
 if gear=='sword' then
  local hit=Instance.new('Part');hit.Anchored=true;hit.CanCollide=false;hit.Size=Vector3.new(1,1,6)
  hit.CFrame=root.CFrame*CFrame.new(0,0,-4);hit.Color=Color3.new(1,1,1);hit.Parent=workspace;Debris:AddItem(hit,0.2);return
 end
 local b=Instance.new('Part');b.Shape=Enum.PartType.Ball;b.Size=Vector3.one*(gear=='bomb' and 2 or 1)
 b.Position=root.Position+Vector3.new(0,2,0)+root.CFrame.LookVector*3;b.Color=Color3.fromRGB(255,80,80);b.Parent=workspace
 b:SetNetworkOwner(nil);b.AssemblyLinearVelocity=(aim-b.Position).Unit*(gear=='rocket' and 100 or 55)
 if gear=='superball' then b.CustomPhysicalProperties=PhysicalProperties.new(1,0.2,0.9) end
 local touched=false
 b.Touched:Connect(function(part)
  if part:IsDescendantOf(c) or touched then return end
  if gear=='paintball' then touched=true;if part:IsDescendantOf(blocks) then part.Color=Color3.fromRGB(80,200,255) end;b:Destroy() end
 end)
 if gear=='rocket' or gear=='bomb' then task.delay(gear=='bomb' and 3 or 1.5,function()
  if b.Parent then local e=Instance.new('Explosion');e.Position=b.Position;e.BlastPressure=0;e.DestroyJointRadiusPercent=0;e.Parent=workspace;b:Destroy() end
 end) end
 Debris:AddItem(b,6)
end)
print('AVATAR_LAB_SERVER_READY')
