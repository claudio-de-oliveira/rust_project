# 📬 Guia de Uso - Coleção Postman Clalulana API

## 📥 Importando a Coleção

1. Abra o **Postman**
2. Clique em **Import**
3. Selecione o arquivo `Clalulana_API.postman_collection.json`
4. A coleção será importada com todas as requisições estruturadas

## 🔧 Configuração das Variáveis

A coleção usa variáveis de ambiente que são automaticamente preenchidas ao executar os testes:

| Variável | Descrição | Exemplo |
|----------|-----------|---------|
| `base_url` | URL base da API | `http://localhost:8088` |
| `jwt_token` | Token JWT do usuário comum | Preenchido automaticamente ao fazer login |
| `admin_jwt_token` | Token JWT do admin | Preenchido automaticamente ao fazer login |
| `user_id` | ID do usuário criado | Preenchido automaticamente ao registrar |
| `username` | Username do usuário | Preenchido automaticamente ao registrar |
| `user_email` | Email do usuário | Preenchido automaticamente ao registrar |

## 🚀 Fluxo Recomendado de Testes

### 1️⃣ Health Check
```
GET /api/v1/health
```
Verifica se a API está rodando.

### 2️⃣ Autenticação
Execute nesta ordem para preencher os tokens:

**a) Registrar Usuário**
```
POST /api/v1/auth/register
Body: { username, email, password (min 8 chars) }
```
✅ Automaticamente salva `user_id`

**b) Registrar Admin**
```
POST /api/v1/auth/register (com dados de admin)
```

**c) Login Usuário**
```
POST /api/v1/auth/login
Body: { email, password }
```
✅ Automaticamente salva `jwt_token`

**d) Login Admin**
```
POST /api/v1/auth/login (com credenciais do admin)
```
✅ Automaticamente salva `admin_jwt_token`

### 3️⃣ Operações com Usuários

**Obter usuário atual:**
```
GET /api/v1/users/me
Headers: Authorization: Bearer {{jwt_token}}
```

**Obter todos os usuários (admin only):**
```
GET /api/v1/users?limit=10&offset=0
Headers: Authorization: Bearer {{admin_jwt_token}}
```

**Obter usuário por ID:**
```
GET /api/v1/users/{{user_id}}
Headers: Authorization: Bearer {{jwt_token}}
```

**Atualizar usuário:**
```
PUT /api/v1/users/{{user_id}}
Headers: Authorization: Bearer {{jwt_token}}
Body: { username, email }
```

**Deletar usuário (admin only):**
```
DELETE /api/v1/users/{{user_id}}
Headers: Authorization: Bearer {{admin_jwt_token}}
```

## 🔐 Segurança e Autenticação

- **Token JWT**: Válido por 3600 segundos (1 hora) por padrão
- **Bearer Token**: Use o formato `Authorization: Bearer <token>`
- **Roles**: 
  - `user`: Usuário padrão
  - `admin`: Administrador com permissões especiais

## ❌ Casos de Erro (Testing)

A coleção inclui requisições para testar casos de erro:

1. **Senha muito curta** - Registrar com `password: "123"` (menos de 8 caracteres)
2. **Credenciais inválidas** - Tentar login com email/senha errados
3. **Sem autenticação** - Acessar `/users/me` sem token
4. **Sem permissão** - Usuário comum tentando acessar endpoints admin

## 📝 Exemplo Completo de Teste

```bash
# 1. Health Check
GET http://localhost:8088/api/v1/health

# 2. Registrar
POST http://localhost:8088/api/v1/auth/register
Content-Type: application/json
{
  "username": "claudio",
  "email": "claudio@example.com",
  "password": "password123"
}

# 3. Login
POST http://localhost:8088/api/v1/auth/login
Content-Type: application/json
{
  "email": "claudio@example.com",
  "password": "password123"
}

# Resposta contém o token:
{
  "data": {
    "token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
    "token_type": "Bearer",
    "expires_in": 3600,
    "user": { ... }
  }
}

# 4. Usar o token
GET http://localhost:8088/api/v1/users/me
Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGc...
```

## 🛠️ Variáveis por Ambiente

Para usar diferentes URLs em Postman:
1. Clique em **Manage Environments**
2. Crie um novo environment (dev, prod)
3. Configure o `base_url` para cada ambiente

## 📊 Observações

- A API valida:
  - ✅ Username não vazio
  - ✅ Email válido
  - ✅ Senha mínimo 8 caracteres
  - ✅ Autenticação via JWT
  - ✅ Permissões por role

- Headers obrigatórios:
  - `Content-Type: application/json` (para POST/PUT)
  - `Authorization: Bearer <token>` (para rotas autenticadas)
