import { useState } from 'react';
import Home from './components/Home';
import Paths from './components/Paths';
import LLM from './components/LLM';
import Settings from './components/Settings';
import './App.css';

type Tab = 'home' | 'paths' | 'llm' | 'settings';

function App() {
  const [activeTab, setActiveTab] = useState<Tab>('home');
  const [theme, setTheme] = useState<'light' | 'dark'>(() => {
    return (localStorage.getItem('theme') as 'light' | 'dark') || 'light';
  });

  const toggleTheme = () => {
    const newTheme = theme === 'light' ? 'dark' : 'light';
    setTheme(newTheme);
    localStorage.setItem('theme', newTheme);
  };

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
        {activeTab === 'settings' && <Settings />}
      </main>
    </div>
  );
}

export default App;