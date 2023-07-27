# moto-trackr-route-api

## Description

API en Rust utilisant le framework Actix-web. Elle permet de créer des itinéraires.

## Documentation

Pour ouvrir la documentation, il faut ouvrir le fichier /target/doc/moto_trackr_route_api/index.html dans un navigateur.

## Installation

- Build Docker

```
docker build -t moto-trackr-route-api .
```

- Démarrer l'image Docker

  ```bash
  docker run -p 8000:8000 moto-trackr-route-api:latest
  ```

## Développement

- Ajouter les dépendances dans le fichier Cargo.toml

  - ```toml
      [dependencies]
      rand = "0.8.4"
    ```
  - ```bash
      cargo add rand
    ```

  - Compiler le projet

  ```bash
    cargo build
  ```

  - Vérifier que le projet est compilable

  ```bash
  cargo check
  ```

  - Démarrer le projet

  ```bash
  cargo shuttle run
  ```

  - Déployer

  ```
  cargo shuttle deploy
  ```

- Generate documentation

  ```bash
  cargo doc --document-private-items --no-deps --open
  ```
