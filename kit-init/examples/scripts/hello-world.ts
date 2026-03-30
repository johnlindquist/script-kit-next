import "@scriptkit/sdk";

export const metadata = {
  name: "Hello World",
  description: "A simple greeting demonstrating basic prompts and display",
};

const name = await arg("What's your name?", ["World", "Script Kit", "Friend"]);

await div(`
  <div class="flex flex-col items-center justify-center h-full p-8">
    <h1 class="text-4xl font-bold text-yellow-400 mb-4">
      Hello, ${name}!
    </h1>
    <p class="text-gray-400 text-lg">Welcome to Script Kit</p>
  </div>
`);
