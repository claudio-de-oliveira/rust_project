import { useState } from 'react';
import * as api from '../api';

interface HealthResponse {
  status: string;
  timestamp?: string;
}

export default function Health() {
  const [loading, setLoading] = useState(false);
  const [response, setResponse] = useState<HealthResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleHealthCheck = async () => {
    setLoading(true);
    setError(null);
    setResponse(null);

    try {
      const result = await api.healthCheck();
      setResponse(result);
    } catch (err: any) {
      setError(err.response?.data?.message || err.message || 'Erro ao conectar à API');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <h2 style={{ marginBottom: '20px' }}>✓ Health Check da API</h2>

      <div className="form-group">
        <p style={{ marginBottom: '15px', color: '#666' }}>
          Verifique se a API está respondendo corretamente.
        </p>
      </div>

      <button
        className="primary"
        onClick={handleHealthCheck}
        disabled={loading}
        style={{ opacity: loading ? 0.6 : 1 }}
      >
        {loading ? <span className="spinner"></span> : '🔍 Fazer Health Check'}
      </button>

      {response && (
        <div className="response-box success">
          <div className="status-badge success">Status: {response.status}</div>
          <pre>{JSON.stringify(response, null, 2)}</pre>
        </div>
      )}

      {error && (
        <div className="response-box error">
          <div className="status-badge error">Erro</div>
          <pre>{error}</pre>
        </div>
      )}
    </div>
  );
}
