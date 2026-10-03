-- Studio Command Bar, in the local reference place created during capture.
-- Evaluates real Animator tracks in edit mode; saves data, never publishes.
local rig=assert(workspace:FindFirstChild('R6Reference'),'Missing R6Reference')
local H=game:GetService('HttpService')
local controller=rig:FindFirstChildOfClass('Humanoid')
local animator=controller:FindFirstChildOfClass('Animator') or Instance.new('Animator',controller)
local joints={}
for _,j in ipairs(rig:GetDescendants()) do
 if j:IsA('Motor6D') and j.Part0 and j.Part1 then table.insert(joints,j) end
end
local rest=H:JSONDecode(game.ServerStorage.R6StudioValidationJSON.Value)
for name,p in pairs(rest.parts) do
 local part=rig:FindFirstChild(name)
 if part then part.Anchored=true;part.CFrame=CFrame.new(table.unpack(p.cframe)) end
end
for _,track in ipairs(animator:GetPlayingAnimationTracks()) do track:Stop(0) end
local captures={studioVersion=version(),method='Animator.StepAnimations edit-mode evaluation',clips={}}
local ids={idle1=180435571,idle2=180435792,walk=180426354,jump=125750702,fall=180436148,climb=180436334}
for name,id in pairs(ids) do
 local animation=Instance.new('Animation');animation.AnimationId='rbxassetid://'..id
 game:GetService('ContentProvider'):PreloadAsync({animation})
 local track=animator:LoadAnimation(animation)
 local deadline=os.clock()+10
 while track.Length==0 and os.clock()<deadline do task.wait(0.05) end
 assert(track.Length>0,'Clip did not load: '..name)
 track:Play(0,1,0);animator:StepAnimations(0)
 local capture={name=name,length=track.Length,samples={}}
 for _,fraction in ipairs({0,0.125,0.25,0.5,0.75}) do
  track.TimePosition=track.Length*fraction;animator:StepAnimations(0)
  local sample={time=track.TimePosition,poses={}}
  for _,j in ipairs(joints) do sample.poses[j.Part1.Name]={j.Transform:GetComponents()} end
  table.insert(capture.samples,sample)
 end
 table.insert(captures.clips,capture)
 track:Stop(0);animator:StepAnimations(0);track:Destroy();animation:Destroy()
end
local previous=game.ServerStorage:FindFirstChild('R6AnimatorCaptureJSON')
if previous then previous:Destroy() end
local value=Instance.new('StringValue');value.Name='R6AnimatorCaptureJSON'
value.Value=H:JSONEncode(captures);value.Parent=game.ServerStorage
-- Restore rest pose and lift the rig clear of the spawn platform for inspection.
local floor=workspace:FindFirstChild('SpawnLocation') or workspace:FindFirstChild('Baseplate')
local minY=math.huge
for name,p in pairs(rest.parts) do
 local part=rig:FindFirstChild(name)
 if part then part.CFrame=CFrame.new(table.unpack(p.cframe));if name~='HumanoidRootPart' then minY=math.min(minY,part.Position.Y-part.Size.Y/2) end end
end
local surface=floor and floor.Position.Y+floor.Size.Y/2 or 0
rig:PivotTo(rig:GetPivot()+Vector3.new(0,surface-minY+0.05,0))
print('ANIMATOR_CAPTURE_COMPLETE',#captures.clips,'floor clearance',0.05)
