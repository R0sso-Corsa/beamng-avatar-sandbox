-- Run in the SERVER command bar of a disposable Studio play session.
-- Native Humanoid reference only: no custom controller, animation or gear code.
-- Returns JSON; no script/asset/account data. PostSimulation is frame sampling.
local RunService = game:GetService("RunService")
local Players = game:GetService("Players")
local rig = Players:CreateHumanoidModelFromDescriptionAsync(
    Instance.new("HumanoidDescription"), Enum.HumanoidRigType.R6)
local folder = Instance.new("Folder")
folder.Name = "TemporaryNativeReference"
folder.Parent = workspace
rig.Parent = folder
local h, root = rig.Humanoid, rig.HumanoidRootPart
root:SetNetworkOwner(nil)
h.WalkSpeed, h.JumpPower, h.UseJumpPower = 16, 50, true
local floor = Instance.new("Part")
floor.Anchored = true
floor.Size = Vector3.new(300, 2, 300)
floor.Position = Vector3.new(500, 0, 500)
floor.Parent = folder
local result = {metadata = {version = version(), rig = h.RigType.Name,
    gravity = workspace.Gravity, mass = root.AssemblyMass,
    rootSize = {root.Size.X, root.Size.Y, root.Size.Z}, hipHeight = h.HipHeight,
    walkSpeed = h.WalkSpeed, jumpPower = h.JumpPower, jumpHeight = h.JumpHeight,
    autoRotate = h.AutoRotate, maxSlope = h.MaxSlopeAngle,
    ownership = "server", sampling = "PreSimulation input / PostSimulation output"},
    trials = {}}
local connection
local function reset(position)
    h:Move(Vector3.zero)
    root.CFrame = CFrame.new(position)
    root.AssemblyLinearVelocity, root.AssemblyAngularVelocity = Vector3.zero, Vector3.zero
    h:ChangeState(Enum.HumanoidStateType.Running)
    task.wait(0.5)
end
local function capture(label, duration, control)
    local t, input, samples = 0, Vector3.zero, {}
    connection = RunService.PreSimulation:Connect(function()
        input = control(t)
        h:Move(input)
    end)
    while t < duration do
        local dt = RunService.PostSimulation:Wait()
        t += dt
        local p, v, look = root.Position, root.AssemblyLinearVelocity, root.CFrame.LookVector
        table.insert(samples, {t = t, dt = dt, x = p.X, y = p.Y, z = p.Z,
            vx = v.X, vy = v.Y, vz = v.Z, input = input.X,
            look = {look.X, look.Z}, angular = root.AssemblyAngularVelocity.Y,
            state = h:GetState().Name, floor = h.FloorMaterial.Name})
    end
    connection:Disconnect()
    connection = nil
    h:Move(Vector3.zero)
    table.insert(result.trials, {label = label, samples = samples})
end
local ok, err = xpcall(function()
    for _, friction in ipairs({0.05, 0.5, 1}) do
        floor.CustomPhysicalProperties = PhysicalProperties.new(1, friction, 0, 100, 100)
        reset(Vector3.new(500, 4, 500))
        capture("friction_" .. friction, 1.8, function(t)
            return t < 0.6 and Vector3.xAxis or t < 1.2 and -Vector3.xAxis or Vector3.zero
        end)
    end
    floor.CustomPhysicalProperties = nil
    for _, speed in ipairs({8, 16, 32}) do
        h.WalkSpeed = speed
        for _, magnitude in ipairs({0, 0.5, 1}) do
            reset(Vector3.new(500, 4, 500))
            capture("walk_" .. speed .. "_" .. magnitude, 1.2, function(t)
                return t < 0.6 and Vector3.xAxis * magnitude or Vector3.zero
            end)
        end
    end
    h.WalkSpeed = 16
    for _, mode in ipairs({"power25", "power50", "power53", "height7.2", "airReverse"}) do
        h.UseJumpPower = mode ~= "height7.2"
        h.JumpPower = tonumber(string.match(mode, "^power(%d+)")) or 50
        h.JumpHeight = 7.2
        reset(Vector3.new(500, 4, 500))
        local fired = false
        capture(mode, 1.2, function(t)
            if not fired then h.Jump = true; fired = true end
            if mode == "airReverse" then return t < 0.15 and Vector3.xAxis or -Vector3.xAxis end
            return Vector3.zero
        end)
    end
end, debug.traceback)
if connection then connection:Disconnect() end
folder:Destroy()
if not ok then error(err) end
-- Retain the full result for chunked retrieval; large tool/Output returns truncate.
_G.NativeReferenceResult = result
return {metadata = result.metadata, trials = #result.trials}
