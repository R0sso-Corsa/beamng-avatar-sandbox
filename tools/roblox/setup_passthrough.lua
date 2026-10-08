-- Command-bar installer: SESSION_CONFIG, SERVER_SOURCE and CLIENT_SOURCE supplied
-- by tools/configure_passthrough.py. Install only in a disposable local R6 place.
assert(game:GetService('RunService'):IsStudio(), 'Studio only')
local RS = game:GetService('ReplicatedStorage')
assert(not RS:FindFirstChild('AvatarPassthrough'), 'Bridge already installed; remove it before reinstalling')
assert(game:GetService('HttpService').HttpEnabled, 'Enable HTTP requests in this local test place first')
local starter = game:GetService('StarterPlayer')
if not starter:FindFirstChild('StarterCharacter') then
    local description = Instance.new('HumanoidDescription')
    description.HeadColor=Color3.fromRGB(245,205,48)
    description.LeftArmColor=description.HeadColor; description.RightArmColor=description.HeadColor
    description.TorsoColor=Color3.fromRGB(13,105,172)
    description.LeftLegColor=Color3.fromRGB(75,151,75); description.RightLegColor=description.LeftLegColor
    local rig=game:GetService('Players'):CreateHumanoidModelFromDescription(description,Enum.HumanoidRigType.R6)
    rig.Name='StarterCharacter'; rig.Parent=starter
    rig:SetAttribute('AvatarPassthroughCreated',true)
end
local remote = Instance.new('RemoteEvent'); remote.Name='AvatarPassthrough'; remote.Parent=RS
local server = Instance.new('Script'); server.Name='AvatarPassthrough'
server.Source=SERVER_SOURCE
local config = Instance.new('ModuleScript'); config.Name='SessionConfig'
config.Source='return game:GetService("HttpService"):JSONDecode('..string.format('%q',SESSION_CONFIG)..')'
config.Parent=server; server.Parent=game:GetService('ServerScriptService')
local client = Instance.new('LocalScript'); client.Name='AvatarPassthrough'
client.Source=CLIENT_SOURCE; client.Parent=game:GetService('StarterPlayer').StarterPlayerScripts
print('Local passthrough installed. Start relay and Play; configuration stays in this private place.')
