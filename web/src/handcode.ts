// Hand codes: a short name for an exact moment of a game, to ask about it later. The host (the
// program serving this app) keeps the game under the code in %LOCALAPPDATA%/500/hands, where
// `fh-lab show --code ...` reads it. Without a host the code carries the game itself ("500:…").

/** The save cut to its first `at` actions: the game as it stood at that moment. */
export function saveAt(save: string, at: number): string {
  const data = JSON.parse(save) as { log: number[] };
  return JSON.stringify({ ...data, log: data.log.slice(0, at) });
}

const CROCKFORD = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/** Eight characters (40 bits of a 64-bit FNV-1a hash), as XXXX-XXXX. */
function shortCode(text: string): string {
  let h = 0xcbf29ce484222325n;
  for (const byte of new TextEncoder().encode(text)) {
    h ^= BigInt(byte);
    h = (h * 0x100000001b3n) & 0xffffffffffffffffn;
  }
  let code = "";
  for (let i = 0; i < 8; i++) {
    code += CROCKFORD[Number((h >> BigInt(59 - 5 * i)) & 31n)];
  }
  return `${code.slice(0, 4)}-${code.slice(4)}`;
}

function base64url(text: string): string {
  let binary = "";
  for (const byte of new TextEncoder().encode(text)) binary += String.fromCharCode(byte);
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

/** A code for the game `save` as it stood after `at` actions. With a host it's short and kept there. */
export async function handCode(save: string, at: number, hosted: boolean): Promise<string> {
  const position = saveAt(save, at);
  if (hosted) {
    const code = shortCode(position);
    try {
      const response = await fetch("/api/hands", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ code, save: position }),
      });
      if (response.ok) return code;
    } catch {
      // Fall back to a code that carries the game.
    }
  }
  return `500:${base64url(position)}`;
}
