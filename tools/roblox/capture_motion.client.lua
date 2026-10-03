-- Studio only: put in StarterPlayerScripts in a controlled R6 test place.
-- Run separate trials and use normal controls. Copy JSON from Studio Output.
local Players = game:GetService("Players")
local RunService = game:GetService("RunService")
local HttpService = game:GetService("HttpService")
local player = Players.LocalPlayer
local character = player.Character or player.CharacterAdded:Wait()
local humanoid = character:WaitForChild("Humanoid")
local root = character:WaitForChild("HumanoidRootPart")
local function xyz(v) return {v.X, v.Y, v.Z} end
local capture = {
    metadata = {
        clientVersion = version(), rig = humanoid.RigType.Name,
        gravity = workspace.Gravity, walkSpeed = humanoid.WalkSpeed,
        jumpPower = humanoid.JumpPower, jumpHeight = humanoid.JumpHeight,
        useJumpPower = humanoid.UseJumpPower, hipHeight = humanoid.HipHeight,
        maxSlopeAngle = humanoid.MaxSlopeAngle, assemblyMass = root.AssemblyMass,
        steppingMethod = workspace.PhysicsSteppingMethod.Name,
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
        print("AVATAR_MOTION_JSON " .. HttpService:JSONEncode(capture))
    end
end)
