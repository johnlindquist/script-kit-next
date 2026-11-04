import { createHash } from "node:crypto";

export const fileSha256 = (bytes: Uint8Array): string => createHash("sha256").update(bytes).digest("hex");

/** Hash the program, not its terminal code signature or the sizes changed by signing. */
export function unsignedMachOBytesPayloadSha256(
  bytes: Buffer,
  invalid: (detail: string) => never = detail => { throw Error("invalid_macho_payload: " + detail); },
): string {
  if (bytes.length < 32 || bytes.readUInt32LE(0) !== 0xfeedfacf) invalid("signed derivation requires a thin 64-bit Mach-O executable");
  const commands = bytes.readUInt32LE(16), commandBytes = bytes.readUInt32LE(20);
  if (commands > 4096 || 32 + commandBytes > bytes.length) invalid("Mach-O load command bounds");
  const normalized = Buffer.from(bytes);
  let cursor = 32, payloadEnd = bytes.length, signatures = 0;
  for (let index = 0; index < commands; index++) {
    if (cursor + 8 > 32 + commandBytes) invalid("truncated Mach-O command");
    const command = bytes.readUInt32LE(cursor), size = bytes.readUInt32LE(cursor + 4);
    if (size < 8 || cursor + size > 32 + commandBytes) invalid("invalid Mach-O command size");
    if (command === 0x1d) {
      if (size !== 16 || ++signatures !== 1) invalid("ambiguous Mach-O signature command");
      const offset = bytes.readUInt32LE(cursor + 8), length = bytes.readUInt32LE(cursor + 12);
      if (offset < 32 + commandBytes || offset + length !== bytes.length) invalid("signature is not the terminal Mach-O region");
      payloadEnd = offset;
      normalized.fill(0, cursor + 8, cursor + 16);
    } else if (command === 0x19 && size >= 72 && bytes.subarray(cursor + 8, cursor + 24).toString("ascii").replace(/\0+$/, "") === "__LINKEDIT") {
      normalized.fill(0, cursor + 32, cursor + 40);
      normalized.fill(0, cursor + 48, cursor + 56);
    }
    cursor += size;
  }
  if (signatures !== 1) invalid("expected compiler ad-hoc/signing signature region");
  return fileSha256(normalized.subarray(0, payloadEnd));
}
