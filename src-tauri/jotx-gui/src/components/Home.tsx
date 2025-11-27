import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './Home.css';

interface GUISearchResult {
    title: string,
    content: string,
    score: number,
    source: string,
    timestamp: number,
}

export default function Home() {
    const [mode, setMode] = useState<'ask' | 'search'>('ask');
    const [query, setQuery] = useState('');
    const [messages, setMessages] = useState<Array<{ content: string; type: string }>>([]);
    const [results, setResults] = useState<GUISearchResult[] | null>(null);
    const [loading, setLoading] = useState(false);

    const handleSubmit = async () => {
        if (!query.trim()) return;

        const userMessage = { content: query, type: 'user' };
        setMessages(prev => [...prev, userMessage]);
        setQuery('');
        setLoading(true);
        setResults(null);

        try {
            const result = await invoke<GUISearchResult[]>(
                mode === 'ask' ? 'ask_command' : 'search_command',
                { query, directory: './' }
            );

            setResults(result);
        } catch (error) {
            setMessages(prev => [...prev, {
                content: `Error: ${error}`,
                type: 'error'
            }]);
        } finally {
            setLoading(false);
        }
    };

    const getIcon = (type: string) => {
        const icons: Record<string, string> = {
            clipboard: '📋',
            shell: '💻',
            file: '📄',
            note: '📝',
        };
        return icons[type] || '📄';
    };

    const onClickResult = (result: GUISearchResult) => {
        setMessages(prev => [...prev, {
            content: `Selected: ${result.content}`,
            type: 'assistant'
        }]);
        setResults(null);
    };

    return (
        <div className="chat-container">
            <div className="chat-messages">
                {messages.length === 0 && !results && (
                    <div className="welcome-message">
                        <h2>Welcome to Jotx</h2>
                        <p>Ask questions or search through your content</p>
                    </div>
                )}

                {messages.map((msg, i) => (
                    <div key={i} className={`message ${msg.type}`}>
                        {msg.content}
                    </div>
                ))}

                {loading && <div className="loading">Processing...</div>}

                {results && (
                    <div className="results-container">
                        <div className="results-title">
                            Found {results.length} result(s)
                        </div>
                        {results.map((result, i) => (
                            <div
                                key={i}
                                className="result-item"
                                onClick={() => onClickResult(result)}
                            >
                                <span className="result-icon">{getIcon(result.source)}</span>
                                <div className="result-content">{result.content}</div>
                            </div>
                        ))}
                    </div>
                )}
            </div>

            <div className="input-container">
                <div className="input-wrapper">
                    <select
                        value={mode}
                        onChange={(e) => setMode(e.target.value as 'ask' | 'search')}
                        className="mode-select"
                    >
                        <option value="ask">Ask</option>
                        <option value="search">Search</option>
                    </select>
                    <input
                        type="text"
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        onKeyPress={(e) => e.key === 'Enter' && handleSubmit()}
                        placeholder="Type your query here..."
                        className="query-input"
                        autoFocus
                    />
                    <button onClick={handleSubmit} className="send-button">
                        <span>→</span>
                    </button>
                </div>
            </div>
        </div>
    );
}