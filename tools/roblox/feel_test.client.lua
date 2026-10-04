-- Native Studio reference: matches profile speeds, not the Rust collision solver.
local player = game:GetService('Players').LocalPlayer
player.CameraMode = Enum.CameraMode.Classic
player.CameraMinZoomDistance = 0.5
player.CameraMaxZoomDistance = 50
local gui = Instance.new('ScreenGui')
gui.Name = 'AvatarFeelTest'; gui.ResetOnSpawn = false; gui.Parent = player:WaitForChild('PlayerGui')
local label = Instance.new('TextLabel')
label.Size = UDim2.fromOffset(470,100); label.Position = UDim2.fromOffset(16,60)
label.BackgroundColor3 = Color3.fromRGB(20,25,35); label.BackgroundTransparency = 0.15
label.TextColor3 = Color3.new(1,1,1); label.TextSize = 18; label.Font = Enum.Font.Code
label.Text = 'R6 FEEL TEST — Native Studio reference\nWASD move | Space jump | Right-drag orbit\nScroll/pinch: zoom into first person\n16 studs/s | Jump 53 | Gravity 196.2'
label.Parent = gui
local function configure(character)
 local humanoid = character:WaitForChild('Humanoid')
 humanoid.WalkSpeed = 16; humanoid.UseJumpPower = true; humanoid.JumpPower = 53
 humanoid.MaxSlopeAngle = 45
 local animate = character:FindFirstChild('Animate'); if animate then animate.Disabled = true end
 local animator = humanoid:WaitForChild('Animator')
 local tracks = {}
 for name,id in pairs({idle=180435571,walk=180426354,jump=125750702,fall=180436148,climb=180436334}) do
  local animation = Instance.new('Animation'); animation.AnimationId = 'rbxassetid://'..id
  local track = animator:LoadAnimation(animation)
  track.Priority = name=='idle' and Enum.AnimationPriority.Idle or Enum.AnimationPriority.Movement
  track.Looped = name~='jump'; tracks[name] = track
 end
 -- Classic R6 tool-hold clip overrides the equipped arm above locomotion.
 local holdAnimation=Instance.new('Animation');holdAnimation.AnimationId='rbxassetid://182393478'
 local hold=animator:LoadAnimation(holdAnimation);hold.Priority=Enum.AnimationPriority.Action;hold.Looped=true
 local function updateHold()
  if character:GetAttribute('EquippedGear') then
   if not hold.IsPlaying then hold:Play(0.1) end
  else hold:Stop(0.1) end
 end
 character:GetAttributeChangedSignal('EquippedGear'):Connect(updateHold)
 humanoid.Died:Connect(function() hold:Stop(0) end)
 updateHold()
 local swordTracks={}
 for name,id in pairs({Slash=129967390,Lunge=129967478}) do
  local a=Instance.new('Animation');a.AnimationId='rbxassetid://'..id
  local t=animator:LoadAnimation(a);t.Priority=Enum.AnimationPriority.Action2;t.Looped=false;swordTracks[name]=t
 end
 character.DescendantAdded:Connect(function(v)
  if v:IsA('StringValue') and v.Name=='toolanim' then
   local t=swordTracks[v.Value]
   if t then for _,old in pairs(swordTracks) do old:Stop(0) end;t:Play(0);game:GetService('Debris'):AddItem(v,0.3) end
  end
 end)
 local current
 local function play(name,rate)
  if current~=name then
   if current then tracks[current]:Stop(0.1) end
   current=name; tracks[name]:Play(0.1)
  end
  tracks[name]:AdjustSpeed(rate or 1)
 end
 humanoid.Running:Connect(function(speed)
  local state=humanoid:GetState()
  if state==Enum.HumanoidStateType.Running or state==Enum.HumanoidStateType.RunningNoPhysics then
   play(speed>0.1 and 'walk' or 'idle',speed>0.1 and speed/16 or 1)
  end
 end)
 humanoid.Climbing:Connect(function(speed) play('climb',math.abs(speed)/12) end)
 humanoid.StateChanged:Connect(function(_,state)
  if state==Enum.HumanoidStateType.Jumping then play('jump')
  elseif state==Enum.HumanoidStateType.Freefall then play('fall')
  elseif state==Enum.HumanoidStateType.Landed then play('idle') end
 end)
 play('idle')
 print('AVATAR_FEEL_TEST_READY',humanoid.RigType.Name,humanoid.WalkSpeed,humanoid.JumpPower,workspace.Gravity)
end
player.CharacterAdded:Connect(configure)
if player.Character then configure(player.Character) end
