# Utilisation du ProxyRotator Global

Ce document explique comment utiliser le ProxyRotator global dans votre application Soundome.

## Initialisation (une seule fois au démarrage)

```rust
use shared::libs::http::ProxyRotator;
use config::model::{ProxyConfig, ProxyStrategy};

// Dans votre main.rs ou lib.rs
fn init_proxy() -> Result<(), Box<dyn std::error::Error>> {
    let proxy_config = ProxyConfig {
        enabled: true,
        strategy: ProxyStrategy::RoundRobin,
        urls: vec![
            "127.0.0.1:8080:user:pass".to_string(),
            "127.0.0.1:8081:user:pass".to_string(),
        ],
        no_proxy: Some(vec!["localhost".to_string(), "127.0.0.1".to_string()]),
    };

    ProxyRotator::init_global(proxy_config)
        .map_err(|_| "Failed to initialize global proxy rotator")?;
    
    Ok(())
}
```

## Utilisation directe du ProxyRotator

```rust
use shared::libs::http::ProxyRotator;

// N'importe où dans votre code
fn get_proxy_for_request() -> Option<String> {
    let rotator = ProxyRotator::global();
    rotator.get_next_proxy()
}

// Vérifier si un domaine doit utiliser le proxy
fn should_proxy_domain(domain: &str) -> bool {
    let rotator = ProxyRotator::global();
    rotator.should_use_proxy(domain)
}
```

## Utilisation avec HttpClientBuilder

```rust
use shared::libs::http::HttpClientBuilder;

// Créer un client HTTP avec proxy automatique
async fn make_request() -> Result<String, Box<dyn std::error::Error>> {
    let client = HttpClientBuilder::build_with_global_proxy()?;
    
    let response = client
        .get("https://api.example.com/data")
        .send()
        .await?;
    
    Ok(response.text().await?)
}

// Créer un client pour un domaine spécifique
async fn make_request_to_domain(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let domain = url.split("://").nth(1).unwrap_or("").split("/").next().unwrap_or("");
    let client = HttpClientBuilder::build_for_domain(domain)?;
    
    let response = client.get(url).send().await?;
    Ok(response.text().await?)
}
```

## Utilisation sécurisée (avec vérification)

```rust
use shared::libs::http::ProxyRotator;

// Vérifier si l'instance globale est initialisée
fn safe_get_proxy() -> Option<String> {
    if let Some(rotator) = ProxyRotator::try_global() {
        rotator.get_next_proxy()
    } else {
        println!("Warning: Global proxy not initialized");
        None
    }
}

// Vérifier l'état d'initialisation
fn check_proxy_status() {
    if ProxyRotator::is_global_initialized() {
        println!("Global proxy is ready to use");
    } else {
        println!("Global proxy needs to be initialized");
    }
}
```

## Avantages de cette approche

1. **Pas de prop drilling** : Le ProxyRotator est accessible depuis n'importe où dans le code
2. **Thread-safe** : Utilise `OnceLock` qui est thread-safe
3. **Performance** : Une seule instance partagée, pas de création multiple
4. **Flexibilité** : Possibilité d'utiliser directement le rotator ou via HttpClientBuilder
5. **Sécurité** : Méthodes pour vérifier l'initialisation avant utilisation

## Notes importantes

- `ProxyRotator::init_global()` doit être appelé **une seule fois** au démarrage
- `ProxyRotator::global()` va panic si l'instance n'est pas initialisée
- Utilisez `ProxyRotator::try_global()` pour une utilisation sécurisée
- L'instance globale persiste pendant toute la durée de vie de l'application
