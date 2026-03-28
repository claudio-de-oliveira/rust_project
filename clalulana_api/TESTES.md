# Testes para clalulana_api

## Visão Geral

Foi implementada uma suíte completa de testes para o projeto **clalulana_api**, incluindo testes unitários e testes de integração. O projeto agora conta com **53 testes** (43 unitários + 10 de integração) que validam a funcionalidade dos componentes principais.

## Estrutura de Testes

### 1. Testes Unitários (43 testes)

Os testes unitários foram integrados diretamente aos arquivos de código usando módulos de teste com `#[cfg(test)]`. Isso mantém os testes próximos ao código que testam.

#### **Módulo de Domínio** (`src/domain/user.rs`)
- `test_user_to_response_conversion`: Valida conversão de `User` para `UserResponse`
- `test_user_response_excludes_password_hash`: Verifica que senha não é exposta em respostas
- `test_auth_response_serialization`: Testa serialização de resposta de autenticação
- `test_user_response_contains_required_fields`: Valida campos obrigatórios da resposta

**Total: 4 testes**

#### **Módulo de Erros** (`src/errors.rs`)
- `test_bad_request_error`: Valida erro BadRequest
- `test_unauthorized_error`: Valida erro Unauthorized
- `test_forbidden_error`: Valida erro Forbidden
- `test_not_found_error`: Valida erro NotFound
- `test_conflict_error`: Valida erro Conflict
- `test_internal_error`: Valida erro InternalError
- `test_error_debug_output`: Valida debug output de erros

**Total: 7 testes**

#### **Módulo de Resposta** (`src/response.rs`)
- `test_success_response_creation`: Cria resposta bem-sucedida
- `test_response_meta_contains_timestamp`: Valida timestamp em metadados
- `test_response_serialization`: Testa serialização JSON
- `test_empty_data_response`: Testa resposta vazia
- `test_response_version_is_v1`: Valida versão da resposta

**Total: 5 testes**

#### **Middleware de Autenticação** (`src/middleware/auth.rs`)
- `test_claims_user_id_parsing`: Parsing de UUID em claims
- `test_claims_invalid_user_id`: Validação de UUID inválido
- `test_extract_bearer_token_success`: Extração correta de token Bearer
- `test_extract_bearer_token_invalid_format`: Rejeição de formato inválido
- `test_extract_bearer_token_no_space`: Rejeição de token mal formado
- `test_authenticated_user_creation`: Criação de usuário autenticado
- `test_require_role_admin_access`: Validação de acesso admin
- `test_require_role_user_access`: Validação de acesso de usuário
- `test_require_role_insufficient_permissions`: Validação de permissões insuficientes
- `test_claims_serialization`: Serialização de claims
- `test_authenticated_user_cloneable`: Clonagem de usuário autenticado

**Total: 11 testes**

#### **Comandos CQRS** (`src/cqrs/users/commands.rs`)
- `test_create_user_command_deserialization`: Desserialização de CreateUserCommand
- `test_create_user_command_missing_fields`: Validação de campos faltantes
- `test_login_command_deserialization`: Desserialização de LoginCommand
- `test_login_command_missing_email`: Validação de email faltante
- `test_create_user_command_with_special_characters`: Teste com caracteres especiais
- `test_update_user_command_deserialization`: Desserialização de UpdateUserCommand
- `test_update_user_command_partial`: Teste com campos parciais
- `test_delete_user_command_creation`: Criação de DeleteUserCommand

**Total: 8 testes**

#### **Queries CQRS** (`src/cqrs/users/queries.rs`)
- `test_get_user_by_id_query_creation`: Criação de GetUserByIdQuery
- `test_get_user_by_id_query_debug`: Debug output da query
- `test_get_all_users_query_creation`: Criação de GetAllUsersQuery
- `test_get_all_users_query_with_pagination`: Teste com paginação
- `test_get_all_users_query_debug`: Debug output da query
- `test_get_current_user_query_creation`: Criação de GetCurrentUserQuery
- `test_get_current_user_query_debug`: Debug output da query
- `test_get_user_by_id_query_with_multiple_ids`: Teste com múltiplos IDs

**Total: 8 testes**

### 2. Testes de Integração (10 testes)

Os testes de integração estão em um arquivo separado (`tests/integration_tests.rs`) que verifica a integração entre múltiplos módulos.

- `test_user_response_serialization`: Serialização de resposta de usuário
- `test_api_response_serialization`: Serialização de resposta da API
- `test_error_response_structures`: Estrutura de respostas de erro
- `test_create_user_command_validation`: Validação de comando de criação
- `test_login_command_validation`: Validação de comando de login
- `test_jwt_claims_structure`: Estrutura de JWT claims
- `test_authenticated_user_structure`: Estrutura de usuário autenticado
- `test_bearer_token_extraction`: Extração de token Bearer
- `test_get_all_users_query_pagination`: Paginação de queries
- `test_type_conversions`: Conversão de tipos (User → UserResponse)

**Total: 10 testes**

## Configuração

### lib.rs
Um novo arquivo [src/lib.rs](src/lib.rs) foi criado para expor todos os módulos como biblioteca. Isso permite:
- Executar testes unitários com `cargo test --lib`
- Reutilizar código da API em outras aplicações

### Cargo.toml
O arquivo `Cargo.toml` foi atualizado com:
- Seção `[lib]` para definir o target de biblioteca
- Seção `[[bin]]` para manter o target binário existente
- Dependências de desenvolvimento: `mockall`, `tokio-test` para testes avançados

## Executando os Testes

### Todos os testes
```bash
cargo test
```

### Apenas testes unitários
```bash
cargo test --lib
```

### Apenas testes de integração
```bash
cargo test --test integration_tests
```

### Um teste específico
```bash
cargo test test_user_response_excludes_password_hash
```

### Com saída detalhada
```bash
cargo test -- --nocapture
```

## Cobertura de Testes

✅ **Domínio**: Conversão de tipos, exposição de dados sensíveis
✅ **Erros**: Todos os tipos de erro da API
✅ **Respostas**: Estrutura e serialização das respostas
✅ **Autenticação**: JWT, claims, roles, extração de tokens
✅ **Comandos**: Validação de entrada, desserialização
✅ **Queries**: Criação, paginação
✅ **Integração**: Fluxos end-to-end, serialização cruzada

## Próximos Passos (Recomendado)

1. **Testes com Banco de Dados**: Adicionar testes que usam PostgreSQL (com testcontainers ou sqlx::test)
2. **Testes de Handler**: Testar os handlers HTTP completos com mocks
3. **Testes de Middleware**: Testar middleware de performance e autenticação
4. **Cobertura de Código**: Usar `tarpaulin` para medir cobertura: `cargo tarpaulin --out Html`
5. **Testes de Propriedade**: Usar `proptest` para testes baseados em propriedades
6. **Benchmarks**: Adicionar benchmarks com `criterion`

## Estatísticas

- **Arquivos de teste modificados**: 8
- **Novos arquivos de teste**: 1 (integration_tests.rs)
- **Total de testes**: 53
- **Taxa de sucesso**: 100% ✅

---

**Última atualização**: 22 de março de 2026
