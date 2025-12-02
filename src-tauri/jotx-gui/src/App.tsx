import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import Home from './components/Home';
import Paths from './components/Paths';
import LLM from './components/LLM';
import Settings from './components/Settings';
import SetupPage from './SetupPage/SetupPage';
import './App.css';

type Tab = 'setup' | 'home' | 'paths' | 'llm' | 'settings';

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('home');
  const [setupComplete, setSetupComplete] = useState<boolean>(true); // Default true until checked
  const [theme, setTheme] = useState<'light' | 'dark'>(() => {
    return (localStorage.getItem('theme') as 'light' | 'dark') || 'light';
  });

  // Check setup status on mount
  useEffect(() => {
    const checkSetup = async () => {
      try {
        const isComplete = await invoke<boolean>('check_setup_status');
        setSetupComplete(isComplete);
        
        // If setup not complete, show setup tab
        if (!isComplete) {
          setActiveTab('setup');
        }
      } catch (error) {
        console.error('Failed to check setup status:', error);
      }
    };
    
    checkSetup();
  }, []);

  const toggleTheme = () => {
    const newTheme = theme === 'light' ? 'dark' : 'light';
    setTheme(newTheme);
    localStorage.setItem('theme', newTheme);
  };

  const onSetupComplete = () => {
    setSetupComplete(true);
    setActiveTab('home');
  };

  // If setup not complete, show only setup screen
  if (!setupComplete) {
    return (
      <div className={`app-container ${theme}-theme`}>
        <SetupPage onComplete={onSetupComplete} />
      </div>
    );
  }

  return (
    <div className={`app-container ${theme}-theme`}>
      <header className="app-header">
        <div className="header-left">
          <h1 className="app-title">Jotx</h1>
        </div>
        <nav className="nav-tabs">
          <button
            className={`nav-tab ${activeTab === 'home' ? 'active' : ''}`}
            onClick={() => setActiveTab('home')}
          >
            <span className="tab-icon">🏠</span>
            <span>Home</span>
          </button>
          <button
            className={`nav-tab ${activeTab === 'llm' ? 'active' : ''}`}
            onClick={() => setActiveTab('llm')}
          >
            <span className="tab-icon">🤖</span>
            <span>LLM</span>
          </button>
          <button
            className={`nav-tab ${activeTab === 'settings' ? 'active' : ''}`}
            onClick={() => setActiveTab('settings')}
          >
            <span className="tab-icon">⚙️</span>
            <span>Settings</span>
          </button>
          <button
            className={`nav-tab ${activeTab === 'paths' ? 'active' : ''}`}
            onClick={() => setActiveTab('paths')}
          >
            <span className="tab-icon">📂</span>
            <span>Paths</span>
          </button>
        </nav>
        <div className="header-right">
          <button className="theme-toggle" onClick={toggleTheme}>
            <span className="theme-icon">{theme === 'light' ? '🌙' : '☀️'}</span>
          </button>
        </div>
      </header>
      <main className="main-content">
        {activeTab === 'home' && <Home />}
        {activeTab === 'paths' && <Paths />}
        {activeTab === 'llm' && <LLM />}
        {activeTab === 'settings' && <Settings check_setup_status={setupComplete} />}
      </main>
    </div>
  );
}

export default App;