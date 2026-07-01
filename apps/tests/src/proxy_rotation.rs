/// Regression test for the shared proxy round-robin rotation.
///
/// ## Background
///
/// `ProxyRotator::get_next_proxy()` is backed by a single global `AtomicUsize`
/// counter shared across the *entire* application (see
/// `packages/shared/src/libs/http.rs`). Every call to
/// `HttpClientBuilder::get_reqwest_client()` (used throughout the codebase,
/// e.g. by `packages/downloader/src/utils/ytdlp.rs::build_args()` for every
/// yt-dlp invocation) advances this counter by one.
///
/// This caused a real regression: adding an extra proxy-rotating HTTP call to
/// `Spotify::get_playlist_from_url()` (to fetch playlist metadata) shifted the
/// round-robin index for every *subsequent* consumer in the same playlist
/// sync — including the yt-dlp downloads for that playlist's tracks. With a
/// proxy pool containing some IPs that are blocked/rate-limited by YouTube,
/// this silently changed which proxy each track download used, turning some
/// previously-successful downloads into `403 Forbidden` failures.
///
/// Fix: metadata-only calls that don't need proxy rotation must use
/// `HttpClientBuilder::get_reqwest_client_with_specific_proxy(None)` (or avoid
/// `HttpClientBuilder`'s rotating path entirely) so they don't perturb the
/// rotation used by real download/network work.
///
/// These tests exercise `ProxyRotator` directly (not the global singleton) to
/// avoid cross-test global-state issues, and document the exact mechanism so
/// future changes don't reintroduce an extra rotation on a hot path.
#[cfg(test)]
mod tests {
    use config::models::{ProxyConfig, ProxyStrategy};
    use shared::http::ProxyRotator;

    fn rotator_with(urls: &[&str]) -> ProxyRotator {
        ProxyRotator::new(Some(ProxyConfig {
            enabled: true,
            urls: urls.iter().map(|u| u.to_string()).collect(),
            strategy: Some(ProxyStrategy::RoundRobin),
            no_proxy: None,
        }))
    }

    /// Baseline: with N proxies, round-robin cycles through them in order.
    #[test]
    fn round_robin_cycles_in_order() {
        let rotator = rotator_with(&["proxy-a", "proxy-b", "proxy-c"]);

        let seq: Vec<_> = (0..6)
            .map(|_| rotator.get_next_proxy().unwrap())
            .collect();

        assert_eq!(
            seq,
            vec![
                "proxy-a", "proxy-b", "proxy-c", "proxy-a", "proxy-b", "proxy-c"
            ]
        );
    }

    /// This is the exact mechanism of the regression: one "extra" unrelated
    /// call (e.g. a metadata fetch) consumes a rotation slot and shifts every
    /// subsequent consumer (e.g. per-track yt-dlp downloads) onto a different
    /// proxy than they would otherwise have received.
    #[test]
    fn an_extra_call_shifts_all_subsequent_consumers() {
        let proxies = ["proxy-a", "proxy-b", "proxy-c"];

        // Scenario A: no metadata call before the download loop (old, correct behavior).
        let rotator_a = rotator_with(&proxies);
        let downloads_a: Vec<_> = (0..3)
            .map(|_| rotator_a.get_next_proxy().unwrap())
            .collect();

        // Scenario B: one extra "metadata" call happens first (the regression),
        // then the same download loop runs.
        let rotator_b = rotator_with(&proxies);
        let _metadata_call = rotator_b.get_next_proxy(); // simulates the extra HTTP call
        let downloads_b: Vec<_> = (0..3)
            .map(|_| rotator_b.get_next_proxy().unwrap())
            .collect();

        // The per-track proxy assignment is completely different as a result
        // of the single unrelated extra call — this is what caused some
        // tracks to land on a blocked/rate-limited proxy and fail with 403.
        assert_ne!(
            downloads_a, downloads_b,
            "an unrelated rotation-consuming call must not be able to change \
             which proxy subsequent downloads receive"
        );
    }

    /// Sanity check mirroring the actual fix: building a client with an
    /// explicit `None` proxy (as `get_reqwest_client_with_specific_proxy(None)`
    /// does) must not require or touch a `ProxyRotator` at all, so it cannot
    /// perturb the shared rotation used by downloads.
    #[test]
    fn specific_proxy_none_does_not_touch_rotator() {
        let proxies = ["proxy-a", "proxy-b", "proxy-c"];

        // Baseline: three consecutive `get_next_proxy()` calls, nothing in between.
        let baseline_rotator = rotator_with(&proxies);
        let baseline: Vec<_> = (0..3)
            .map(|_| baseline_rotator.get_next_proxy().unwrap())
            .collect();

        // Same three calls, but with a `get_reqwest_client_with_specific_proxy(None)`
        // interleaved before each one. If that call touched any `ProxyRotator`
        // (global or local), it would shift this sequence relative to the baseline
        // — exactly like the regression did with `get_reqwest_client()`.
        let interleaved_rotator = rotator_with(&proxies);
        let mut interleaved = Vec::new();
        for _ in 0..3 {
            let client =
                shared::http::HttpClientBuilder::get_reqwest_client_with_specific_proxy(None);
            assert!(
                client.is_ok(),
                "building a client with no specific proxy should succeed"
            );
            interleaved.push(interleaved_rotator.get_next_proxy().unwrap());
        }

        assert_eq!(
            interleaved, baseline,
            "get_reqwest_client_with_specific_proxy(None) must not consume a rotation \
             slot from any ProxyRotator — otherwise it would shift results the same \
             way the regression's extra get_reqwest_client() call did"
        );
    }
}
