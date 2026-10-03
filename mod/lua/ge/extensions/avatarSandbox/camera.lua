-- BeamNG-side facade only. Backend implements the Rust camera contract.
local M = {}
function M.new(backend)
  if type(backend)~='table' then return nil,'Rust camera backend required' end
  for _,method in ipairs({'setActive','look','zoom','pose'}) do
    if type(backend[method])~='function' then return nil,'Incomplete camera backend' end
  end
  local camera={}
  function camera.setActive(value) return backend.setActive(value) end
  function camera.look(yaw,pitch) return backend.look(yaw,pitch) end
  function camera.zoom(steps) return backend.zoom(steps) end
  function camera.pose(focus,eye,hit) return backend.pose(focus,eye,hit) end
  return camera
end
return M
