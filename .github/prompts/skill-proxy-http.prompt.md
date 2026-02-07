---
name: "Skill: Proxy-aware HTTP"
description: "Aide à rendre un appel HTTP compatible proxy/rotation dans Soundome."
---

Objectif : utiliser le proxy Soundome correctement.

Étapes :
1) Vérifier que `shared::init_globals()` est appelé au boot.
2) Utiliser `shared::libs::http::HttpClientBuilder` pour créer un `reqwest::Client`.
3) Vérifier si le domaine doit bypass (no_proxy) avec `ProxyRotator::should_use_proxy`.
4) Ne pas logger d’URL proxy complète avec credentials.

Réfs : docs/proxy-configuration.md
