"""Offline diagnostics check; requires lupa (does not emulate BeamNG)."""
from pathlib import Path
from lupa import LuaRuntime
root = Path(__file__).resolve().parents[1]
lua = LuaRuntime(unpack_returned_tuples=True)
module = lua.execute((root / "mod/lua/ge/extensions/avatarSandbox/diagnostics.lua").read_text())
report = module.collect()
assert report["schemaVersion"] == 1
assert report["nativeLoadingVerified"] is False
assert report["staticRaycastSymbolPresent"] is False
lua.execute("castRayStatic = function() error('Diagnostics must never call raycast') end")
assert module.collect()["staticRaycastSymbolPresent"] is True
lua.execute("messages = {}; log = function(level, tag, text) messages[#messages+1] = text end")
module.printReport()
assert len(lua.globals().messages) == 12
print("Diagnostics inventory and read-only behavior passed")
