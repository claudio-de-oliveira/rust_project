import { useState } from 'react';
import Health from './components/Health';
import Auth from './components/Auth';
import Users from './components/Users';

type Tab = 'health' | 'auth' | 'users';

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('health');
  const [token, setToken] = useState<string | null>(localStorage.getItem('token'));

  const handleLogin = (newToken: string) => {
    setToken(newToken);
    localStorage.setItem('token', newToken);
  };

  const handleLogout = () => {
    setToken(null);
    localStorage.removeItem('token');
    setActiveTab('health');
  };

  return (
    <div className="container">
      <div className="header">
        <h1>🚀 Clalulana API Testador</h1>
        <p>Interface para testar sua API REST com CQRS</p>
      </div>

      <div className="tabs">
        <button
          className={`tab-button ${activeTab === 'health' ? 'active' : ''}`}
          onClick={() => setActiveTab('health')}
        >
          ✓ Health Check
        </button>
        <button
          className={`tab-button ${activeTab === 'auth' ? 'active' : ''}`}
          onClick={() => setActiveTab('auth')}
        >
          🔐 Autenticação
        </button>
        {token && (
          <>
            <button
              className={`tab-button ${activeTab === 'users' ? 'active' : ''}`}
              onClick={() => setActiveTab('users')}
            >
              👥 Usuários
            </button>
            <button
              className="tab-button"
              onClick={handleLogout}
              style={{ background: '#ef4444', color: 'white' }}
            >
              Sair
            </button>
          </>
        )}
      </div>

      <div className="content">
        {activeTab === 'health' && <Health />}
        {activeTab === 'auth' && <Auth onLogin={handleLogin} token={token} />}
        {activeTab === 'users' && token && <Users token={token} />}
        {!token && activeTab === 'users' && (
          <div className="info-message">
            Você precisa estar autenticado para acessar os usuários. Vá para a aba Autenticação.
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
