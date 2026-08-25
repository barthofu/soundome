/** Returns a debounced wrapper around `fn` that delays invocation until `wait` ms
 * have passed without another call. Used for search-box inputs so we don't fire
 * a server request on every keystroke. */
export function debounce<A extends unknown[]>(fn: (...args: A) => void, wait = 300): (...args: A) => void {
  let timer: ReturnType<typeof setTimeout> | undefined;
  return (...args: A) => {
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => fn(...args), wait);
  };
}
