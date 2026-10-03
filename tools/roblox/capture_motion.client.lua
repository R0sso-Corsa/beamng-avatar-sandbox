-- Studio only: put in StarterPlayerScripts in a controlled R6 test place.
-- Run separate trials and use normal controls. Copy JSON from Studio Output.
local Players = game:GetService("Players")
local RunService = game:GetService("RunService")
local HttpService = game:GetService("HttpService")
local player = Players.LocalPlayer
local character = player.Character or player.CharacterAdded:Wait()
local humanoid = character:WaitForChild("Humanoid")
local root = character:WaitForChild("HumanoidRootPart")
-- Fill these in for each trial. PhysicsSteppingMethod is NotScriptable;
-- copy its setting from Studio's Workspace Properties panel.
local trialLabel = "flat_idle"
local steppingMethod = "record manually from Workspace Properties"
local function xyz(v) return {v.X, v.Y, v.Z} end
local capture = {
    metadata = {
        clientVersion = version(), rig = humanoid.RigType.Name,
        gravity = workspace.Gravity, walkSpeed = humanoid.WalkSpeed,
        jumpPower = humanoid.JumpPower, jumpHeight = humanoid.JumpHeight,
        useJumpPower = humanoid.UseJumpPower, hipHeight = humanoid.HipHeight,
        maxSlopeAngle = humanoid.MaxSlopeAngle, assemblyMass = root.AssemblyMass,
        trial = trialLabel, steppingMethod = steppingMethod,
        sampling = "PostSimulation (frame level)", axis = "Roblox Y-up, studs",
    }, samples = {},
}
local elapsed = 0
local connection
connection = RunService.PostSimulation:Connect(function(dt)
    if not root.Parent or not humanoid.Parent then
        connection:Disconnect()
        warn("Capture stopped: character removed; discard incomplete trial")
        return
    end
    elapsed = elapsed + dt
    table.insert(capture.samples, {
        time = elapsed, dt = dt, position = xyz(root.Position),
        velocity = xyz(root.AssemblyLinearVelocity), moveDirection = xyz(humanoid.MoveDirection),
        state = humanoid:GetState().Name, floor = humanoid.FloorMaterial.Name,
        jump = humanoid.Jump,
    })
    if elapsed >= 10 then
        connection:Disconnect()
        -- Small batches avoid truncating one large Studio Output message.
        print("AVATAR_MOTION_METADATA " .. HttpService:JSONEncode(capture.metadata))
        for first = 1, #capture.samples, 30 do
            local batch = {}
            for i = first, math.min(first + 29, #capture.samples) do
                table.insert(batch, capture.samples[i])
            end
            print("AVATAR_MOTION_SAMPLES " .. HttpService:JSONEncode({first = first, samples = batch}))
        end
    end
end)
