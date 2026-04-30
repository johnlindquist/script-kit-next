import "@scriptkit/sdk";
import { persistDeviceToken, runDeviceFlow } from "./lib/oauth-device-flow";

export const metadata = {
  name: "Microsoft Graph Device Login",
  description: "Sign in with Microsoft Graph's device flow and save the token locally",
  icon: "key-round",
};

const isVerify = process.env.SK_VERIFY === "1";
const tenant = process.env.MS_GRAPH_TENANT || "common";
const clientId = process.env.MS_GRAPH_CLIENT_ID || "";

if (isVerify) {
  console.log(JSON.stringify({ ok: true, provider: "microsoft-graph", needs: "MS_GRAPH_CLIENT_ID" }));
} else if (!clientId) {
  await div(`
    <div class="p-6 space-y-3">
      <h1 class="text-xl font-semibold">Microsoft Graph Device Login</h1>
      <p>Set <code>MS_GRAPH_CLIENT_ID</code> to an app registration client ID, then run this script again.</p>
      <p class="text-sm opacity-70">Optional: set <code>MS_GRAPH_TENANT</code> to <code>common</code>, <code>organizations</code>, or a tenant ID.</p>
    </div>
  `);
} else {
  const authority = `https://login.microsoftonline.com/${tenant}/oauth2/v2.0`;
  const token = await runDeviceFlow({
    provider: "microsoft-graph",
    clientId,
    deviceAuthorizationUrl: `${authority}/devicecode`,
    tokenUrl: `${authority}/token`,
    scopes: ["offline_access", "User.Read", "Calendars.ReadWrite"],
    onDeviceCode: async (authorization) => {
      await copy(authorization.user_code);
      const url = authorization.verification_uri_complete || authorization.verification_uri;
      await open(url);
      console.log(`Opened Microsoft device authorization and copied ${authorization.user_code}`);
    },
    onPoll: ({ attempt }) => {
      if (attempt === 1) console.log("Waiting for Microsoft authorization...");
    },
  });

  const path = await persistDeviceToken("microsoft-graph", token);
  const action = await arg("Microsoft Graph connected", [
    {
      name: "Open App Registrations",
      description: `Saved token to ${path}`,
      value: "open",
    },
    {
      name: "Done",
      description: "Keep the saved token for Microsoft Graph API scripts",
      value: "done",
    },
  ]);

  if (action === "open") {
    await open("https://portal.azure.com/#view/Microsoft_AAD_RegisteredApps/ApplicationsListBlade");
  }

  await div(`
    <div class="p-6 space-y-3">
      <h1 class="text-xl font-semibold">Microsoft Graph Connected</h1>
      <p>Saved token to <code>${path}</code>.</p>
      <p class="text-sm opacity-70">This token can call Microsoft Graph endpoints allowed by the requested scopes.</p>
    </div>
  `);
}
