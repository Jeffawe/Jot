import { useState, useEffect, useCallback } from 'react';
import './Settings.css';
import PrivacySettings from './PrivacySettings';
import { invoke } from '@tauri-apps/api/core';

interface AppSettings {
    capture_clipboard: boolean;
    capture_shell: boolean;
    capture_shell_history_with_files: boolean;
    clipboard_case_sensitive: boolean;
    shell_case_sensitive: boolean;
    clipboard_limit: number;
    shell_limit: number;
}

export default function Settings() {
    const [settings, setSettings] = useState<AppSettings>({
        capture_clipboard: true,
        capture_shell: true,
        capture_shell_history_with_files: true,
        clipboard_case_sensitive: false,
        shell_case_sensitive: false,
        clipboard_limit: 1000,
        shell_limit: 1000,
    });

    const [activeTab, setActiveTab] = useState<'general' | 'privacy'>('general');
    const [editingLimit, setEditingLimit] = useState<'clipboard' | 'shell' | null>(null);
    const [tempLimit, setTempLimit] = useState('');
    const [saveMessage, setSaveMessage] = useState<string | null>(null);

    const loadSettings = useCallback(async () => {
        try {
            const result = await invoke<AppSettings>('get_settings');

            setSettings(result);
        } catch (error) {
            console.error('Failed to load settings:', error);
        }
    }, []);

    useEffect(() => {
        let mounted = true;
        (async () => {
            await Promise.resolve();
            if (!mounted) return;
            await loadSettings();
        })();
        return () => { mounted = false; };
    }, [loadSettings]);

    const saveSettings = async (updatedSettings: AppSettings) => {
        try {
            await invoke('save_settings', { settings: updatedSettings });

            setSettings(updatedSettings);
            setSaveMessage('Settings saved!');
            setTimeout(() => setSaveMessage(null), 2000);
        } catch (error) {
            console.error('Failed to save settings:', error);
            setSaveMessage('Failed to save settings');
            setTimeout(() => setSaveMessage(null), 2000);
        }
    };

    const toggleSetting = (key: keyof AppSettings) => {
        const updated = { ...settings, [key]: !settings[key] };
        saveSettings(updated);
    };

    const startEditLimit = (type: 'clipboard' | 'shell') => {
        setEditingLimit(type);
        setTempLimit(type === 'clipboard' ? settings.clipboard_limit.toString() : settings.shell_limit.toString());
    };

    const saveLimit = () => {
        if (!editingLimit) return;

        const value = parseInt(tempLimit);
        if (isNaN(value) || value < 1) {
            alert('Please enter a valid number greater than 0');
            return;
        }

        const key = editingLimit === 'clipboard' ? 'clipboard_limit' : 'shell_limit';
        const updated = { ...settings, [key]: value };
        saveSettings(updated);
        setEditingLimit(null);
        setTempLimit('');
    };

    const cancelEdit = () => {
        setEditingLimit(null);
        setTempLimit('');
    };

    return (
        <div className="tab-content">
            <div className="content-header">
                <h2>Settings</h2>
                <p>Configure JotX behavior and privacy options</p>
            </div>

            <div className="settings-content">
                {saveMessage && (
                    <div className="save-message">
                        {saveMessage}
                    </div>
                )}

                {/* Tab Navigation */}
                <div className="settings-tabs">
                    <button
                        className={`settings-tab ${activeTab === 'general' ? 'active' : ''}`}
                        onClick={() => setActiveTab('general')}
                    >
                        General Settings
                    </button>
                    <button
                        className={`settings-tab ${activeTab === 'privacy' ? 'active' : ''}`}
                        onClick={() => setActiveTab('privacy')}
                    >
                        Privacy Settings
                    </button>
                </div>

                {/* General Settings Tab */}
                {activeTab === 'general' && (
                    <div className="settings-section">
                        <h3>Capture Options</h3>

                        <div className="setting-item">
                            <div className="setting-info">
                                <label>Capture Clipboard</label>
                                <p className="setting-description">Monitor and save clipboard changes</p>
                            </div>
                            <button
                                className={`toggle-button ${settings.capture_clipboard ? 'active' : ''}`}
                                onClick={() => toggleSetting('capture_clipboard')}
                            >
                                <span className="toggle-slider"></span>
                                <span className="toggle-label">
                                    {settings.capture_clipboard ? '✓ ON' : '✗ OFF'}
                                </span>
                            </button>
                        </div>

                        <div className="setting-item">
                            <div className="setting-info">
                                <label>Capture Shell</label>
                                <p className="setting-description">Monitor and save shell command history</p>
                            </div>
                            <button
                                className={`toggle-button ${settings.capture_shell ? 'active' : ''}`}
                                onClick={() => toggleSetting('capture_shell')}
                            >
                                <span className="toggle-slider"></span>
                                <span className="toggle-label">
                                    {settings.capture_shell ? '✓ ON' : '✗ OFF'}
                                </span>
                            </button>
                        </div>

                        <div className="setting-item">
                            <div className="setting-info">
                                <label>Use Shell History With Files</label>
                                <p className="setting-description">Include file context when capturing shell commands</p>
                            </div>
                            <button
                                className={`toggle-button ${settings.capture_shell_history_with_files ? 'active' : ''}`}
                                onClick={() => toggleSetting('capture_shell_history_with_files')}
                            >
                                <span className="toggle-slider"></span>
                                <span className="toggle-label">
                                    {settings.capture_shell_history_with_files ? '✓ ON' : '✗ OFF'}
                                </span>
                            </button>
                        </div>

                        <div className="settings-divider"></div>

                        <h3>Search Options</h3>

                        <div className="setting-item">
                            <div className="setting-info">
                                <label>Clipboard Case Sensitive</label>
                                <p className="setting-description">Make clipboard searches case-sensitive</p>
                            </div>
                            <button
                                className={`toggle-button ${settings.clipboard_case_sensitive ? 'active' : ''}`}
                                onClick={() => toggleSetting('clipboard_case_sensitive')}
                            >
                                <span className="toggle-slider"></span>
                                <span className="toggle-label">
                                    {settings.clipboard_case_sensitive ? '✓ ON' : '✗ OFF'}
                                </span>
                            </button>
                        </div>

                        <div className="setting-item">
                            <div className="setting-info">
                                <label>Shell Case Sensitive</label>
                                <p className="setting-description">Make shell searches case-sensitive</p>
                            </div>
                            <button
                                className={`toggle-button ${settings.shell_case_sensitive ? 'active' : ''}`}
                                onClick={() => toggleSetting('shell_case_sensitive')}
                            >
                                <span className="toggle-slider"></span>
                                <span className="toggle-label">
                                    {settings.shell_case_sensitive ? '✓ ON' : '✗ OFF'}
                                </span>
                            </button>
                        </div>

                        <div className="settings-divider"></div>

                        <h3>History Limits</h3>

                        <div className="setting-item">
                            <div className="setting-info">
                                <label>Clipboard History Size</label>
                                <p className="setting-description">Maximum number of clipboard entries to store</p>
                            </div>
                            {editingLimit === 'clipboard' ? (
                                <div className="limit-editor">
                                    <input
                                        type="number"
                                        value={tempLimit}
                                        onChange={(e) => setTempLimit(e.target.value)}
                                        className="limit-input"
                                        min="1"
                                        autoFocus
                                    />
                                    <button onClick={saveLimit} className="limit-button save">✓</button>
                                    <button onClick={cancelEdit} className="limit-button cancel">✗</button>
                                </div>
                            ) : (
                                <div className="limit-display" onClick={() => startEditLimit('clipboard')}>
                                    <span className="limit-value">{settings.clipboard_limit}</span>
                                    <span className="edit-hint">Click to edit</span>
                                </div>
                            )}
                        </div>

                        <div className="setting-item">
                            <div className="setting-info">
                                <label>Shell History Size</label>
                                <p className="setting-description">Maximum number of shell commands to store</p>
                            </div>
                            {editingLimit === 'shell' ? (
                                <div className="limit-editor">
                                    <input
                                        type="number"
                                        value={tempLimit}
                                        onChange={(e) => setTempLimit(e.target.value)}
                                        className="limit-input"
                                        min="1"
                                        autoFocus
                                    />
                                    <button onClick={saveLimit} className="limit-button save">✓</button>
                                    <button onClick={cancelEdit} className="limit-button cancel">✗</button>
                                </div>
                            ) : (
                                <div className="limit-display" onClick={() => startEditLimit('shell')}>
                                    <span className="limit-value">{settings.shell_limit}</span>
                                    <span className="edit-hint">Click to edit</span>
                                </div>
                            )}
                        </div>
                    </div>
                )}

                {/* Privacy Settings Tab */}
                {activeTab === 'privacy' && <PrivacySettings />}
            </div>
        </div>
    );
}