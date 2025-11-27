import { useEffect, useState } from 'react';
import './Paths.css';
import { invoke } from '@tauri-apps/api/core';

interface PathInfo {
    label: string;
    path: string;
}

export default function Paths() {
    const [paths, setPaths] = useState<PathInfo[]>();
    const [copiedIndex, setCopiedIndex] = useState<number | null>(null);

    useEffect(() => {
        invoke<PathInfo[]>('get_all_paths')
            .then(setPaths)
            .catch(console.error);
    }, []);

    const copyPath = async (path: string, index: number) => {
        try {
            await navigator.clipboard.writeText(path);
            setCopiedIndex(index);
            setTimeout(() => setCopiedIndex(null), 2000);
        } catch (error) {
            console.error('Failed to copy:', error);
        }
    };

    return (
        <div className="tab-content">
            <div className="content-header">
                <h2>Paths</h2>
                <p>Quick access to your important directories</p>
            </div>
            <div className="paths-list">
                {paths?.map((path, i) => (
                    <div key={i} className="path-item">
                        <div className="path-info">
                            <div className="path-label">{path.label}</div>
                            <div className="path-value">{path.path}</div>
                        </div>
                        <button
                            className={`copy-button ${copiedIndex === i ? 'copied' : ''}`}
                            onClick={() => copyPath(path.path, i)}
                        >
                            {copiedIndex === i ? '✓ Copied' : 'Copy'}
                        </button>
                    </div>
                ))}
            </div>
        </div>
    );
}