// Headless subset of the script SDK. The launcher supplies the full interactive SDK instead.
const args = await Bun.stdin.json();
let result: Record<string, unknown> | undefined;
Object.assign(globalThis, {
	schema: {},
	input: async () => args,
	output: (data: Record<string, unknown>) => {
		result = { ...result, ...data };
	},
	defineSchema: (schema: object) => ({
		schema,
		input: globalThis.input,
		output: globalThis.output,
	}),
});
await import("./command.ts");
if (!result) throw Error("No output from reviewed command");
process.stdout.write(JSON.stringify(result));
