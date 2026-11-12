# rust-wow-web

Workspace Cargo com os crates em `crates/`, os binários em `apps/` e os testes em `tests/`.

## Comandos básicos do Cargo

### Criar projetos

```sh
cargo new apps/meu-app              # novo binário (cria main.rs)
cargo new crates/meu-crate --lib    # nova biblioteca (cria lib.rs)
cargo init                          # inicializa um projeto na pasta atual
```

Dentro deste repositório o `cargo new` não cria `.git` nem `.gitignore` (ele detecta o
repositório existente). Fora de um repositório, use `--vcs none` para evitar isso.

Como o workspace usa `members = ["crates/*", "apps/*", "tests/*"]`, o novo pacote
já entra no workspace automaticamente. Para usá-lo em outro pacote, declare-o em
`[workspace.dependencies]` no `Cargo.toml` da raiz e referencie com `nome.workspace = true`.

### Build e execução

```sh
cargo build                         # compila (debug)
cargo build --release               # compila otimizado
cargo run -p auth-server            # executa um binário do workspace
cargo run --release -p auth-server  # executa em release
cargo check --workspace             # verifica erros sem gerar binário (mais rápido)
```

### Testes

```sh
cargo test --workspace              # roda todos os testes
cargo test -p infraestructure       # testes de um pacote
cargo test nome_do_teste            # testes filtrados por nome
cargo test -- --nocapture           # mostra a saída (println!) dos testes
```

### Qualidade de código

```sh
cargo fmt --all                     # formata o código
cargo fmt --all -- --check          # só verifica a formatação
cargo clippy --workspace            # lint
cargo clippy --workspace -- -D warnings   # trata warnings como erro
```

### Dependências

```sh
cargo add tokio                     # adiciona dependência ao pacote atual
cargo add tokio -p auth-server      # adiciona a um pacote específico
cargo add serde --features derive   # com features
cargo remove tokio                  # remove dependência
cargo update                        # atualiza o Cargo.lock
cargo tree                          # árvore de dependências
```

### Outros

```sh
cargo clean                         # apaga a pasta target/
cargo doc --open                    # gera e abre a documentação
cargo install nome-da-ferramenta    # instala um binário globalmente
```

> Dica: use `-p <nome>` (`--package`) para atuar em um pacote específico e
> `--workspace` para atuar em todos.
