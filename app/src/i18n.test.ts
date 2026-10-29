import { describe, expect, it } from "vitest";
import { fill, LANGS, PACKS } from "./i18n";

const reference = PACKS.uk as Record<string, string>;
const referenceKeys = Object.keys(reference).sort();

function placeholders(text: string): string[] {
  return Array.from(text.matchAll(/\{(\w+)\}/g), (m) => m[1]).sort();
}

describe("interface languages", () => {
  it("lists seven languages with a pack each", () => {
    expect(LANGS.map((l) => l.code).sort()).toEqual(["de", "en", "es", "fr", "ja", "ko", "uk"]);
    for (const { code } of LANGS) expect(PACKS[code]).toBeDefined();
  });

  for (const { code } of LANGS) {
    const pack = PACKS[code] as Record<string, string>;

    it(`${code}: has exactly the key set of the Ukrainian reference`, () => {
      expect(Object.keys(pack).sort()).toEqual(referenceKeys);
    });

    it(`${code}: has no empty strings`, () => {
      const empty = referenceKeys.filter((key) => !String(pack[key] ?? "").trim());
      expect(empty).toEqual([]);
    });

    it(`${code}: keeps the same {placeholders} as the reference`, () => {
      const different = referenceKeys.filter(
        (key) => JSON.stringify(placeholders(pack[key] ?? "")) !== JSON.stringify(placeholders(reference[key])),
      );
      expect(different).toEqual([]);
    });
  }
});

describe("fill", () => {
  it("replaces placeholders and leaves unknown ones empty", () => {
    expect(fill("a {x} b {y}", { x: 1 })).toBe("a 1 b ");
  });
});
