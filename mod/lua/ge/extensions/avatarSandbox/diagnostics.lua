-- Read-only GE capability inventory. Symbol presence is not API validation.
local M = {}
function M.collect()
  return {
    schemaVersion = 1,
    luaVersion = _VERSION,
    jitPresent = type(jit) == 'table',
    jitVersion = type(jit) == 'table' and tostring(jit.version) or 'unavailable',
    extensionsPresent = type(extensions) == 'table',
    sceneTreePresent = type(scenetree) == 'table',
    vectorConstructorPresent = type(vec3) == 'function',
    staticRaycastSymbolPresent = type(castRayStatic) == 'function',
    cameraPositionSymbolPresent = type(getCameraPosition) == 'function',
    nativeLoadingVerified = false,
    collisionExtractionVerified = false,
    avatarRenderingVerified = false
  }
end
function M.printReport()
  local report = M.collect()
  local keys = {}
  for key in pairs(report) do keys[#keys + 1] = key end
  table.sort(keys)
  for _, key in ipairs(keys) do
    local message = key .. '=' .. tostring(report[key])
    if type(log) == 'function' then log('I', 'avatarSandboxDiagnostics', message)
    elseif type(print) == 'function' then print(message) end
  end
  return report
end
return M
