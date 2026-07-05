// Name: Hello World
// Description: Greets the chosen person
// Shortcut: cmd shift h

const who = await arg("Who should we greet?", ["world", "team", "future me"]);
await div(md(`# Hello, ${who}!`));
