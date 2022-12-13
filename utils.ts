export type Params = Record<string, string>;
export type Method = "GET" | "POST" | "PUT";

export function getParams(url: URL): { [k: string]: string } {
  const params = new URLSearchParams(url.search);
  return Object.fromEntries(params.entries());
}

export function setParams(url: URL, params?: Params) {
  for (const [key, value] of Object.entries(params ?? {})) {
    url.searchParams.set(key, value);
  }
}

export function urlWithParams(url: string, params?: Params) {
  const ret = new URL(url);
  setParams(ret, params);
  return ret;
}
