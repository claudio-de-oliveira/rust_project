# Rust MediatR: Guia Completo de Uso

Este guia fornece uma documentação detalhada sobre como utilizar a biblioteca Rust MediatR em seus projetos. A biblioteca implementa o padrão Mediator com suporte a CQRS, Pipelines, Validação e Injeção de Dependências.

## Índice

1. [Instalação](#instalação)
2. [Conceitos Básicos](#conceitos-básicos)
3. [CQRS (Command Responsibility Segregation)](#cqrs)
4. [Notificações (Pub/Sub)](#notificações)
5. [Pipeline Behaviors (Middleware)](#pipeline-behaviors)
6. [Validação Automática](#validação)
7. [Injeção de Dependências](#injeção-de-dependências)
8. [Tratamento de Erros](#tratamento-de-erros)

---

## Instalação

Adicione a biblioteca ao seu `Cargo.toml`. Como é uma biblioteca local neste contexto, você pode referenciá-la pelo caminho:

```toml
[dependencies]
mediatr = { path = "../path/to/mediatr" }
async-trait = "0.1"
tokio = { version = "1.0", features = ["full"] }
validator = { version = "0.16", features = ["derive"] } // Opcional, para validação
```

---

## Conceitos Básicos

O padrão Mediator desacopla o envio de uma requisição do seu processamento. Você precisa definir:
1. Uma **Request** (a mensagem).
2. Um **Handler** (quem processa a mensagem).
3. O **Mediator** (quem liga os dois).

### Exemplo Simples: Ping/Pong

```rust
use mediatr::prelude::*;

// 1. Defina a Request e o tipo de Resposta
struct Ping {
    id: i32
}

impl Request for Ping {
    type Response = String;
}

// 2. Defina o Handler
struct PingHandler;

#[async_trait]
impl RequestHandler<Ping> for PingHandler {
    async fn handle(&self, request: Ping) -> Result<String> {
        Ok(format!("Pong #{}", request.id))
    }
}

// 3. Configure e use o Mediator
#[tokio::main]
async fn main() -> Result<()> {
    let mediator = Mediator::builder()
        .register_handler(PingHandler)
        .build();

    let response = mediator.send(Ping { id: 1 }).await?;
    println!("Resposta: {}", response); // "Pong #1"
    
    Ok(())
}
```

---

## CQRS

O MediatR suporta explicitamente a separação entre Comandos (escrita) e Consultas (leitura).

### Commands (Comandos)

Comandos alteram o estado do sistema.

```rust
struct CreateUserCommand {
    name: String,
    email: String,
}

impl Request for CreateUserCommand {
    type Response = u64; // Retorna o ID do usuário criado
}

impl Command for CreateUserCommand {}

struct CreateUserHandler;

#[async_trait]
impl CommandHandler<CreateUserCommand> for CreateUserHandler {
    async fn handle(&self, cmd: CreateUserCommand) -> Result<u64> {
        // Lógica de criação de usuário...
        println!("Criando usuário: {}", cmd.name);
        Ok(100) // ID fictício
    }
}
```

### Queries (Consultas)

Consultas apenas leem dados.

```rust
struct GetUserQuery {
    id: u64,
}

impl Request for GetUserQuery {
    type Response = Option<String>;
}

impl Query for GetUserQuery {}

struct GetUserHandler;

#[async_trait]
impl QueryHandler<GetUserQuery> for GetUserHandler {
    async fn handle(&self, query: GetUserQuery) -> Result<Option<String>> {
        Ok(Some("Alice".to_string()))
    }
}
```

**Registro:**
```rust
let mediator = Mediator::builder()
    .register_command_handler(CreateUserHandler)
    .register_query_handler(GetUserHandler)
    .build();
```

---

## Notificações

Diferente de requisições que têm um único handler e retornam valor, notificações podem ter **múltiplos handlers** e não retornam valor. Útil para eventos de domínio.

```rust
struct UserCreatedEvent {
    user_id: u64,
}

impl Notification for UserCreatedEvent {}

// Handler 1: Enviar Email
struct EmailHandler;
#[async_trait]
impl NotificationHandler<UserCreatedEvent> for EmailHandler {
    async fn handle(&self, event: UserCreatedEvent) -> Result<()> {
        println!("Enviando email de boas-vindas para user {}", event.user_id);
        Ok(())
    }
}

// Handler 2: Logar Auditoria
struct AuditHandler;
#[async_trait]
impl NotificationHandler<UserCreatedEvent> for AuditHandler {
    async fn handle(&self, event: UserCreatedEvent) -> Result<()> {
        println!("Auditando criação do user {}", event.user_id);
        Ok(())
    }
}

// Uso
let mediator = Mediator::builder()
    .register_notification_handler(EmailHandler)
    .register_notification_handler(AuditHandler)
    .build();

mediator.publish(UserCreatedEvent { user_id: 100 }).await?;
```

---

## Pipeline Behaviors

Pipelines, ou "comportamentos", envolvem a execução do handler. São ideais para lógica transversal (cross-cutting concerns) como logging, métricas, transações e validação.

### Usando Behaviors Integrados

A biblioteca já vem com `LoggingBehavior`, `ValidationBehavior` e `TimingBehavior`.

```rust
let mediator = Mediator::builder()
    .register_handler(MyHandler)
    // A ordem importa! Eles são executados na ordem de registro.
    .add_behavior(LoggingBehavior) // Loga entrada/saída
    .add_behavior(ValidationBehavior) // Valida a request antes de processar
    .add_behavior(TimingBehavior) // Mede o tempo de execução
    .build();
```

### Criando um Behavior Customizado

```rust
struct TransactionBehavior;

#[async_trait]
impl<R: Request> PipelineBehavior<R> for TransactionBehavior {
    async fn handle<'a>(
        &'a self,
        request: R,
        next: RequestDelegate<'a, R>,
    ) -> Result<R::Response> {
        println!("Iniciando transação...");
        // Chama o próximo behavior ou o handler
        let result = next(request).await;
        
        if result.is_ok() {
            println!("Commit da transação.");
        } else {
            println!("Rollback da transação.");
        }
        
        result
    }
}
```

---

## Validação

Integração direta com a crate `validator`. Se sua request derivar `Validate`, você pode usar o `ValidationBehavior` para rejeitar requisições inválidas automaticamente.

```rust
use validator::Validate;

#[derive(Validate)]
struct RegisterProduct {
    #[validate(length(min = 3))]
    name: String,
    
    #[validate(range(min = 0, max = 1000))]
    price: i32,
}

impl Request for RegisterProduct { type Response = (); }

// ... registrar handler ...

let mediator = Mediator::builder()
    .register_handler(ProductHandler)
    .add_behavior(ValidationBehavior) // Ativa a validação
    .build();

// Isso retornará um erro de validação (ValidationError)
mediator.send(RegisterProduct { name: "A".into(), price: -10 }).await?;
```

---

## Injeção de Dependências

A biblioteca inclui um container IoC simples (`mediatr::di::Container`) que pode ser usado para gerenciar dependências dos seus handlers.

```rust
use mediatr::di::{Container, ServiceLifetime};

// 1. Defina um serviço
trait Repository: Send + Sync {
    fn save(&self, data: &str);
}

struct DbRepository;
impl Repository for DbRepository {
    fn save(&self, data: &str) { println!("Salvando no banco: {}", data); }
}

// 2. Configure o Container
let mut container = Container::new();
container.register_singleton::<DbRepository, _>(|_| DbRepository);

// 3. Resolva dependências (Manual)
// Em uma aplicação real, você injetaria isso nos seus handlers
// no momento da criação.
let repo = container.resolve::<DbRepository>().unwrap();
repo.save("dados");
```

---

## Tratamento de Erros

Todas as operações retornam `mediatr::Result<T>`. Os principais tipos de erro são:

- `Error::HandlerNotFound`: Quando você tenta enviar uma request sem handler registrado.
- `Error::Validation`: Quando a validação automática falha.
- `Error::Generic`: Para erros gerais dentro dos handlers.

Recomendamos usar a crate `thiserror` ou `anyhow` em seus handlers e converter erros de domínio para erros do MediatR quando necessário, ou simplesmente retornar strings de erro que serão convertidas automaticamente (devido à implementação de `From` para `Box<dyn StdError>`).
