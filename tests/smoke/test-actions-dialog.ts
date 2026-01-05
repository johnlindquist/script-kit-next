// Test for actions dialog vibrancy and shadow
import '../../scripts/kit-sdk';

// Show main menu with actions - the user can press Cmd+K to see the actions dialog
await arg({
  placeholder: "Press Cmd+K to test actions dialog...",
  choices: [
    { name: "AI Chat", description: "Chat with AI assistants", value: "ai-chat" },
    { name: "Script Runner", description: "Run your scripts", value: "runner" },
    { name: "Settings", description: "Configure Script Kit", value: "settings" },
  ],
  actions: [
    { name: "Run", shortcut: "enter" },
    { name: "Edit Script", shortcut: "cmd+e" },
    { name: "Configure Shortcut", shortcut: "cmd+shift+k" },
    { name: "View Logs", shortcut: "cmd+l" },
    { name: "Reveal in Finder", shortcut: "cmd+shift+f" },
    { name: "Copy Path", shortcut: "cmd+shift+c" },
    { name: "Copy Deeplink", shortcut: "cmd+shift+d" },
    { name: "Create New Script", shortcut: "cmd+n" },
    { name: "Reload Scripts", shortcut: "cmd+r" },
  ]
});
