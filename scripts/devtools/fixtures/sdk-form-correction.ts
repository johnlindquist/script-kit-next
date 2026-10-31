// Fixed, provider-free SDK script. Without a delivered submit this promise stays pending.
import "../../kit-sdk.ts";
import { writeSync } from "node:fs";
const fixtureId = "sdk.form-correction.v1";
if (process.env.SDK_TEST_AUTOSUBMIT !== "0" || process.env.SCRIPT_KIT_SDK_FIXTURE !== fixtureId) throw Error("reviewed_sdk_fixture_environment_required");
let rpcResolved = false, promptResolved = false;
let value: Record<string, string> | null = null;
const received: Array<{type: string; id: string; value: string | null}> = [];
const report = (event: string, extra = {}) => writeSync(2, `SDK_FIXTURE ${JSON.stringify({fixtureId,event,rpcResolved,promptResolved,received,value,...extra})}\n`);
let buffer = "";
process.stdin.prependListener("data", chunk => {
  buffer += chunk.toString();
  if (buffer.length > 65536) throw Error("sdk_fixture_input_limit");
  for (let i; (i = buffer.indexOf("\n")) >= 0;) {
    const message = JSON.parse(buffer.slice(0, i)); buffer = buffer.slice(i + 1);
    if (message.type === "submit") { received.push(message); if (received.length > 1) throw Error("duplicate_submit"); }
  }
});
process.on("exit", code => report("exit", {code}));
const prompt = form('<form><label for="email">Email</label><input id="email" name="email" type="email" value="invalid"/><label for="notes">Notes</label><textarea id="notes" name="notes">First line</textarea></form>');
const state = await getState(); rpcResolved = true; report("rpc", {state});
value = await prompt; promptResolved = true; report("resolved");
