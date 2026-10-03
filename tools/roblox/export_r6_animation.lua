-- Studio Command Bar: select an R6 model and local KeyframeSequence(s).
-- Optional: fill animationIds with accessible asset IDs, e.g. rbxassetid://...
-- This exports data only; no downloads, HTTP upload or executable extraction.
local animationIds = {}
local Selection = game:GetService('Selection')
local HttpService = game:GetService('HttpService')
local Provider = game:GetService('KeyframeSequenceProvider')
local rig, sequences = nil, {}
for _, item in ipairs(Selection:Get()) do
    if item:IsA('Model') and item:FindFirstChildOfClass('Humanoid') then
        assert(not rig, 'Select only one rig'); rig = item
    elseif item:IsA('KeyframeSequence') then
        table.insert(sequences, {sequence=item, source='local KeyframeSequence'})
    end
end
assert(rig and rig.Humanoid.RigType == Enum.HumanoidRigType.R6, 'Select an R6 rig')
for _, id in ipairs(animationIds) do
    local ok, sequence = pcall(function() return Provider:GetKeyframeSequenceAsync(id) end)
    assert(ok, 'Cannot access animation '..tostring(id)..': '..tostring(sequence))
    table.insert(sequences, {sequence=sequence, source=id})
end
assert(#sequences > 0, 'Select exported KeyframeSequence(s) or fill animationIds')
local function frame(cf) return {cf:GetComponents()} end
local joints = {}
for _, joint in ipairs(rig:GetDescendants()) do
    if joint:IsA('Motor6D') and joint.Part0 and joint.Part1 then
        table.insert(joints, {name=joint.Name, parent=joint.Part0.Name, child=joint.Part1.Name,
            c0=frame(joint.C0), c1=frame(joint.C1)})
    end
end
table.sort(joints, function(a,b) return a.name < b.name end)
local clips = {}
for _, entry in ipairs(sequences) do
    local sequence = entry.sequence
    local keys = sequence:GetKeyframes()
    table.sort(keys, function(a,b) return a.Time < b.Time end)
    local frames = {}
    for _, key in ipairs(keys) do
        local poses = {}
        local function visit(pose, path)
            path = path..'/'..pose.Name
            table.insert(poses, {path=path, part=pose.Name, transform=frame(pose.CFrame),
                weight=pose.Weight, easingStyle=pose.EasingStyle.Name,
                easingDirection=pose.EasingDirection.Name})
            for _, child in ipairs(pose:GetSubPoses()) do visit(child,path) end
        end
        for _, pose in ipairs(key:GetPoses()) do visit(pose,'') end
        table.insert(frames, {time=key.Time, name=key.Name, poses=poses})
    end
    table.insert(clips, {name=sequence.Name, source=entry.source, loop=sequence.Loop,
        priority=sequence.Priority.Name, frames=frames})
end
local json = HttpService:JSONEncode({schemaVersion=1, rig='R6',
    coordinates='Roblox Y-up, studs; CFrame position followed by row-major 3x3 rotation',
    joints=joints, clips=clips})
-- Store a selectable full string so large clips do not rely on truncated Output.
local result = Instance.new('StringValue')
result.Name = 'R6AnimationExportJSON'; result.Value = json
result.Parent = game:GetService('ServerStorage')
Selection:Set({result})
print('Export complete: copy R6AnimationExportJSON.Value to a local JSON file')
