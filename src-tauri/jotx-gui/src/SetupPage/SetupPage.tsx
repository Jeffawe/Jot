import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './SetupPage.css';

interface SetupProps {
  onComplete: () => void;
}

function Setup({ onComplete }: SetupProps) {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');
  const [status, setStatus] = useState('');
  const [showSkipWarning, setShowSkipWarning] = useState(false);

  const startSetup = async () => {
    setIsLoading(true);
    setError('');

    try {
      setStatus('Setting up shell hooks...');
      await new Promise(resolve => setTimeout(resolve, 500));

      setStatus('Installing Ollama...');
      await invoke('run_setup');

      setStatus('Setup complete! ✓');

      // Wait a moment then complete
      await new Promise(resolve => setTimeout(resolve, 1500));
      onComplete();

    } catch (err) {
      setError(`Setup failed: ${err}`);
      setIsLoading(false);
    }
  };

  const handleSkipClick = () => {
    setShowSkipWarning(true);
  };

  const confirmSkip = () => {
    setShowSkipWarning(false);
    onComplete();
  };

  const cancelSkip = () => {
    setShowSkipWarning(false);
  };

  return (
    <div className="setup-container">
      <div className="setup-card">
        <h1>Welcome to Jotx! 🚀</h1>
        <p className="subtitle">First-time setup required</p>

        <div className="requirements">
          <h3>What will be installed:</h3>
          <ul>
            <li>Ollama (AI engine) - ~300MB download</li>
            <li>Shell hooks for system monitoring</li>
            <li>Default AI model (qwen2.5:1.5b)</li>
          </ul>
        </div>

        {!isLoading ? (
          <div className="buttons">
            <button className="btn-secondary" onClick={handleSkipClick}>
              Skip for Now
            </button>
            <button className="btn-primary" onClick={startSetup}>
              Start Setup
            </button>
          </div>
        ) : (
          <div className="progress">
            <div className="spinner"></div>
            <p className="status-text">{status}</p>
          </div>
        )}

        {error && (
          <div className="error">
            {error}
          </div>
        )}
      </div>

      {/* Skip Warning Modal */}
      {showSkipWarning && (
        <div className="modal-overlay">
          <div className="modal">
            <div className="modal-icon">⚠️</div>
            <h2>Skip Setup?</h2>
            <p className="modal-text">
              Without setup, Jotx will have <strong>limited functionality</strong>:
            </p>
            <ul className="warning-list">
              <li>❌ No AI-powered search</li>
              <li>❌ No clipboard/shell monitoring</li>
              <li>❌ No natural language queries</li>
            </ul>
            <p className="modal-note">
              You can set up these features later:
            </p>
            <ul className="info-list">
              <li>🤖 LLM setup in the <strong>LLM tab</strong></li>
              <li>⚙️ Shell hooks in the <strong>Settings tab</strong></li>
            </ul>
            <div className="modal-buttons">
              <button className="btn-secondary" onClick={cancelSkip}>
                Go Back
              </button>
              <button className="btn-warning" onClick={confirmSkip}>
                Skip Anyway
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default Setup;