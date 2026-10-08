-- Local Studio only. HttpService runs on the server; no protected-client hooks.
local RunService = game:GetService('RunService')
assert(RunService:IsStudio(), 'Passthrough is restricted to local Studio sessions')
local Http = game:GetService('HttpService')
assert(Http.HttpEnabled, 'Enable HTTP requests in this disposable local place')
local Players = game:GetService('Players')
local config = require(script:WaitForChild('SessionConfig'))
local remote = game:GetService('ReplicatedStorage'):WaitForChild('AvatarPassthrough')
local scale = config.metresPerStud
local origin = Vector3.new(table.unpack(config.hostOrigin))
remote:SetAttribute('Mapping', Http:JSONEncode({metresPerStud=scale, hostOrigin=config.hostOrigin}))
local proxies = Instance.new('Folder')
proxies.Name = 'AvatarPassthroughCollision'; proxies.Parent = workspace
local boxes, player, latest, latestAt, sequence = {}, nil, nil, 0, 0
local running = true
local function point(v)
    local q = Vector3.new(table.unpack(v)) - origin
    return Vector3.new(q.X, q.Z, -q.Y) / scale
end
local function geometry(colliders)
    local keep = {}
    for _, b in ipairs(colliders) do
        keep[b.id] = true
        local p = boxes[b.id]
        if not p then
            p = Instance.new('Part'); p.Name = 'Host_'..b.id
            p.Anchored = true; p.Transparency = 1; p.CanCollide = true
            p.CanTouch = false; p.Parent = proxies; boxes[b.id] = p
        end
        p.Size = Vector3.new(b.size[1], b.size[3], b.size[2]) / scale
        p.CFrame = CFrame.new(point(b.position))
    end
    for id, p in pairs(boxes) do
        if not keep[id] then p:Destroy(); boxes[id] = nil end
    end
end
local receive = remote.OnServerEvent:Connect(function(sender, pose)
    -- Single local test player. Native relay validates the full numeric schema.
    if player and player ~= sender then return end
    if type(pose) ~= 'table' or type(pose.parts) ~= 'table' or #pose.parts > 16 then return end
    player = sender; latest = pose; latestAt = os.clock()
end)
local removing = Players.PlayerRemoving:Connect(function(p)
    if p == player then player = nil; latest = nil end
end)
local function release()
    if player then remote:FireClient(player, {active=false}) end
end
script.Destroying:Connect(function()
    running = false; release(); receive:Disconnect(); removing:Disconnect(); proxies:Destroy()
end)
task.spawn(function()
    while running do
        if player and latest and os.clock()-latestAt < 0.5 then
            sequence += 1
            local ok, result = pcall(function()
                local response = Http:RequestAsync({
                    Url = 'http://127.0.0.1:'..config.httpPort..'/exchange', Method = 'POST',
                    Headers = {['Content-Type']='application/json'},
                    Body = Http:JSONEncode({version=1, token=config.token, session=config.session,
                        sequence=sequence, kind='guest', payload=latest})
                })
                assert(response.Success, 'Relay rejected exchange: '..response.StatusCode)
                local reply = Http:JSONDecode(response.Body)
                assert(reply.ok and reply.version==1 and reply.session==config.session
                    and reply.sequence==sequence, 'Invalid relay reply')
                return reply
            end)
            if not running then break end
            if ok then
                if result.payload then geometry(result.payload.colliders) end
                if player then remote:FireClient(player, result) end
                remote:SetAttribute('LastError', '')
                remote:SetAttribute('Exchanges', sequence)
            else
                release()
                -- Do not log request bodies, config or account details.
                remote:SetAttribute('LastError', tostring(result):sub(1,200))
            end
        else release() end
        task.wait(0.2) -- ponytail: 5 Hz HTTP prototype; benchmark before faster transport.
    end
end)
