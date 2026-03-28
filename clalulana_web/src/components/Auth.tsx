import { useState } from 'react';
import * as api from '../api';

interface AuthProps {
  onLogin: (token: string) => void;
  token: string | null;
}

export default function Auth({ onLogin, token }: AuthProps) {
  const [activeForm, setActiveForm] = useState<'login' | 'register'>('login');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Login form
  const [loginEmail, setLoginEmail] = useState('');
  const [loginPassword, setLoginPassword] = useState('');

  // Register form
  const [registerName, setRegisterName] = useState('');
  const [registerEmail, setRegisterEmail] = useState('');
  const [registerPassword, setRegisterPassword] = useState('');

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      const result = await api.login({
        email: loginEmail,
        password: loginPassword,
      });
      setSuccess('Login realizado com sucesso!');
      onLogin(result.token);
      setLoginEmail('');
      setLoginPassword('');
    } catch (err: any) {
      setError(err.response?.data?.message || err.message || 'Erro ao fazer login');
    } finally {
      setLoading(false);
    }
  };

  const handleRegister = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      await api.register({
        email: registerEmail,
        password: registerPassword,
        name: registerName,
      });
      setSuccess('Registro realizado com sucesso! Faça login para continuar.');
      setRegisterName('');
      setRegisterEmail('');
      setRegisterPassword('');
      setActiveForm('login');
    } catch (err: any) {
      setError(err.response?.data?.message || err.message || 'Erro ao registrar');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <h2 style={{ marginBottom: '20px' }}>🔐 Autenticação</h2>

      {token && (
        <div className="success-message">
          ✓ Você está autenticado! Pode usar os endpoints protegidos.
        </div>
      )}

      {error && <div className="error-message">{error}</div>}
      {success && <div className="success-message">{success}</div>}

      <div className="tabs" style={{ marginBottom: '20px' }}>
        <button
          className={`tab-button ${activeForm === 'login' ? 'active' : ''}`}
          onClick={() => setActiveForm('login')}
        >
          Login
        </button>
        <button
          className={`tab-button ${activeForm === 'register' ? 'active' : ''}`}
          onClick={() => setActiveForm('register')}
        >
          Registrar
        </button>
      </div>

      {activeForm === 'login' ? (
        <form onSubmit={handleLogin}>
          <div className="form-group">
            <label htmlFor="login-email">Email</label>
            <input
              id="login-email"
              type="email"
              value={loginEmail}
              onChange={(e) => setLoginEmail(e.target.value)}
              placeholder="seu@email.com"
              required
            />
          </div>

          <div className="form-group">
            <label htmlFor="login-password">Senha</label>
            <input
              id="login-password"
              type="password"
              value={loginPassword}
              onChange={(e) => setLoginPassword(e.target.value)}
              placeholder="Sua senha"
              required
            />
          </div>

          <button
            type="submit"
            className="primary"
            disabled={loading}
            style={{ opacity: loading ? 0.6 : 1 }}
          >
            {loading ? 'Entrando...' : 'Entrar'}
          </button>
        </form>
      ) : (
        <form onSubmit={handleRegister}>
          <div className="form-group">
            <label htmlFor="register-name">Nome</label>
            <input
              id="register-name"
              type="text"
              value={registerName}
              onChange={(e) => setRegisterName(e.target.value)}
              placeholder="Seu nome"
              required
            />
          </div>

          <div className="form-group">
            <label htmlFor="register-email">Email</label>
            <input
              id="register-email"
              type="email"
              value={registerEmail}
              onChange={(e) => setRegisterEmail(e.target.value)}
              placeholder="seu@email.com"
              required
            />
          </div>

          <div className="form-group">
            <label htmlFor="register-password">Senha</label>
            <input
              id="register-password"
              type="password"
              value={registerPassword}
              onChange={(e) => setRegisterPassword(e.target.value)}
              placeholder="Crie uma senha"
              required
            />
          </div>

          <button
            type="submit"
            className="primary"
            disabled={loading}
            style={{ opacity: loading ? 0.6 : 1 }}
          >
            {loading ? 'Registrando...' : 'Registrar'}
          </button>
        </form>
      )}

      <div style={{ marginTop: '30px', padding: '15px', background: '#f0f0f0', borderRadius: '6px' }}>
        <h3 style={{ marginBottom: '10px', color: '#333' }}>📝 Credenciais de Teste</h3>
        <p style={{ fontSize: '13px', color: '#666', marginBottom: '5px' }}>
          <strong>Email:</strong> teste@email.com
        </p>
        <p style={{ fontSize: '13px', color: '#666' }}>
          <strong>Senha:</strong> senha123
        </p>
      </div>
    </div>
  );
}
