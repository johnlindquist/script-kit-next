import "@scriptkit/sdk";
import { persistDeviceToken, runDeviceFlow } from "./lib/oauth-device-flow";

export const metadata = {
  name: "Generic OAuth Device Flow",
  description: "Template for any RFC 8628 OAuth device authorization provider",
  icon: "key",
};

const isVerify = process.env.SK_VERIFY === "1";

if (isVerify) {
  console.log(JSON.stringify({ ok: true, provider: "generic-oauth-device-flow" }));
} else {
  const provider = await arg("Provider slug", "my-provider");
  const clientId = await arg("OAuth client ID");
  const deviceAuthorizationUrl = await arg("Device authorization URL");
  const tokenUrl = await arg("Token URL");
  const scopeText = await arg("Scopes (space-separated)", "openid profile email");
  let verificationUrl = "";

  const token = await runDeviceFlow({
    provider,
    clientId,
    deviceAuthorizationUrl,
    tokenUrl,
    scopes: scopeText.split(/\s+/).filter(Boolean),
    onDeviceCode: async (authorization) => {
      await copy(authorization.user_code);
      const url = authorization.verification_uri_complete || authorization.verification_uri;
      verificationUrl = url;
      await open(url);
      console.log(`Opened device authorization and copied ${authorization.user_code}`);
    },
    onPoll: ({ attempt, intervalSeconds }) => {
      if (attempt === 1) console.log(`Polling every ${intervalSeconds}s...`);
    },
  });

  const path = await persistDeviceToken(provider, token);
  const action = await arg("Device flow complete", [
    {
      name: "Open Provider",
      description: verificationUrl,
      value: "open",
    },
    {
      name: "Done",
      description: `Saved token to ${path}`,
      value: "done",
    },
  ]);

  if (action === "open") {
    await open(verificationUrl || deviceAuthorizationUrl);
  }

  await div(`
    <div class="p-6 space-y-3">
      <h1 class="text-xl font-semibold">Device Flow Complete</h1>
      <p>Saved token to <code>${path}</code>.</p>
      <p class="text-sm opacity-70">Copy this script and hard-code the provider endpoints for a production helper.</p>
    </div>
  `);
}
