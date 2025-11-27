import { invoke } from '@tauri-apps/api/core';
import { useState, useEffect, useCallback } from 'react';

interface PrivacyConfig {
    excludes_contains_string: string[];
    excludes_starts_with_string: string[];
    excludes_ends_with_string: string[];
    excludes_regex: string[];
    exclude_folders: string[];
}

type ExclusionType = keyof PrivacyConfig;

export default function PrivacySettings() {
    const [privacy, setPrivacy] = useState<PrivacyConfig>({
        excludes_contains_string: [],
        excludes_starts_with_string: [],
        excludes_ends_with_string: [],
        excludes_regex: [],
        exclude_folders: [],
    });

    const [editingType, setEditingType] = useState<ExclusionType | null>(null);
    const [newItem, setNewItem] = useState('');
    const [saveMessage, setSaveMessage] = useState<string | null>(null);


    const loadPrivacySettings = useCallback(async () => {
        try {
            const result = await invoke<PrivacyConfig>('get_privacy_config');

            setPrivacy(result);
        } catch (error) {
            console.error('Failed to load privacy settings:', error);
        }
    }, []);

    useEffect(() => {
        let mounted = true;
        (async () => {
            await Promise.resolve();
            if (!mounted) return;
            await loadPrivacySettings();
        })();
        return () => { mounted = false; };
    }, [loadPrivacySettings]);

    const savePrivacySettings = async (updatedPrivacy: PrivacyConfig) => {
        try {
            await invoke('save_privacy_config', { privacy: updatedPrivacy });

            setPrivacy(updatedPrivacy);
            setSaveMessage('Privacy settings saved!');
            setTimeout(() => setSaveMessage(null), 2000);
        } catch (error) {
            console.error('Failed to save privacy settings:', error);
            setSaveMessage('Failed to save privacy settings');
            setTimeout(() => setSaveMessage(null), 2000);
        }
    };

    const addItem = (type: ExclusionType) => {
        if (!newItem.trim()) return;

        const updated = {
            ...privacy,
            [type]: [...privacy[type], newItem.trim()],
        };

        savePrivacySettings(updated);
        setNewItem('');
    };

    const removeItem = (type: ExclusionType, index: number) => {
        const updated = {
            ...privacy,
            [type]: privacy[type].filter((_, i) => i !== index),
        };

        savePrivacySettings(updated);
    };

    const exclusionCategories: {
        key: ExclusionType;
        title: string;
        description: string;
        placeholder: string;
    }[] = [
            {
                key: 'excludes_contains_string',
                title: 'Contains String Exclusions',
                description: 'Exclude entries containing these strings',
                placeholder: 'e.g., password, secret, token',
            },
            {
                key: 'excludes_starts_with_string',
                title: 'Starts With Exclusions',
                description: 'Exclude entries starting with these strings',
                placeholder: 'e.g., sudo, ssh, rm -rf',
            },
            {
                key: 'excludes_ends_with_string',
                title: 'Ends With Exclusions',
                description: 'Exclude entries ending with these strings',
                placeholder: 'e.g., .key, .pem, .env',
            },
            {
                key: 'excludes_regex',
                title: 'Regex Exclusions',
                description: 'Exclude entries matching these regex patterns',
                placeholder: 'e.g., .*api[_-]key.*',
            },
            {
                key: 'exclude_folders',
                title: 'Folder Exclusions',
                description: 'Exclude all files in these folders',
                placeholder: 'e.g., /etc/secrets, ~/.ssh',
            },
        ];

    return (
        <div className="privacy-settings">
            {saveMessage && (
                <div className="save-message">
                    {saveMessage}
                </div>
            )}

            <div className="privacy-intro">
                <p>
                    Configure privacy rules to prevent sensitive information from being captured.
                    These exclusions apply to both clipboard and shell history.
                </p>
            </div>

            {exclusionCategories.map((category) => (
                <div key={category.key} className="exclusion-category">
                    <div className="category-header">
                        <div>
                            <h3>{category.title}</h3>
                            <p className="category-description">{category.description}</p>
                        </div>
                        <button
                            className="edit-button"
                            onClick={() => setEditingType(editingType === category.key ? null : category.key)}
                        >
                            {editingType === category.key ? 'Done' : 'Edit'}
                        </button>
                    </div>

                    <div className="category-count">
                        {privacy[category.key].length} {privacy[category.key].length === 1 ? 'rule' : 'rules'}
                    </div>

                    {editingType === category.key && (
                        <div className="edit-section">
                            {/* Add new item */}
                            <div className="add-item-form">
                                <input
                                    type="text"
                                    value={newItem}
                                    onChange={(e) => setNewItem(e.target.value)}
                                    placeholder={category.placeholder}
                                    className="add-input"
                                    onKeyPress={(e) => {
                                        if (e.key === 'Enter') {
                                            addItem(category.key);
                                        }
                                    }}
                                />
                                <button
                                    onClick={() => addItem(category.key)}
                                    className="add-button"
                                    disabled={!newItem.trim()}
                                >
                                    Add
                                </button>
                            </div>

                            {/* List of items */}
                            {privacy[category.key].length === 0 ? (
                                <div className="empty-state">No exclusions added yet</div>
                            ) : (
                                <div className="exclusion-list">
                                    {privacy[category.key].map((item, index) => (
                                        <div key={index} className="exclusion-item">
                                            <span className="item-number">{index + 1}.</span>
                                            <span className="item-text">{item}</span>
                                            <button
                                                onClick={() => removeItem(category.key, index)}
                                                className="remove-button"
                                                title="Remove"
                                            >
                                                ✗
                                            </button>
                                        </div>
                                    ))}
                                </div>
                            )}
                        </div>
                    )}
                </div>
            ))}
        </div>
    );
}