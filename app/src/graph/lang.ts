// Copied from MD Notes (app/src); ZaNytky modules stay independent of it.
import { createContext, useContext } from "react";

export type LanguageCode = "uk" | "en" | "fr" | "de" | "es" | "ko" | "ja";

export const LanguageContext = createContext<LanguageCode>("uk");

/** The same shape the MD Notes graph code expects. */
export function useI18n(): { language: LanguageCode } {
  return { language: useContext(LanguageContext) };
}
