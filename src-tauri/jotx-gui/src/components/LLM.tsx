import { useState, useEffect } from 'react';
import './LLM.css';
import { invoke } from '@tauri-apps/api/core';

interface Model {
  name: string;
  size: string;
  description: string;
}

interface OllamaStatus {
  installed: boolean;
  running: boolean;
  models: string[];
}

const POPULAR_MODELS: Model[] = [
  { name: 'smollm:135m', size: '~80MB', description: 'Tiny, ultra-fast' },
  { name: 'smollm:360m', size: '~200MB', description: 'Very small' },
  { name: 'qwen2:0.5b', size: '~350MB', description: 'Fast, good for structured output' },
  { name: 'tinyllama:1.1b', size: '~600MB', description: 'Balanced speed/quality' },
  { name: 'smollm:1.7b', size: '~1GB', description: 'SmolLM largest' },
  { name: 'qwen2.5:1.5b', size: '~900MB', description: 'Better reasoning (Recommended)' },
  { name: 'llama3.2:1b', size: '~1.3GB', description: "Meta's 1B model" },
  { name: 'phi3:3.8b', size: '~2.3GB', description: 'Microsoft, punches above weight' },
  { name: 'qwen2.5:3b', size: '~2GB', description: 'Recommended for NLP tasks' },
  { name: 'llama3.2:3b', size: '~2GB', description: "Meta's 3B model" },
];

