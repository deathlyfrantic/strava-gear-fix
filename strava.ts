import { refreshAndSaveToken } from "./auth.ts";
import * as data from "./data-store.ts";
import * as utils from "./utils.ts";
import log from "./logger.ts";

export type Activity = {
  id: number;
  name: string;
  type: string;
  trainer: boolean;
  gear_id: string;
  sport_type: string;
  start_date: string;
};

const BASE_URL = "https://www.strava.com/api/v3/";

async function request<T>(
  method: utils.Method,
  path: string,
  { params, body }: { params?: utils.Params; body?: Record<string, unknown> },
): Promise<T> {
  const tokenExpiresAt = data.get("tokenExpiresAt");
  const accessToken = data.get("accessToken");
  const refreshToken = data.get("refreshToken");
  if (typeof accessToken !== "string") {
    throw new Error("No access token found. Need to authorize application.");
  }
  if (typeof refreshToken !== "string") {
    throw new Error("No refresh token found. Need to authorize application.");
  }
  if (!tokenExpiresAt) {
    throw new Error(
      "No token expiration date found. Need to authorize application.",
    );
  }
  const dateExpires = new Date(tokenExpiresAt);
  // add five seconds for clock drift etc
  if ((Date.now() + 5000) > dateExpires.valueOf()) {
    await refreshAndSaveToken(refreshToken);
  }
  log.debug("Making request to Strava", {
    url: BASE_URL + path,
    path,
    method,
    params,
    body,
    tokenExpiresAt,
    refreshToken,
    accessToken,
  });
  const options: {
    method: utils.Method;
    headers: Record<string, string>;
    body?: string;
  } = {
    method,
    headers: { Authorization: `Bearer ${accessToken}` },
  };
  if (body) {
    options.headers["Content-Type"] = "application/json";
    options.body = JSON.stringify(body);
  }
  const response = await fetch(
    utils.urlWithParams(BASE_URL + path, params),
    options,
  );
  if (!response.ok) {
    let json: { errors?: { field?: string; code?: string }[] } = {};
    try {
      json = await response.json();
    } catch (error) {
      log.debug("Error trying to decode JSON from response", { error });
    }
    if (
      response.status === 401 && response.statusText === "Unauthorized" &&
      json.errors?.some((error) =>
        error.field === "access_token" && error.code === "invalid"
      )
    ) {
      log.debug("Received invalid token response. Refreshing access token.");
      await refreshAndSaveToken(refreshToken);
      return request<T>(method, path, { params, body });
    }
    throw new Error(
      `${response.status} ${response.statusText}: ${await response.text()}`,
    );
  }
  return response.json();
}

const get = <T>(path: string, params?: utils.Params) =>
  request<T>("GET", path, { params });

const put = <T>(
  path: string,
  { params, body }: { params?: utils.Params; body?: Record<string, unknown> },
) => request<T>("PUT", path, { params, body });

export function getActivitiesSince(since: Date | string): Promise<Activity[]> {
  if (typeof since === "string") {
    since = new Date(since);
  }
  const after = Math.floor(since.valueOf() / 1000).toString();
  // strava wants `after` to be seconds so we need to divide it by 1000
  return get<Activity[]>("athlete/activities", { after });
}

export function updateActivity(
  id: number,
  body: Record<string, unknown>,
): Promise<void> {
  return put<void>(`activities/${id}`, { body });
}

export async function getBike(id: string) {
  log.info(`bike ${id}`, await get<Record<string, unknown>>(`gear/${id}`));
}
