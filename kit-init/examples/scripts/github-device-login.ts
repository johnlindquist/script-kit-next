import "@scriptkit/sdk";
import { persistDeviceToken, runDeviceFlow } from "./lib/oauth-device-flow";

export const metadata = {
  name: "GitHub Device Login",
  description: "Sign in with GitHub's OAuth device flow and save the token locally",
  icon: "github",
};

const isVerify = process.env.SK_VERIFY === "1";
const clientId = process.env.GITHUB_OAUTH_CLIENT_ID || "";

if (isVerify) {
  console.log(JSON.stringify({ ok: true, provider: "github", needs: "GITHUB_OAUTH_CLIENT_ID" }));
} else if (!clientId) {
  await div(`
    <div class="p-6 space-y-3">
      <h1 class="text-xl font-semibold">GitHub Device Login</h1>
      <p>Set <code>GITHUB_OAUTH_CLIENT_ID</code> to the client ID from a GitHub OAuth app, then run this script again.</p>
      <p class="text-sm opacity-70">Suggested scopes in this example: <code>read:user user:email repo</code>. Trim them for your own script.</p>
    </div>
  `);
} else {
  const token = await runDeviceFlow({
    provider: "github",
    clientId,
    deviceAuthorizationUrl: "https://github.com/login/device/code",
    tokenUrl: "https://github.com/login/oauth/access_token",
    scopes: ["read:user", "user:email", "repo"],
    onDeviceCode: async (authorization) => {
      await copy(authorization.user_code);
      const url = authorization.verification_uri_complete || authorization.verification_uri;
      await open(url);
      console.log(`Opened GitHub device authorization and copied ${authorization.user_code}`);
    },
    onPoll: ({ attempt }) => {
      if (attempt === 1) console.log("Waiting for GitHub authorization...");
    },
  });

  const path = await persistDeviceToken("github", token);
  const action = await arg("GitHub connected", [
    {
      name: "Open OAuth Apps",
      description: `Saved token to ${path}`,
      value: "open",
    },
    {
      name: "Done",
      description: "Keep the saved token for GitHub API examples",
      value: "done",
    },
  ]);

  if (action === "open") {
    await open("https://github.com/settings/developers");
  }

  await div(`
    <div class="p-6 space-y-3">
      <h1 class="text-xl font-semibold">GitHub Connected</h1>
      <p>Saved token to <code>${path}</code>.</p>
      <p class="text-sm opacity-70">The <code>;github</code> example can now create issues through the GitHub API.</p>
    </div>
  `);
}
