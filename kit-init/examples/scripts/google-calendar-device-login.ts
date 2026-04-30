import "@scriptkit/sdk";
import { persistDeviceToken, runDeviceFlow } from "./lib/oauth-device-flow";

export const metadata = {
  name: "Google Calendar Device Login",
  description: "Sign in with Google's OAuth device flow and save a Calendar API token locally",
  icon: "calendar-days",
};

const isVerify = process.env.SK_VERIFY === "1";
const clientId = process.env.GOOGLE_OAUTH_CLIENT_ID || "";

if (isVerify) {
  console.log(
    JSON.stringify({
      ok: true,
      provider: "google-calendar",
      needs: "GOOGLE_OAUTH_CLIENT_ID",
    }),
  );
} else if (!clientId) {
  await div(`
    <div class="p-6 space-y-3">
      <h1 class="text-xl font-semibold">Google Calendar Device Login</h1>
      <p>Set <code>GOOGLE_OAUTH_CLIENT_ID</code> to an OAuth client ID that allows device flow, then run this script again.</p>
      <p class="text-sm opacity-70">Requested scope: <code>https://www.googleapis.com/auth/calendar.events</code>.</p>
    </div>
  `);
} else {
  const token = await runDeviceFlow({
    provider: "google-calendar",
    clientId,
    deviceAuthorizationUrl: "https://oauth2.googleapis.com/device/code",
    tokenUrl: "https://oauth2.googleapis.com/token",
    scopes: ["https://www.googleapis.com/auth/calendar.events"],
    onDeviceCode: async (authorization) => {
      await copy(authorization.user_code);
      const url = authorization.verification_uri_complete || authorization.verification_uri;
      await open(url);
      console.log(`Opened Google device authorization and copied ${authorization.user_code}`);
    },
    onPoll: ({ attempt }) => {
      if (attempt === 1) console.log("Waiting for Google authorization...");
    },
  });

  const path = await persistDeviceToken("google-calendar", token);
  const action = await arg("Google Calendar connected", [
    {
      name: "Open Calendar",
      description: `Saved token to ${path}`,
      value: "calendar",
    },
    {
      name: "Done",
      description: "Use ;gcal to create events through the Calendar API",
      value: "done",
    },
  ]);

  if (action === "calendar") {
    await open("https://calendar.google.com/calendar/u/0/r");
  }

  await div(`
    <div class="p-6 space-y-3">
      <h1 class="text-xl font-semibold">Google Calendar Connected</h1>
      <p>Saved token to <code>${path}</code>.</p>
      <p class="text-sm opacity-70">The <code>;gcal</code> example can now insert events through the Google Calendar API.</p>
    </div>
  `);
}
