/**
 * Svelte action: observes the element it's attached to and invokes `onIntersect`
 * whenever it scrolls into view. Used as a bottom-of-list sentinel to trigger
 * loading the next page (infinite scroll) for the library's tracks/albums/
 * artists/playlists tabs.
 *
 * Usage: `<div use:infiniteScroll={() => loadMore()}></div>` placed as the
 * last element of a scrollable list.
 */
export function infiniteScroll(node: HTMLElement, onIntersect: () => void) {
  let callback = onIntersect;

  const observer = new IntersectionObserver(
    (entries) => {
      if (entries.some((e) => e.isIntersecting)) callback();
    },
    { root: null, rootMargin: '200px', threshold: 0 },
  );
  observer.observe(node);

  return {
    update(next: () => void) {
      callback = next;
    },
    destroy() {
      observer.disconnect();
    },
  };
}
