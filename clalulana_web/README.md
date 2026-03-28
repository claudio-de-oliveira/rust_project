# 🚀 Clalulana API - Testador Web

Interface moderna e intuitiva para testar a API REST Clalulana com CQRS pattern.

## 🎯 Funcionalidades

- ✓ Health Check - Verifica o status da API
- 🔐 Autenticação - Login e Registro de usuários
- 👥 Gerenciar Usuários - CRUD completo
  - Listar todos os usuários
  - Criar novos usuários
  - Buscar usuários por ID
  - Editar usuários
  - Deletar usuários

## 🛠️ Tech Stack

- **Frontend:** React 18 com TypeScript
- **Build Tool:** Vite
- **HTTP Client:** Axios
- **Styling:** CSS Nativo (sem dependências)

## 📦 Instalação

```bash
cd clalulana_web
npm install
```

## 🚀 Desenvolvimento

```bash
npm run dev
```

A aplicação estará disponível em `http://localhost:5173`

## 🏗️ Build

```bash
npm run build
```

## 🔗 Integração com API

A aplicação se conecta à API em `http://localhost:8088/api/v1`

### Endpoints utilizados:

| Método | Endpoint | Autenticação | Descrição |
|--------|----------|--------------|-----------|
| GET | `/health` | Não | Verificar status |
| POST | `/auth/register` | Não | Registrar usuário |
| POST | `/auth/login` | Não | Fazer login |
| GET | `/users` | Sim | Listar usuários |
| GET | `/users/me` | Sim | Usuário atual |
| GET | `/users/{id}` | Sim | Buscar por ID |
| PUT | `/users/{id}` | Sim | Atualizar usuário |
| DELETE | `/users/{id}` | Sim | Deletar usuário |

## 💡 Como Usar

1. **Health Check:** Clique em "Fazer Health Check" para verificar se a API está rodando
2. **Registrar:** Vá para Autenticação > Registrar e crie uma nova conta
3. **Login:** Faça login com suas credenciais
4. **Gerenciar Usuários:** Após autenticado, acesse a aba "Usuários"

## 📝 Credenciais de Teste

A interface fornece credenciais padrão para testes:
- Email: `teste@email.com`
- Senha: `senha123`

## 🎨 Design

- Interface limpa e minimalista
- Tema gradiente moderno (roxo)
- Responsivo para mobile
- Feedback visual claro (loading, erros, sucesso)
- Animações suaves

## 📄 Estrutura de Arquivos

```
clalulana_web/
├── src/
│   ├── components/
│   │   ├── Auth.tsx        # Autenticação (login/registrar)
│   │   ├── Health.tsx      # Health check
│   │   └── Users.tsx       # Gerenciamento de usuários
│   ├── api.ts              # Configuração do cliente HTTP
│   ├── App.tsx             # Componente principal
│   ├── main.tsx            # Entry point
│   └── index.css            # Estilos globais
├── index.html
├── vite.config.ts
├── tsconfig.json
└── package.json
```