export default function LLM() {
  const [status, setStatus] = useState<OllamaStatus>({
    installed: false,
    running: false,
    models: [],
  });
  const [loading, setLoading] = useState(true);
  const [activeAction, setActiveAction] = useState<string | null>(null);
  const [selectedModel, setSelectedModel] = useState('');
  const [actionInProgress, setActionInProgress] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error' | 'info'; text: string } | null>(null);

  useEffect(() => {
    checkStatus();
  }, []);

  const checkStatus = async () => {
    setLoading(true);
    try {
      const result = await invoke<OllamaStatus>('check_ollama_status');
    
      setStatus(result);
    } catch (error) {
      console.error('Failed to check status:', error);
      setStatus({ installed: false, running: false, models: [] });
    } finally {
      setLoading(false);
    }
  };

  const handleInstall = async () => {
    setActionInProgress(true);
    setMessage({ type: 'info', text: 'Installing Ollama...' });
    
    try {
      await invoke('install_ollama');
      
      setMessage({ type: 'success', text: '✓ Installation complete!' });
      await checkStatus();
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    } catch (error) {
      setMessage({ type: 'error', text: '✗ Installation failed' });
    } finally {
      setActionInProgress(false);
      setTimeout(() => setMessage(null), 3000);
    }
  };

  const handleDownload = async () => {
    if (!selectedModel) {
      setMessage({ type: 'error', text: 'Please select a model' });
      setTimeout(() => setMessage(null), 3000);
      return;
    }

    setActionInProgress(true);
    setMessage({ type: 'info', text: `Downloading ${selectedModel}...` });
    
    try {
      await invoke('download_model', { model: selectedModel });
      
      setMessage({ type: 'success', text: `✓ ${selectedModel} downloaded!` });
      setSelectedModel('');
      await checkStatus();
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    } catch (error) {
      setMessage({ type: 'error', text: '✗ Download failed' });
    } finally {
      setActionInProgress(false);
      setTimeout(() => setMessage(null), 3000);
    }
  };

  const handleRemove = async (model: string) => {
    if (!window.confirm(`Remove ${model}?`)) return;

    setActionInProgress(true);
    setMessage({ type: 'info', text: `Removing ${model}...` });
    
    try {
      await invoke('remove_model', { model });
      
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      setMessage({ type: 'success', text: `✓ ${model} removed!` });
      await checkStatus();
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    } catch (error) {
      setMessage({ type: 'error', text: '✗ Removal failed' });
    } finally {
      setActionInProgress(false);
      setTimeout(() => setMessage(null), 3000);
    }
  };

  const handleStartService = async () => {
    setActionInProgress(true);
    setMessage({ type: 'info', text: 'Starting Ollama service...' });
    
    try {
      await invoke('start_ollama');
      
      setMessage({ type: 'success', text: '✓ Ollama service started' });
      await checkStatus();
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    } catch (error) {
      setMessage({ type: 'error', text: '✗ Failed to start service' });
    } finally {
      setActionInProgress(false);
      setTimeout(() => setMessage(null), 3000);
    }
  };

  if (loading) {
    return (
      <div className="tab-content">
        <div className="content-header">
          <h2>LLM Management</h2>
          <p>Large Language Model configuration and settings</p>
        </div>
        <div className="llm-content">
          <div className="loading">Checking Ollama status...</div>
        </div>
      </div>
    );
  }

  return (
    <div className="tab-content">
      <div className="content-header">
        <h2>LLM Management</h2>
        <p>Configure and manage your local language models with Ollama</p>
      </div>

      <div className="llm-content">
        {message && (
          <div className={`message message-${message.type}`}>
            {message.text}
          </div>
        )}

        {/* Status Section */}
        <section className="llm-section">
          <h3>Current Status</h3>
          <div className="status-grid">
            <div className="status-item">
              <span className={`status-indicator ${status.installed ? 'success' : 'error'}`}>
                {status.installed ? '✓' : '✗'}
              </span>
              <span>Ollama {status.installed ? 'installed' : 'not installed'}</span>
            </div>
            {status.installed && (
              <div className="status-item">
                <span className={`status-indicator ${status.running ? 'success' : 'error'}`}>
                  {status.running ? '✓' : '✗'}
                </span>
                <span>Service {status.running ? 'running' : 'not running'}</span>
              </div>
            )}
          </div>

          {!status.installed && (
            <button 
              className="action-button primary"
              onClick={handleInstall}
              disabled={actionInProgress}
            >
              Install Ollama
            </button>
          )}

          {status.installed && !status.running && (
            <button 
              className="action-button primary"
              onClick={handleStartService}
              disabled={actionInProgress}
            >
              Start Ollama Service
            </button>
          )}
        </section>

        {/* Installed Models */}
        {status.installed && status.models.length > 0 && (
          <section className="llm-section">
            <h3>Installed Models</h3>
            <div className="models-list">
              {status.models.map((model) => (
                <div key={model} className="model-item installed">
                  <div className="model-info">
                    <span className="model-name">{model}</span>
                  </div>
                  <button
                    className="action-button small danger"
                    onClick={() => handleRemove(model)}
                    disabled={actionInProgress}
                  >
                    Remove
                  </button>
                </div>
              ))}
            </div>
          </section>
        )}

        {/* Available Models */}
        {status.installed && (
          <section className="llm-section">
            <div className="section-header">
              <h3>Available Models</h3>
              <button
                className="action-button small"
                onClick={() => setActiveAction(activeAction === 'browse' ? null : 'browse')}
              >
                {activeAction === 'browse' ? 'Hide' : 'Browse Models'}
              </button>
            </div>

            {activeAction === 'browse' && (
              <div className="models-browse">
                <p className="info-text">
                  Recommended: <strong>qwen2.5:1.5b</strong> or <strong>qwen2.5:3b</strong> for best balance of speed and capability.
                </p>
                <div className="models-grid">
                  {POPULAR_MODELS.map((model) => {
                    const isInstalled = status.models.includes(model.name);
                    return (
                      <div
                        key={model.name}
                        className={`model-card ${isInstalled ? 'installed' : ''} ${selectedModel === model.name ? 'selected' : ''}`}
                        onClick={() => !isInstalled && setSelectedModel(model.name)}
                      >
                        <div className="model-card-header">
                          <span className="model-name">{model.name}</span>
                          <span className="model-size">{model.size}</span>
                        </div>
                        <p className="model-description">{model.description}</p>
                        {isInstalled && <span className="installed-badge">✓ Installed</span>}
                      </div>
                    );
                  })}
                </div>

                {selectedModel && (
                  <div className="download-section">
                    <p>Selected: <strong>{selectedModel}</strong></p>
                    <button
                      className="action-button primary"
                      onClick={handleDownload}
                      disabled={actionInProgress}
                    >
                      {actionInProgress ? 'Downloading...' : 'Download Model'}
                    </button>
                  </div>
                )}

                <div className="external-link">
                  <p>Visit <a href="https://ollama.com/models" target="_blank" rel="noopener noreferrer">ollama.com/models</a> for more models</p>
                </div>
              </div>
            )}
          </section>
        )}
      </div>
    </div>
  );
}