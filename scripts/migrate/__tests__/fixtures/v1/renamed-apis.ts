// Name: Renamed Grab Bag

// db() is only mentioned in this comment, which must NOT be flagged.
const text = await textarea("Draft something");
await wait(500);
const lodash = await npm("lodash");
await edit(kenvPath("scripts", "renamed-apis.ts"));
await dev({ text, chunked: lodash.chunk([1, 2, 3], 2) });
