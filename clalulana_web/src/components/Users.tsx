import { useState, useEffect } from 'react';
import * as api from '../api';

interface UsersProps {
  token: string;
}

interface User {
  id: string;
  email: string;
  name: string;
  created_at: string;
  updated_at: string;
}

export default function Users({ token }: UsersProps) {
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);
  const [activeView, setActiveView] = useState<'list' | 'create' | 'search'>('list');

  // Form states
  const [createName, setCreateName] = useState('');
  const [createEmail, setCreateEmail] = useState('');
  const [searchId, setSearchId] = useState('');
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState('');
  const [editEmail, setEditEmail] = useState('');

  useEffect(() => {
    if (activeView === 'list') {
      fetchUsers();
    }
  }, [activeView]);

  const fetchUsers = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await api.getAllUsers();
      setUsers(result);
    } catch (err: any) {
      setError(err.response?.data?.message || err.message || 'Erro ao buscar usuários');
    } finally {
      setLoading(false);
    }
  };

  const handleCreateUser = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      await api.register({
        name: createName,
        email: createEmail,
        password: 'password123', // Default password
      });
      setSuccess('Usuário criado com sucesso!');
      setCreateName('');
      setCreateEmail('');
      setActiveView('list');
      await fetchUsers();
    } catch (err: any) {
      setError(err.response?.data?.message || err.message || 'Erro ao criar usuário');
    } finally {
      setLoading(false);
    }
  };

  const handleSearchUser = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const user = await api.getUserById(searchId);
      setUsers([user]);
      setSuccess('Usuário encontrado!');
    } catch (err: any) {
      setError(err.response?.data?.message || err.message || 'Usuário não encontrado');
      setUsers([]);
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteUser = async (id: string) => {
    if (!window.confirm('Tem certeza que deseja deletar este usuário?')) return;

    setError(null);
    setSuccess(null);

    try {
      await api.deleteUser(id);
      setSuccess('Usuário deletado com sucesso!');
      await fetchUsers();
    } catch (err: any) {
      setError(err.response?.data?.message || err.message || 'Erro ao deletar usuário');
    }
  };

  const handleUpdateUser = async (e: React.FormEvent, id: string) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      await api.updateUser(id, {
        name: editName || undefined,
        email: editEmail || undefined,
      });
      setSuccess('Usuário atualizado com sucesso!');
      setEditingId(null);
      setEditName('');
      setEditEmail('');
      await fetchUsers();
    } catch (err: any) {
      setError(err.response?.data?.message || err.message || 'Erro ao atualizar usuário');
    } finally {
      setLoading(false);
    }
  };

  const startEdit = (user: User) => {
    setEditingId(user.id);
    setEditName(user.name);
    setEditEmail(user.email);
  };

  return (
    <div>
      <h2 style={{ marginBottom: '20px' }}>👥 Gerenciar Usuários</h2>

      {error && <div className="error-message">{error}</div>}
      {success && <div className="success-message">{success}</div>}

      <div className="tabs" style={{ marginBottom: '20px' }}>
        <button
          className={`tab-button ${activeView === 'list' ? 'active' : ''}`}
          onClick={() => setActiveView('list')}
        >
          Listar Usuários
        </button>
        <button
          className={`tab-button ${activeView === 'create' ? 'active' : ''}`}
          onClick={() => setActiveView('create')}
        >
          Criar Usuário
        </button>
        <button
          className={`tab-button ${activeView === 'search' ? 'active' : ''}`}
          onClick={() => setActiveView('search')}
        >
          Buscar Usuário
        </button>
      </div>

      {activeView === 'list' && (
        <div>
          <button className="secondary" onClick={fetchUsers} style={{ marginBottom: '15px' }}>
            🔄 Recarregar
          </button>

          {loading && (
            <div className="loading">
              <span className="spinner"></span> Carregando usuários...
            </div>
          )}

          {!loading && users.length === 0 && (
            <div className="info-message">Nenhum usuário encontrado.</div>
          )}

          {users.map((user) =>
            editingId === user.id ? (
              <form
                key={user.id}
                onSubmit={(e) => handleUpdateUser(e, user.id)}
                style={{
                  background: '#f0f0f0',
                  padding: '15px',
                  borderRadius: '6px',
                  marginBottom: '15px',
                }}
              >
                <div className="form-group">
                  <label>Nome</label>
                  <input
                    type="text"
                    value={editName}
                    onChange={(e) => setEditName(e.target.value)}
                  />
                </div>
                <div className="form-group">
                  <label>Email</label>
                  <input
                    type="email"
                    value={editEmail}
                    onChange={(e) => setEditEmail(e.target.value)}
                  />
                </div>
                <div className="button-group">
                  <button type="submit" className="success" disabled={loading}>
                    Salvar
                  </button>
                  <button
                    type="button"
                    className="secondary"
                    onClick={() => setEditingId(null)}
                  >
                    Cancelar
                  </button>
                </div>
              </form>
            ) : (
              <div key={user.id} className="user-card">
                <div className="user-info">
                  <h3>{user.name}</h3>
                  <p>Email: {user.email}</p>
                  <p style={{ fontSize: '12px', color: '#999' }}>
                    ID: {user.id}
                  </p>
                  <p style={{ fontSize: '12px', color: '#999' }}>
                    Criado em: {new Date(user.created_at).toLocaleString('pt-BR')}
                  </p>
                </div>
                <div className="user-actions">
                  <button
                    className="secondary"
                    onClick={() => startEdit(user)}
                    disabled={loading}
                  >
                    Editar
                  </button>
                  <button
                    className="danger"
                    onClick={() => handleDeleteUser(user.id)}
                    disabled={loading}
                  >
                    Deletar
                  </button>
                </div>
              </div>
            )
          )}
        </div>
      )}

      {activeView === 'create' && (
        <form onSubmit={handleCreateUser}>
          <div className="form-group">
            <label htmlFor="create-name">Nome</label>
            <input
              id="create-name"
              type="text"
              value={createName}
              onChange={(e) => setCreateName(e.target.value)}
              placeholder="Nome completo"
              required
            />
          </div>

          <div className="form-group">
            <label htmlFor="create-email">Email</label>
            <input
              id="create-email"
              type="email"
              value={createEmail}
              onChange={(e) => setCreateEmail(e.target.value)}
              placeholder="seu@email.com"
              required
            />
          </div>

          <button
            type="submit"
            className="primary"
            disabled={loading}
            style={{ opacity: loading ? 0.6 : 1 }}
          >
            {loading ? 'Criando...' : 'Criar Usuário'}
          </button>
        </form>
      )}

      {activeView === 'search' && (
        <form onSubmit={handleSearchUser}>
          <div className="input-group">
            <input
              type="text"
              value={searchId}
              onChange={(e) => setSearchId(e.target.value)}
              placeholder="Insira o ID do usuário"
              required
            />
            <button
              type="submit"
              className="primary"
              disabled={loading}
              style={{ opacity: loading ? 0.6 : 1 }}
            >
              {loading ? 'Buscando...' : 'Buscar'}
            </button>
          </div>

          {loading && (
            <div className="loading">
              <span className="spinner"></span> Buscando...
            </div>
          )}

          {!loading && users.length > 0 && (
            <div>
              {users.map((user) => (
                <div key={user.id} className="user-card">
                  <div className="user-info">
                    <h3>{user.name}</h3>
                    <p>Email: {user.email}</p>
                    <p style={{ fontSize: '12px', color: '#999' }}>ID: {user.id}</p>
                  </div>
                </div>
              ))}
            </div>
          )}
        </form>
      )}
    </div>
  );
}
