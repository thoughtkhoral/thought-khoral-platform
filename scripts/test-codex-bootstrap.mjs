import assert from 'node:assert/strict';
import fs from 'node:fs';
const path = new URL('../ui/codex-admission.js', import.meta.url);
assert(fs.existsSync(path), 'host opt-in admission loader is missing');
const { readConversationAdmission } = await import(`data:text/javascript,${encodeURIComponent(fs.readFileSync(path,'utf8'))}`);
const valid = { profileVersion: 'thought-khoral.agent-conversation.v1', agentId:'74686f75-6768-746b-686f-72616c000004', conversationScope:'room', invocation:'explicitly-addressed', delivery:'room', roomHistory:'baseline-and-delta', modelSelection:true };
const response = (value,ok=true,expiresAtUnixMs=Date.now()+4000) => async () => ({ok,text:async()=>JSON.stringify({admission:value,expiresAtUnixMs})});
assert.equal(await readConversationAdmission(response(null,false)), undefined);
assert.deepEqual(await readConversationAdmission(response(valid)),valid);
for (const invalid of [{...valid,endpoint:'http://worker'}, {...valid,toString:'unexpected'}, {...valid,modelSelection:'yes'}, {...valid,agentId:'other'}, {...valid,roomHistory:undefined}]) {
  assert.equal(await readConversationAdmission(response(invalid)),undefined);
}
assert.equal(await readConversationAdmission(async()=>{throw Error('offline')}),undefined);
console.log('Codex host admission fail-closed checks passed');

assert.equal(await readConversationAdmission(response(valid,true,Date.now()-1)),undefined);
assert.equal(await readConversationAdmission(response(valid,true,Date.now()+60000)),undefined);
