import { serve } from "https://deno.land/std@0.167.0/http/server.ts";
import * as data from "./data-store.ts";
import * as utils from "./utils.ts";
import log from "./logger.ts";

type TokenResponse = {
  token_type: string;
  expires_at: number;
  expires_in: number;
  refresh_token: string;
  access_token: string;
  athlete: { id: number };
};

function updateAndSaveToken(token: TokenResponse): Promise<void> {
  log.debug("Updating and saving token", { token });
  data.set("tokenExpiresAt", new Date(token.expires_at * 1000).toISOString());
  data.set("refreshToken", token.refresh_token);
  data.set("accessToken", token.access_token);
  return data.save();
}

const post = (url: string, params: utils.Params) =>
  fetch(utils.urlWithParams(url, params), { method: "POST" });

function getToken(code: string): Promise<Response> {
  const path = "https://www.strava.com/oauth/token";
  log.debug("Getting authorization tokens", { path });
  return post(path, {
    client_id: data.clientId,
    client_secret: data.clientSecret,
    code,
    grant_type: "authorization_code",
  });
}

async function handler(req: Request): Promise<Response> {
  const url = new URL(req.url);
  log.debug("Received request", { path: url.pathname });
  const params = utils.getParams(url);
  if (url.pathname === "/strava-auth" && params.code) {
    const response = await getToken(params.code);
    if (response.ok) {
      const json: TokenResponse = await response.json();
      await updateAndSaveToken(json);
      setTimeout(() => Deno.exit(0), 500);
    }
  }
  return new Response(JSON.stringify(params, null, 2));
}

function openAuthURL() {
  const url = new URL("https://www.strava.com/oauth/authorize");
  utils.setParams(url, {
    client_id: data.clientId,
    redirect_uri: "http://localhost:8000/strava-auth",
    response_type: "code",
    scope: ["activity:read_all", "activity:write"].join(","),
  });
  log.debug("Opening authorization URL", { path: url.toString() });
  Deno.run({ cmd: ["open", url.toString()] });
}

function authorize() {
  setTimeout(openAuthURL, 500);
  serve(handler);
}

export async function refreshAndSaveToken(token: string): Promise<void> {
  const path = "https://www.strava.com/oauth/token";
  log.debug("Refreshing tokens", { path });
  const response = await post(path, {
    client_id: data.clientId,
    client_secret: data.clientSecret,
    grant_type: "refresh_token",
    refresh_token: token,
  });
  if (response.ok) {
    const json: TokenResponse = await response.json();
    await updateAndSaveToken(json);
  }
}

if (import.meta.main) {
  authorize();
}
