-- Studio Command Bar: isolate climb's left-arm interpolation.
local rig=workspace.R6Reference
local animator=rig.Humanoid:FindFirstChildOfClass('Animator')
for _,t in ipairs(animator:GetPlayingAnimationTracks()) do t:Stop(0) end
local animation=Instance.new('Animation');animation.AnimationId='rbxassetid://180436334'
game:GetService('ContentProvider'):PreloadAsync({animation})
local track=animator:LoadAnimation(animation);track:Play(0,1,0)
local deadline=os.clock()+10
while track.Length==0 and os.clock()<deadline do task.wait(0.05) end
assert(track.Length>0,'Track not loaded')
local keys={}
for _,key in ipairs(game.ServerStorage.climb:GetKeyframes()) do
 for _,p in ipairs(key:GetDescendants()) do
  if p:IsA('Pose') and p.Name=='Left Arm' then table.insert(keys,{time=key.Time,pose=p.CFrame}) end
 end
end
table.sort(keys,function(a,b)return a.time<b.time end)
local motor
for _,j in ipairs(rig:GetDescendants()) do if j:IsA('Motor6D') and j.Part1 and j.Part1.Name=='Left Arm' then motor=j end end
assert(motor,'Missing left arm motor')
local results={studioVersion=version(),samples={}}
for step=0,90 do
 local time=step*0.005
 track.TimePosition=time;animator:StepAnimations(0)
 local a,b=keys[1],keys[#keys]
 for i=1,#keys-1 do if time>=keys[i].time and time<=keys[i+1].time then a,b=keys[i],keys[i+1];break end end
 local alpha=(time-a.time)/(b.time-a.time)
 table.insert(results.samples,{time=time,animator={motor.Transform:GetComponents()},
  lerp={a.pose:Lerp(b.pose,alpha):GetComponents()},alpha=alpha,
  startTime=a.time,endTime=b.time})
end
track:Stop(0);animator:StepAnimations(0);track:Destroy();animation:Destroy()
local old=game.ServerStorage:FindFirstChild('R6ClimbAuditJSON');if old then old:Destroy() end
local v=Instance.new('StringValue');v.Name='R6ClimbAuditJSON';v.Value=game:GetService('HttpService'):JSONEncode(results);v.Parent=game.ServerStorage
print('CLIMB_AUDIT_COMPLETE',#results.samples)
