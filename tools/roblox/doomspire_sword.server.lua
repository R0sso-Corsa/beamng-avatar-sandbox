-- Adapted from public Doomspire source; see assets/source/doomspire_gears/sword.
local Tool=script.Parent
local Damage=Tool:WaitForChild("damage")
local sword=Tool:WaitForChild("Handle")
local r = game:service("RunService")

local base_damage = 5
local slash_damage = 10
local lunge_damage = 30

local ModFx=require(game.ReplicatedStorage.AvatarSwordSupport)
local Player=ModFx.GetPlayerFromTool(Tool)

local trails={
	["Rainbow"]=function()
		return game.ServerStorage.Trails.RainbowKeypoints.Color
	end,
	["RisingSun"]=function()
		return game.ServerStorage.Trails.RisingSunKeypoints.Color
	end,
	["Arctic"]=function()
		return game.ServerStorage.Trails.ArcticKeypoints.Color
	end,
	["TeamColor"]=function()
		return ColorSequence.new(Player.TeamColor.Color)
	end,
	["Fire"]=function()
		sword.Trail.Enabled=false -- just in case
		for i,v in pairs(sword:GetChildren()) do
			if v:IsA("Attachment") then
				Instance.new("Fire",v)
			end
		end
	end
}

local function swordUp()
	Tool.GripForward = Vector3.new(-1,0,0)
	Tool.GripRight = Vector3.new(0,1,0)
	Tool.GripUp = Vector3.new(0,0,1)
end

local function swordOut()
	Tool.GripForward = Vector3.new(0,0,1)
	Tool.GripRight = Vector3.new(0,-1,0)
	Tool.GripUp = Vector3.new(-1,0,0)
end

local function IsEffect(str)
	local a=false
	for i,v in pairs(trails) do
		if str==i then
			a=true
		end
	end
	return a
end

local function applyTrail(typ)
	if IsEffect(typ)==false then
		return
	end
	if typ=="Fire" then
		trails[typ]()
	else
		sword.Trail.Enabled=true
		sword.Trail.Color=trails[typ]()
	end
end

local function removeTrail()
	sword.Trail.Enabled=false
	for i,v in pairs(sword:GetChildren()) do
		if v:IsA("Attachment") then
			v:ClearAllChildren()
		end
	end
end

local function dmg(humanoid,me,ePlayer)
	humanoid:TakeDamage(Tool.damage.Value) -- cover all humanoids, npc or other
	if ePlayer then
		if 	(not humanoid:FindFirstChild("creator")) and 
			me:FindFirstChild("leaderstats") then
			local creator = Instance.new("ObjectValue")
			creator.Value = me
			creator.Name = "creator"
			creator.Parent = humanoid
		end
		--humanoid:TakeDamage(Tool.damage.Value)
	end
end

sword.Touched:connect(function(hit)
	if hit.Parent then 
		local humanoid = hit.Parent:FindFirstChild("Humanoid")
		local vCharacter = Player.Character
		local hum = vCharacter:FindFirstChild("Humanoid")
		
		if humanoid and hum and humanoid~=hum and hum.Health>0 then
			local ePlayer = game.Players:GetPlayerFromCharacter(humanoid.Parent)
			if ePlayer then
				local ATK=Player:FindFirstChild("AntiTeamKill")
				if 	(ATK and ATK.Value==true) and
					(Player.Neutral==false and ePlayer.Neutral==false) and
					Player.TeamColor==ePlayer.TeamColor then
					return -- force-end the function
				end
				dmg(humanoid,Player,ePlayer)
			else
				dmg(humanoid,Player)
			end
		end

	end 
end)

local function attack(plr)
	Damage.Value=slash_damage
	sword.SwordSlash:Play()
	local anim = Instance.new("StringValue")
	anim.Name = "toolanim"
	anim.Value = "Slash"
	anim.Parent = Tool
	Damage.Value=base_damage
end

local function lunge(plr)

	local effect=Player:FindFirstChild("Effect")
	if effect then
		applyTrail(effect.Value)
	end
	sword.SwordLunge:Play()
	Damage.Value=lunge_damage
	local anim = Instance.new("StringValue")
	anim.Name = "toolanim"
	anim.Value = "Lunge"
	anim.Parent = Tool
	
	local vCharacter = Player.Character
	
	local force = Instance.new("BodyVelocity")
	force.velocity = Vector3.new(0,14.5,0) 
	force.maxForce = Vector3.new(0,5000,0)
	
	if Tool.Parent.Name~="Backpack" then
		force.Parent = vCharacter.HumanoidRootPart
		game:GetService("Debris"):AddItem(force,0.5)
	end	
	
	wait(.25)
	swordOut()
--	wait(.25)
--	force:Destroy() -- now in onDied
--	wait(.5)
	wait(.75)
	swordUp()
	removeTrail()
	Damage.Value=base_damage
end

Tool.Enabled = true
local last_attack = 0

Tool.AvatarLabActivate.Event:connect(function()
	if Tool.Enabled==false then return end
	Tool.Enabled = false
	local humanoid = Player.Character.Humanoid
	if not humanoid then
		return
	end
	local t = r.Stepped:wait()
	if humanoid.Health~=0 then
		if (t - last_attack < .2) then
			lunge()
		else
			attack()
		end
	end
	last_attack = t
	--wait(.5)
	Tool.Enabled = true
end)

Tool.Equipped:connect(function(mouse)
	sword.Unsheath:Play()
end)
