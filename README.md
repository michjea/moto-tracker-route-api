## Installation

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
  cargo run
  cargo shuttle run
  ```

- Déployer
  ```
  cargo shuttle deploy
  ```
