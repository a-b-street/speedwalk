import { type Writable, writable } from "svelte/store";

/**
 * Creates a localStorage-backed writable store.
 * The value is automatically synced with localStorage on every change.
 * On initialization, the value is loaded from localStorage if available.
 */
export function localStorageStore<T>(
  key: string,
  defaultValue: T,
): Writable<T> {
  const storageKey = `speedwalk_${key}`;

  // Try to read initial value from localStorage
  let initialValue = defaultValue;
  try {
    const stored = localStorage.getItem(storageKey);
    if (stored !== null) {
      initialValue = JSON.parse(stored) as T;
    }
  } catch (err) {
    console.warn(`Failed to read ${storageKey} from localStorage:`, err);
  }

  // Create the store
  const store = writable<T>(initialValue);

  // Subscribe to changes and write to localStorage
  store.subscribe((value) => {
    try {
      localStorage.setItem(storageKey, JSON.stringify(value));
    } catch (err) {
      // Handle quota exceeded or other localStorage errors
      if (err instanceof DOMException && err.name === "QuotaExceededError") {
        console.warn(`localStorage quota exceeded for ${storageKey}`);
      } else {
        console.warn(`Failed to write ${storageKey} to localStorage:`, err);
      }
    }
  });

  return store;
}
