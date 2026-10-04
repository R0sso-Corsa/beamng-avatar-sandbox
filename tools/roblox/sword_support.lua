-- Minimal adapter for the source sword's player lookup dependency.
return {GetPlayerFromTool=function(tool)
 return assert(game:GetService('Players'):GetPlayerFromCharacter(tool.Parent),'Sword must belong to a player character')
end}
