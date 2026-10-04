-- Diagnostic overlays and reference interactions; never uploads telemetry.
local Players=game:GetService('Players');local player=Players.LocalPlayer
local UIS=game:GetService('UserInputService');local Run=game:GetService('RunService')
local remote=game:GetService('ReplicatedStorage'):WaitForChild('AvatarLab')
local mouse=player:GetMouse();local gear='rocket';local building=false;local op='place';local alternate=false
local gui=Instance.new('ScreenGui');gui.Name='AvatarLab';gui.ResetOnSpawn=false;gui.Parent=player:WaitForChild('PlayerGui')
local panel=Instance.new('Frame');panel.Size=UDim2.fromOffset(360,540);panel.AnchorPoint=Vector2.new(1,0);panel.Position=UDim2.new(1,-12,0,12)
local scale=Instance.new('UIScale',panel);scale.Scale=0.65
panel.BackgroundColor3=Color3.fromRGB(20,25,35);panel.BackgroundTransparency=0.1;panel.Parent=gui
local function text(y,h,value)
 local t=Instance.new('TextLabel');t.Position=UDim2.fromOffset(8,y);t.Size=UDim2.fromOffset(344,h)
 t.BackgroundTransparency=1;t.TextColor3=Color3.new(1,1,1);t.TextSize=15;t.TextWrapped=true;t.Text=value;t.Parent=panel;return t
end
text(0,40,'STUDIO REFERENCE LAB • Roblox runtime')
local status=text(40,100,'Waiting for character');status.TextSize=12
text(140,50,'1–7 gears | B building on/off | Q operation\nClick use/place/edit | R respawn | V avatar')
local function button(y,title,callback)
 local b=Instance.new('TextButton');b.Position=UDim2.fromOffset(8,y);b.Size=UDim2.fromOffset(344,30)
 b.Text=title;b.TextSize=16;b.Parent=panel;b.Activated:Connect(callback)
end
local gears={'sword','slingshot','rocket','bomb','superball','trowel','paintball'}
local index=3;local operations={'place','move','rotate','resize','delete'};local oi=1
button(200,'Cycle gear',function() index=index%#gears+1;gear=gears[index];remote:FireServer('equip',gear) end)
button(235,'Toggle building tools',function() building=not building end)
button(270,'Cycle building operation',function() oi=oi%#operations+1;op=operations[oi] end)
button(305,'Switch avatar / accessory',function() alternate=not alternate;remote:FireServer('avatar',alternate and 'alternate' or 'noob') end)
button(340,'Respawn',function() remote:FireServer('reset') end)
button(375,'Missing avatar → noob fallback',function() remote:FireServer('avatar','missing') end)
button(410,'Toggle face / clothing sample',function() remote:FireServer('outfit') end)
button(445,'Log current measurements',function() print('AVATAR_LAB_MEASUREMENTS',status.Text) end)
local errors=text(480,55,'Local diagnostics: no errors')
game:GetService('ScriptContext').Error:Connect(function(message) errors.Text='Error: '..message:sub(1,100);warn('AVATAR_LAB_ERROR',message) end)
UIS.InputBegan:Connect(function(input,processed)
 if processed then return end
 for i=1,7 do if input.KeyCode==Enum.KeyCode[({'One','Two','Three','Four','Five','Six','Seven'})[i]] then index=i;gear=gears[i];remote:FireServer('equip',gear) end end
 if input.KeyCode==Enum.KeyCode.B then building=not building
 elseif input.KeyCode==Enum.KeyCode.Q then oi=oi%#operations+1;op=operations[oi]
 elseif input.KeyCode==Enum.KeyCode.R then remote:FireServer('reset')
 elseif input.KeyCode==Enum.KeyCode.V then alternate=not alternate;remote:FireServer('avatar',alternate and 'alternate' or 'noob')
 elseif input.UserInputType==Enum.UserInputType.MouseButton1 then
  if building then
   local pos=mouse.Hit.Position+Vector3.new(0,2,0)
   pos=Vector3.new(math.round(pos.X/2)*2,math.round(pos.Y/2)*2,math.round(pos.Z/2)*2)
   remote:FireServer('build',{op=op,position=pos,target=mouse.Target})
  else remote:FireServer('fire',{gear=gear,position=mouse.Hit.Position}) end
 end
end)
local previousSpeed=0;local acceleration=0;local jumpStart;local startY=0;local peak=0;local airTime=0;local height=0
local stopStart;local stopPosition;local stopDistance=0;local lastRoot
Run.RenderStepped:Connect(function(dt)
 local c=player.Character;local h=c and c:FindFirstChildOfClass('Humanoid');local root=c and c:FindFirstChild('HumanoidRootPart')
 if not root or not h then return end
 scale.Scale=math.clamp(workspace.CurrentCamera.ViewportSize.Y/650,0.4,0.85)
 if lastRoot~=root then previousSpeed=0;jumpStart=nil;stopStart=nil;lastRoot=root end
 local speed=Vector3.new(root.AssemblyLinearVelocity.X,0,root.AssemblyLinearVelocity.Z).Magnitude
 acceleration=(speed-previousSpeed)/math.max(dt,0.001);previousSpeed=speed
 if h.FloorMaterial==Enum.Material.Air then
  if not jumpStart then jumpStart=os.clock();startY=root.Position.Y;peak=startY end
  peak=math.max(peak,root.Position.Y)
 elseif jumpStart then airTime=os.clock()-jumpStart;height=peak-startY;jumpStart=nil end
 if h.MoveDirection.Magnitude<0.01 and speed>0.5 and not stopStart then stopStart=os.clock();stopPosition=root.Position end
 if stopStart and speed<0.1 then stopDistance=(root.Position-stopPosition).Magnitude;stopStart=nil end
 if h.MoveDirection.Magnitude>0.1 then stopStart=nil end
 local camera=workspace.CurrentCamera;local distance=(camera.CFrame.Position-camera.Focus.Position).Magnitude
 status.Text=string.format('%s | speed %.2f | accel %.1f studs/s²\nLast jump: %.2f studs / %.2f seconds\nStop distance %.2f | camera %.1f studs\nGear %s | Building %s: %s\nPlayers %d | sample FPS %.0f',h:GetState().Name,speed,acceleration,height,airTime,stopDistance,distance,gear,tostring(building),op,#Players:GetPlayers(),1/math.max(dt,0.001))
end)
print('AVATAR_LAB_CLIENT_READY')
