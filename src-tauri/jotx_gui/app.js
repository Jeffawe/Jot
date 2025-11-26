const { invoke } = window.__TAURI__.tauri;

// State
let currentMode = 'ask';
let currentDirectory = './'; // Default directory, you can make this configurable

// DOM Elements
const themeToggle = document.getElementById('theme-toggle');
const navTabs = document.querySelectorAll('.nav-tab');
const tabContents = document.querySelectorAll('.tab-content');
const queryInput = document.getElementById('query-input');
const sendButton = document.getElementById('send-button');
const modeSelect = document.getElementById('mode-select');
const chatMessages = document.getElementById('chat-messages');
const pathsList = document.getElementById('paths-list');

// Initialize
document.addEventListener('DOMContentLoaded', () => {
    initTheme();
    initTabs();
    initInput();
    loadPaths();
});

// Theme Toggle
function initTheme() {
    const savedTheme = localStorage.getItem('theme') || 'light';
    document.body.className = `${savedTheme}-theme`;
    updateThemeIcon(savedTheme);

    themeToggle.addEventListener('click', () => {
        const currentTheme = document.body.classList.contains('light-theme') ? 'light' : 'dark';
        const newTheme = currentTheme === 'light' ? 'dark' : 'light';
        document.body.className = `${newTheme}-theme`;
        localStorage.setItem('theme', newTheme);
        updateThemeIcon(newTheme);
    });
}

function updateThemeIcon(theme) {
    const icon = themeToggle.querySelector('.theme-icon');
    icon.textContent = theme === 'light' ? '🌙' : '☀️';
}

// Tab Navigation
function initTabs() {
    navTabs.forEach(tab => {
        tab.addEventListener('click', () => {
            const targetTab = tab.dataset.tab;
            
            // Update active states
            navTabs.forEach(t => t.classList.remove('active'));
            tab.classList.add('active');
            
            tabContents.forEach(content => {
                content.classList.remove('active');
                if (content.id === `${targetTab}-tab`) {
                    content.classList.add('active');
                }
            });
        });
    });
}

// Input Handling
function initInput() {
    modeSelect.addEventListener('change', (e) => {
        currentMode = e.target.value;
    });

    sendButton.addEventListener('click', handleSubmit);
    
    queryInput.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            handleSubmit();
        }
    });
}

async function handleSubmit() {
    const query = queryInput.value.trim();
    if (!query) return;

    // Clear input
    queryInput.value = '';
    
    // Remove welcome message if it exists
    const welcomeMsg = chatMessages.querySelector('.welcome-message');
    if (welcomeMsg) {
        welcomeMsg.remove();
    }

    // Add user message
    addMessage(query, 'user');
    
    // Add loading indicator
    const loadingDiv = document.createElement('div');
    loadingDiv.className = 'loading';
    loadingDiv.textContent = 'Processing...';
    chatMessages.appendChild(loadingDiv);
    chatMessages.scrollTop = chatMessages.scrollHeight;

    try {
        let result;
        
        if (currentMode === 'ask') {
            result = await invoke('ask_command', {
                query: query,
                directory: currentDirectory,
                printOnly: false
            });
        } else {
            result = await invoke('search_command', {
                query: query,
                directory: currentDirectory,
                printOnly: false
            });
        }

        // Remove loading indicator
        loadingDiv.remove();
        
        // Parse result
        try {
            const parsedResult = JSON.parse(result);
            
            if (parsedResult.results && Array.isArray(parsedResult.results)) {
                displayResults(parsedResult.results, parsedResult.query);
            } else {
                addMessage(result, 'assistant');
            }
        } catch {
            // If not JSON, just display as text
            addMessage(result, 'assistant');
        }
        
    } catch (error) {
        loadingDiv.remove();
        addMessage(`Error: ${error}`, 'error');
    }
}

function addMessage(content, type) {
    const messageDiv = document.createElement('div');
    messageDiv.className = `message ${type}`;
    messageDiv.textContent = content;
    chatMessages.appendChild(messageDiv);
    chatMessages.scrollTop = chatMessages.scrollHeight;
}

function displayResults(results, query) {
    const resultsContainer = document.createElement('div');
    resultsContainer.className = 'results-container';
    
    const title = document.createElement('div');
    title.className = 'results-title';
    title.textContent = `Found ${results.length} result(s) for "${query}"`;
    resultsContainer.appendChild(title);
    
    results.forEach((result, index) => {
        const resultItem = document.createElement('div');
        resultItem.className = 'result-item';
        
        const icon = document.createElement('span');
        icon.className = 'result-icon';
        icon.textContent = getIconForType(result.entry_type);
        
        const content = document.createElement('div');
        content.className = 'result-content';
        content.textContent = result.content;
        
        resultItem.appendChild(icon);
        resultItem.appendChild(content);
        
        resultItem.addEventListener('click', () => {
            handleResultClick(result);
        });
        
        resultsContainer.appendChild(resultItem);
    });
    
    chatMessages.appendChild(resultsContainer);
    chatMessages.scrollTop = chatMessages.scrollHeight;
}

function getIconForType(type) {
    const icons = {
        'clipboard': '📋',
        'shell': '💻',
        'file': '📄',
        'note': '📝'
    };
    return icons[type] || '📄';
}

function handleResultClick(result) {
    addMessage(`Selected: ${result.content}`, 'assistant');
    // You can add more logic here for what happens when a result is clicked
}

// Paths Management
async function loadPaths() {
    try {
        const response = await fetch('paths.json');
        const paths = await response.json();
        displayPaths(paths);
    } catch (error) {
        pathsList.innerHTML = '<p class="placeholder-text">No paths configured. Create a paths.json file.</p>';
    }
}

function displayPaths(paths) {
    pathsList.innerHTML = '';
    
    paths.forEach(path => {
        const pathItem = document.createElement('div');
        pathItem.className = 'path-item';
        
        const pathInfo = document.createElement('div');
        pathInfo.className = 'path-info';
        
        const label = document.createElement('div');
        label.className = 'path-label';
        label.textContent = path.label;
        
        const value = document.createElement('div');
        value.className = 'path-value';
        value.textContent = path.path;
        
        pathInfo.appendChild(label);
        pathInfo.appendChild(value);
        
        const copyButton = document.createElement('button');
        copyButton.className = 'copy-button';
        copyButton.textContent = 'Copy';
        copyButton.addEventListener('click', () => copyPath(path.path, copyButton));
        
        pathItem.appendChild(pathInfo);
        pathItem.appendChild(copyButton);
        pathsList.appendChild(pathItem);
    });
}

async function copyPath(path, button) {
    try {
        await navigator.clipboard.writeText(path);
        button.textContent = '✓ Copied';
        button.classList.add('copied');
        
        setTimeout(() => {
            button.textContent = 'Copy';
            button.classList.remove('copied');
        }, 2000);
    } catch (error) {
        console.error('Failed to copy:', error);
    }
}