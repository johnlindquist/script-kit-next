import { mkdir, writeFile, chmod, readFile } from "node:fs/promises";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

export type DeviceAuthorization = {
  device_code: string;
  user_code: string;
  verification_uri: string;
  verification_uri_complete?: string;
  expires_in: number;
  interval?: number;
  [key: string]: unknown;
};

export type DeviceToken = {
  access_token: string;
  token_type?: string;
  expires_in?: number;
  refresh_token?: string;
  scope?: string;
  id_token?: string;
  [key: string]: unknown;
};

export type DeviceFlowConfig = {
  provider: string;
  clientId: string;
  deviceAuthorizationUrl: string;
  tokenUrl: string;
  scopes?: string[];
  extraDeviceParams?: Record<string, string>;
  extraTokenParams?: Record<string, string>;
  timeoutSeconds?: number;
  onDeviceCode?: (authorization: DeviceAuthorization) => void | Promise<void>;
  onPoll?: (status: { attempt: number; intervalSeconds: number }) => void | Promise<void>;
};

type OAuthError = {
  error?: string;
  error_description?: string;
};

export async function runDeviceFlow(config: DeviceFlowConfig): Promise<DeviceToken> {
  if (!config.clientId.trim()) {
    throw new Error(`${config.provider} device flow requires a client id`);
  }

  const authorization = await requestDeviceAuthorization(config);
  await config.onDeviceCode?.(authorization);
  return pollForToken(config, authorization);
}

export async function persistDeviceToken(
  provider: string,
  token: DeviceToken,
): Promise<string> {
  const path = deviceTokenPath(provider);
  const dir = dirname(path);
  await mkdir(dir, { recursive: true });
  await writeFile(
    path,
    JSON.stringify(
      {
        provider,
        savedAt: new Date().toISOString(),
        token,
      },
      null,
      2,
    ),
    "utf8",
  );
  await chmod(path, 0o600).catch(() => {});
  return path;
}

export async function loadDeviceToken(provider: string): Promise<DeviceToken> {
  const path = deviceTokenPath(provider);
  let parsed: { token?: DeviceToken };
  try {
    parsed = JSON.parse(await readFile(path, "utf8")) as { token?: DeviceToken };
  } catch (error) {
    throw new Error(
      `No saved ${provider} device token at ${path}. Run the ${provider} device login example first.`,
      { cause: error },
    );
  }

  if (!parsed.token?.access_token) {
    throw new Error(`Saved ${provider} token at ${path} is missing access_token`);
  }

  return parsed.token;
}

export function deviceTokenPath(provider: string): string {
  const root = process.env.SK_PATH || join(homedir(), ".scriptkit");
  return join(root, "secrets", "device-auth", `${provider}.json`);
}

async function requestDeviceAuthorization(
  config: DeviceFlowConfig,
): Promise<DeviceAuthorization> {
  const params = new URLSearchParams({
    client_id: config.clientId,
    ...config.extraDeviceParams,
  });
  if (config.scopes?.length) params.set("scope", config.scopes.join(" "));

  const response = await fetch(config.deviceAuthorizationUrl, {
    method: "POST",
    headers: {
      Accept: "application/json",
      "Content-Type": "application/x-www-form-urlencoded",
    },
    body: params,
  });

  const body = await readJson<DeviceAuthorization & OAuthError>(response);
  if (!response.ok) throw oauthError(config.provider, body, response.status);
  if (!body.device_code || !body.user_code || !body.verification_uri) {
    throw new Error(`${config.provider} device response was missing required fields`);
  }
  return body;
}

async function pollForToken(
  config: DeviceFlowConfig,
  authorization: DeviceAuthorization,
): Promise<DeviceToken> {
  let intervalSeconds = authorization.interval || 5;
  const timeoutSeconds = Math.min(
    config.timeoutSeconds || authorization.expires_in || 900,
    authorization.expires_in || config.timeoutSeconds || 900,
  );
  const deadline = Date.now() + timeoutSeconds * 1000;
  let attempt = 0;

  while (Date.now() < deadline) {
    attempt += 1;
    await config.onPoll?.({ attempt, intervalSeconds });
    await sleep(intervalSeconds * 1000);

    const params = new URLSearchParams({
      client_id: config.clientId,
      device_code: authorization.device_code,
      grant_type: "urn:ietf:params:oauth:grant-type:device_code",
      ...config.extraTokenParams,
    });

    const response = await fetch(config.tokenUrl, {
      method: "POST",
      headers: {
        Accept: "application/json",
        "Content-Type": "application/x-www-form-urlencoded",
      },
      body: params,
    });
    const body = await readJson<DeviceToken & OAuthError>(response);

    if (response.ok && body.access_token) return body;

    switch (body.error) {
      case "authorization_pending":
        continue;
      case "slow_down":
        intervalSeconds += 5;
        continue;
      case "access_denied":
      case "expired_token":
        throw oauthError(config.provider, body, response.status);
      default:
        throw oauthError(config.provider, body, response.status);
    }
  }

  throw new Error(`${config.provider} device authorization timed out`);
}

async function readJson<T>(response: Response): Promise<T> {
  const text = await response.text();
  if (!text.trim()) return {} as T;
  return JSON.parse(text) as T;
}

function oauthError(provider: string, body: OAuthError, status: number): Error {
  const code = body.error || `http_${status}`;
  const description = body.error_description ? `: ${body.error_description}` : "";
  return new Error(`${provider} device authorization failed (${code})${description}`);
}
