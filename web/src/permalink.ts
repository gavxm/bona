import { deflate, inflate } from "pako";
import type { ModelInvestigation } from "./types";
import { SCHEMA_VERSION } from "./types";

/** Encode an investigation snapshot into a URL fragment string. */
export function encodePermalink(
  investigation: ModelInvestigation,
  findingId?: string | null,
): string {
  const json = JSON.stringify(investigation);
  const compressed = deflate(new TextEncoder().encode(json));
  const encoded = base64UrlEncode(compressed);

  const base = window.location.origin + window.location.pathname;
  let hash = `#s=${encoded}`;
  if (findingId) hash += `&f=${encodeURIComponent(findingId)}`;
  return `${base}${hash}`;
}

/** Decode a URL fragment into an investigation and optional finding id.
 *  Returns null on any error (malformed, corrupt, etc.). */
export function decodePermalink(
  hash: string,
): { investigation: ModelInvestigation; findingId: string | null; versionMismatch: boolean } | null {
  try {
    const stripped = hash.startsWith("#") ? hash.slice(1) : hash;
    const params = new URLSearchParams(stripped);
    const encoded = params.get("s");
    if (!encoded) return null;

    const compressed = base64UrlDecode(encoded);
    const json = new TextDecoder().decode(inflate(compressed));
    const investigation: ModelInvestigation = JSON.parse(json);

    const findingId = params.get("f") ?? null;
    const versionMismatch = investigation.schema_version !== SCHEMA_VERSION;

    return { investigation, findingId, versionMismatch };
  } catch {
    return null;
  }
}

function base64UrlEncode(bytes: Uint8Array): string {
  let binary = "";
  for (const b of bytes) binary += String.fromCharCode(b);
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

function base64UrlDecode(str: string): Uint8Array {
  const padded = str.replace(/-/g, "+").replace(/_/g, "/");
  const binary = atob(padded);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}
