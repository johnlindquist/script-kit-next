// Demo for Agent Chat profile-backed menu syntax.
//
// Declares a `/rewrite` skill that can be paired with an Agent Chat profile.
// The profile itself lives in user config; the menuSyntax entry names the
// profile it expects so readers can keep the trigger and AI behavior together.
//
// Example user config shape:
//
// aiPreferences: {
//   profiles: [
//     {
//       id: "editorial-rewrite",
//       label: "Editorial rewrite",
//       model: "gpt-5.4",
//       systemPrompt: "Rewrite clearly, preserve facts, and ask if intent is unclear.",
//       temperature: 0.2,
//     },
//   ],
// },
//
// Reading this file: the launcher picks up `metadata.menuSyntax`, parses
// each entry's `family`, and exposes `skill.v1` rows for Agent Chat routing.

export const metadata = {
  name: "Agent Chat Profile Demo",
  description: "Show how a menuSyntax skill can point at an Agent Chat profile",
  icon: "message-circle-code",
  alias: "agent-profile-demo",
  tags: ["menu-syntax", "demo", "agent-chat"],
  category: "menu-syntax-demo",
  menuSyntax: [
    {
      family: "skill.v1",
      slug: "rewrite",
      label: "Rewrite with editorial profile",
      description:
        "Send the current selection to Agent Chat using the editorial-rewrite profile",
      contextRequirements: ["selection.text"],
      aiProfileId: "editorial-rewrite",
    },
  ],
};

console.log("Agent Chat profile demo invoked.");
console.log(
  "Open Agent Chat and look for the editorial-rewrite profile when running `/rewrite`."
);
