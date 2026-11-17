/** The e2e stand-in for the app's old origin: `playwright.config.ts` serves
 * the bridge page (`vite.bridge.config.ts`) here, pointed at `NEW_ORIGIN`. */
export const OLD_ORIGIN = 'http://localhost:5184'

/** The e2e app itself (Playwright's `baseURL`). */
export const NEW_ORIGIN = 'http://localhost:5183'
