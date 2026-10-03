const fs=require('fs'),vm=require('vm'),assert=require('assert');
let factory,commands=[],handlers={},events={},cancelled=false;
const context={angular:{module:()=>({directive:(name,args)=>{factory=args[args.length-1];}})},
 bngApi:{engineLua:s=>commands.push(s)},window:{addEventListener:(n,f)=>events[n]=f,removeEventListener:n=>delete events[n]}};
vm.runInNewContext(fs.readFileSync('mod/ui/modules/apps/avatarSandboxControls/app.js','utf8'),context);
const interval=()=>1;interval.cancel=()=>cancelled=true;
const scope={$on:(n,f)=>handlers[n]=f,$evalAsync:f=>f()};factory(interval).link(scope);
scope.send("'); bad() --");assert.strictEqual(commands.length,1);
scope.press('forward');assert(commands.at(-1).includes('forward1'));
events.blur();assert(commands.at(-1).includes('forward0'));
scope.press('left');handlers.$destroy();assert(commands.at(-1).includes('left0'));
assert(cancelled && !events.mouseup && !events.blur);
console.log('UI command whitelist, focus release and teardown passed');
