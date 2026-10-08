-- Native physics and animation remain in Studio; only intent and rigid poses cross.
local RunService = game:GetService('RunService')
assert(RunService:IsStudio(), 'Passthrough is restricted to local Studio sessions')
local Players = game:GetService('Players')
local Http = game:GetService('HttpService')
local player = Players.LocalPlayer
local remote = game:GetService('ReplicatedStorage'):WaitForChild('AvatarPassthrough')
while not remote:GetAttribute('Mapping') do remote:GetAttributeChangedSignal('Mapping'):Wait() end
local mapping = Http:JSONDecode(remote:GetAttribute('Mapping'))
local scale = mapping.metresPerStud
local origin = Vector3.new(table.unpack(mapping.hostOrigin))
local controls = require(player.PlayerScripts:WaitForChild('PlayerModule')):GetControls()
local names = {'Head','Torso','Left Arm','Right Arm','Left Leg','Right Leg'}
local command, received, claimed, elapsed = nil, 0, false, 0
local controlsWereEnabled = true
local function direction(v) return {v.X, -v.Z, v.Y} end
local function point(v)
    return {origin.X+scale*v.X, origin.Y-scale*v.Z, origin.Z+scale*v.Y}
end
local function release()
    if claimed then
        claimed = false
        local h = player.Character and player.Character:FindFirstChildOfClass('Humanoid')
        if h then h:Move(Vector3.zero, false); h.Jump = false end
        if controlsWereEnabled then controls:Enable() else controls:Disable() end
    end
end
local connection = remote.OnClientEvent:Connect(function(reply)
    if reply.active and reply.payload then command = reply.payload; received = os.clock() - math.clamp((reply.peerAgeMs or 1000)/1000, 0, 1)
    else command = nil; release() end
end)
-- PlayerModule applies controls in RenderStep. Override just after its input callback.
local binding = 'AvatarPassthroughInput'
RunService:BindToRenderStep(binding, Enum.RenderPriority.Input.Value+1, function()
    local h = player.Character and player.Character:FindFirstChildOfClass('Humanoid')
    if not command or os.clock()-received >= 1 or not h or h.Health <= 0 then release(); return end
    if not claimed then
        controlsWereEnabled = controls.controlsEnabled ~= false
        controls:Disable(); claimed = true
    end
    h:Move(Vector3.new(command.movement[1],0,-command.movement[2]), false)
    h.Jump = command.jump
end)
local capture = RunService.PostSimulation:Connect(function(dt)
    elapsed += dt
    if elapsed < 0.05 then return end
    elapsed = 0
    local character = player.Character
    local root = character and character:FindFirstChild('HumanoidRootPart')
    local h = character and character:FindFirstChildOfClass('Humanoid')
    local camera = workspace.CurrentCamera
    if not root or not h or not camera then return end
    if h.RigType ~= Enum.HumanoidRigType.R6 then
        remote:SetAttribute('ClientError', 'R6 character required'); return
    end
    local parts = {}
    for _, name in ipairs(names) do
        local p = character:FindFirstChild(name)
        if not p then return end
        local cf = p.CFrame
        parts[#parts+1] = {id=name,position=point(p.Position),
            size={p.Size.X*scale,p.Size.Y*scale,p.Size.Z*scale},
            right=direction(cf.RightVector),up=direction(cf.UpVector),back=direction(-cf.LookVector),
            color={p.Color.R,p.Color.G,p.Color.B}}
    end
    local v = root.AssemblyLinearVelocity
    remote:FireServer({position=point(root.Position),velocity={v.X*scale,-v.Z*scale,v.Y*scale},
        state=h:GetState().Name,parts=parts,camera={position=point(camera.CFrame.Position),
        forward=direction(camera.CFrame.LookVector),up=direction(camera.CFrame.UpVector),fov=camera.FieldOfView}})
end)
script.Destroying:Connect(function()
    release(); connection:Disconnect(); capture:Disconnect(); RunService:UnbindFromRenderStep(binding)
end)
