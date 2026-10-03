-- Roblox Classic-inspired camera math. Host applies pose and supplies collision.
local M = {}
local function finite(n)
  return type(n)=='number' and n==n and math.abs(n)<math.huge
end
local function point(p)
  return type(p)=='table' and finite(p.x) and finite(p.y) and finite(p.z)
end
function M.new()
  local yaw,pitch,distance,first,active=0,0,3,false,false
  local camera={}
  function camera.setActive(value)
    if type(value)~='boolean' then return false,'Expected boolean' end
    active=value; return true
  end
  function camera.look(deltaYaw,deltaPitch)
    if not finite(deltaYaw) or not finite(deltaPitch) then return false,'Invalid look delta' end
    if not active then return false,'Camera inactive' end
    local y,p=yaw+deltaYaw,pitch+deltaPitch
    if not finite(y) or not finite(p) then return false,'Look overflow' end
    yaw=(y+math.pi)%(2*math.pi)-math.pi
    pitch=math.max(-math.rad(80),math.min(math.rad(80),p))
    return true
  end
  -- Positive wheel steps zoom out. Bounds and rates are project settings.
  function camera.zoom(steps)
    if not finite(steps) then return false,'Invalid zoom' end
    if not active then return false,'Camera inactive' end
    distance=math.max(0,math.min(15,distance+steps*0.5))
    if first then first=distance<0.45 else first=distance<=0.3 end
    return true
  end
  -- Return the desired boom segment for a host sphere sweep.
  function camera.segment(focus,eye)
    if not active then return nil,'Camera inactive' end
    if not point(focus) or not point(eye) then return nil,'Invalid focus/eye' end
    local cosine=math.cos(pitch)
    local forward={x=-math.sin(yaw)*cosine,y=math.cos(yaw)*cosine,z=math.sin(pitch)}
    local origin=first and eye or focus
    local length=first and 0 or distance
    return {origin={x=origin.x,y=origin.y,z=origin.z},
      desired={x=origin.x-forward.x*length,y=origin.y-forward.y*length,z=origin.z-forward.z*length},
      forward=forward}
  end
  -- obstructionFraction is the host's first safe sweep fraction; nil means clear.
  function camera.pose(focus,eye,obstructionFraction)
    if obstructionFraction~=nil and (not finite(obstructionFraction) or obstructionFraction<0 or obstructionFraction>1) then return nil,'Invalid obstruction' end
    local segment,err=camera.segment(focus,eye); if not segment then return nil,err end
    local fraction=obstructionFraction or 1
    local actual=first and 0 or math.max(0,distance*fraction-(fraction<1 and 0.1 or 0))
    return {position={x=segment.origin.x-segment.forward.x*actual,y=segment.origin.y-segment.forward.y*actual,z=segment.origin.z-segment.forward.z*actual},
      forward=segment.forward,firstPerson=first,hideLocalHead=first,heading=yaw,
      requestedDistance=distance,actualDistance=actual}
  end
  function camera.status() return {active=active,firstPerson=first,distance=distance,yaw=yaw,pitch=pitch} end
  return camera
end
return M
