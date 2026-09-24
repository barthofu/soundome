/** Returns a debounced wrapper around `fn` that delays invocation until `wait` ms
 * have passed without another call. Used for search-box inputs so we don't fire
 * a server request on every keystroke. */
export type Debounced<A extends unknown[]> = ((...args: A) => void) & { cancel: () => void };

export function debounce<A extends unknown[]>(fn: (...args: A) => void, wait = 300): Debounced<A> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  return Object.assign(
    (...args: A) => {
      if (timer !== undefined) clearTimeout(timer);
      timer = setTimeout(() => {
        timer = undefined;
        fn(...args);
      }, wait);
    },
    {
      cancel: () => {
        if (timer !== undefined) clearTimeout(timer);
        timer = undefined;
      },
    },
  );
}
