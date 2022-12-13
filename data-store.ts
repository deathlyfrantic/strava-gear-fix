import { fileURLToPath } from "https://deno.land/std@0.167.0/node/url.ts";
import log from "./logger.ts";

import fileData from "./data.json" assert { type: "json" };

const store: Map<string, string | number> = new Map(Object.entries(fileData));

export const clientId = store.get("clientId") as string;
export const clientSecret = store.get("clientSecret") as string;

export function set(key: string, value: string | number) {
  store.set(key, value);
}

export function get(key: string): string | number | undefined {
  return store.get(key);
}

export function save(): Promise<void> {
  const path = fileURLToPath(
    Deno.mainModule.split("/").slice(0, -1).join("/") + "/data.json",
  );
  log.debug("Writing data file", { path });
  return Deno.writeTextFile(
    path,
    JSON.stringify(Object.fromEntries(store.entries()), null, 2),
  );
}
