// Name: Side Effect Only
// Description: Completes without any prompt — smoke must accept exit 0
const target = tmpPath("migrate-smoke-proof.txt");
await Bun.write(target, `ran at pid ${process.pid}`);
