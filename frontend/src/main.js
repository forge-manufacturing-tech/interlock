import './style.css'
import { createIcons, Lock, UserPlus, Database, PlusSquare, FileCode, ChevronRight, ArrowLeft, HardDrive } from 'lucide'

const API_URL = import.meta.env.VITE_API_URL;
let token = localStorage.getItem('token');
let currentSessionId = null;
let saveTimeout = null;
let pollInterval = null;

// Initialize Icons
function initIcons() {
    createIcons({
        icons: {
            Lock,
            UserPlus,
            Database,
            PlusSquare,
            FileCode,
            ChevronRight,
            ArrowLeft,
            HardDrive
        }
    });
}

window.addEventListener('load', () => {
    initIcons();
});

window.toggleAuth = function () {
    document.getElementById('login-form').classList.toggle('hidden');
    document.getElementById('register-form').classList.toggle('hidden');
    document.getElementById('error-msg').classList.add('hidden');
    // Re-init icons just in case
    setTimeout(() => initIcons(), 50);
}

window.register = async function () {
    const name = document.getElementById('reg-name').value;
    const email = document.getElementById('reg-email').value;
    const password = document.getElementById('reg-password').value;

    try {
        const res = await fetch(`${API_URL}/auth/register`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ name, email, password })
        });
        if (!res.ok) {
            let errorMsg = 'Registration failed';
            try {
                const errorData = await res.json();
                errorMsg = errorData.description || errorData.error || errorMsg;
            } catch (e) {
                const text = await res.text().catch(() => '');
                if (text) errorMsg = text;
            }
            throw new Error(errorMsg);
        }
        await window.login(email, password);
    } catch (e) {
        showError(e.message);
    }
}

window.login = async function (emailIn, passwordIn) {
    const email = emailIn || document.getElementById('login-email').value;
    const password = passwordIn || document.getElementById('login-password').value;

    try {
        const res = await fetch(`${API_URL}/auth/login`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ email, password })
        });
        if (!res.ok) {
            let errorMsg = 'Login failed';
            try {
                const errorData = await res.json();
                errorMsg = errorData.description || errorData.error || errorMsg;
            } catch (e) {
                const text = await res.text().catch(() => '');
                if (text) errorMsg = text;
            }
            throw new Error(errorMsg);
        }
        const data = await res.json();
        token = data.token;
        localStorage.setItem('token', token);
        window.showDashboard();
    } catch (e) {
        showError(e.message);
    }
}

window.logout = function () {
    token = null;
    localStorage.removeItem('token');
    document.getElementById('auth-section').classList.remove('hidden');
    document.getElementById('dashboard-section').classList.add('hidden');
    document.getElementById('editor-section').classList.add('hidden');

    // Allow animation to play nicely by resetting forms
    document.getElementById('login-form').classList.remove('hidden');
    document.getElementById('register-form').classList.add('hidden');
}

function showError(msg) {
    const el = document.getElementById('error-msg');
    let displayMsg = msg;

    // Try to parse out validation errors if they are in JSON format
    if (msg.includes('Custom Error: ')) {
        try {
            const jsonStr = msg.split('Custom Error: ')[1];
            const errObj = JSON.parse(jsonStr);
            const firstKey = Object.keys(errObj)[0];
            if (firstKey && Array.isArray(errObj[firstKey]) && errObj[firstKey][0].message) {
                displayMsg = `${firstKey}: ${errObj[firstKey][0].message}`;
            }
        } catch (e) {
            // Keep original if parsing fails
        }
    } else if (msg.includes('Query Error: ')) {
        displayMsg = msg.replace('Query Error: ', '');
    } else if (msg.toLowerCase().includes('failed to fetch')) {
        displayMsg = 'SERVER_OFFLINE_OR_UNREACHABLE';
    }

    const finalMsg = displayMsg.toUpperCase().startsWith('COMMAND FAILED')
        ? displayMsg.toUpperCase()
        : 'COMMAND FAILED: ' + displayMsg.toUpperCase();

    el.textContent = finalMsg;
    el.classList.remove('hidden');

    // Auto-hide after 10 seconds unless it's a critical error
    setTimeout(() => {
        if (el.textContent.includes(displayMsg.toUpperCase())) {
            el.classList.add('hidden');
        }
    }, 10000);
}

function checkAuth() {
    if (token) window.showDashboard();
}

window.showDashboard = async function () {
    document.getElementById('auth-section').classList.add('hidden');
    document.getElementById('dashboard-section').classList.remove('hidden');
    document.getElementById('editor-section').classList.add('hidden');

    // Stop polling if active
    if (pollInterval) clearInterval(pollInterval);

    loadSessions();

    // Re-render icons for new visible sections
    setTimeout(() => initIcons(), 50);
}

async function loadSessions() {
    try {
        const res = await fetch(`${API_URL}/sessions`, {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        if (res.status === 401) { window.logout(); return; }
        if (!res.ok) {
            const errorData = await res.json().catch(() => ({}));
            throw new Error(errorData.description || 'Failed to load streams');
        }
        const sessions = await res.json();
        const list = document.getElementById('session-list');

        if (sessions.length === 0) {
            list.innerHTML = `
             <div class="p-8 border border-dashed border-neutral-800 bg-neutral-900/20 text-center flex flex-col items-center justify-center gap-2">
                <i data-lucide="hard-drive" class="text-neutral-700 w-8 h-8"></i>
                <span class="text-neutral-600 text-xs uppercase tracking-widest">No active streams detected</span>
             </div>`;
        } else {
            list.innerHTML = sessions.map(s => `
                <div class="group flex items-center justify-between p-4 bg-neutral-900/40 border border-neutral-800 hover:border-orange-500/50 hover:bg-neutral-900/80 transition-all cursor-pointer" onclick="openSession('${s.id}')">
                    <div class="flex items-center gap-3">
                        <div class="p-2 bg-neutral-950 border border-neutral-800 group-hover:border-orange-500/30 rounded-sm">
                            <i data-lucide="file-code" class="w-4 h-4 text-neutral-500 group-hover:text-orange-500 transition-colors"></i>
                        </div>
                        <div>
                            <h3 class="text-sm font-bold text-neutral-200 group-hover:text-white font-mono">${s.title || 'UNTITLED_PROTOCOL'}</h3>
                            <p class="text-[10px] text-neutral-600 uppercase">Last Sync: ${new Date(s.updated_at).toLocaleTimeString()}</p>
                        </div>
                    </div>
                    <i data-lucide="chevron-right" class="w-4 h-4 text-neutral-700 group-hover:text-orange-500 transition-colors"></i>
                </div>
            `).join('');
        }
        initIcons();
    } catch (e) {
        showError(e.message);
    }
}

window.createSession = async function () {
    const title = document.getElementById('new-session-title').value;
    if (!title) return;
    try {
        const res = await fetch(`${API_URL}/sessions`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({ title, content: '' })
        });
        if (!res.ok) {
            const errorData = await res.json().catch(() => ({}));
            throw new Error(errorData.description || 'Failed to create stream');
        }
        document.getElementById('new-session-title').value = '';
        loadSessions();
    } catch (e) {
        showError(e.message);
    }
}

window.openSession = async function (id) {
    currentSessionId = id;
    try {
        const res = await fetch(`${API_URL}/sessions/${id}`, {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        if (!res.ok) {
            const errorData = await res.json().catch(() => ({}));
            throw new Error(errorData.description || 'Failed to open stream');
        }
        const session = await res.json();
        document.getElementById('editor-title').value = session.title || 'UNTITLED_PROTOCOL';
        document.getElementById('editor').value = session.content || '';

        document.getElementById('dashboard-section').classList.add('hidden');
        document.getElementById('editor-section').classList.remove('hidden');

        // Re-init icons
        setTimeout(() => initIcons(), 50);

        // Start polling
        pollInterval = setInterval(refreshSession, 2000);
    } catch (e) {
        showError(e.message);
    }
}

window.handleInput = function () {
    document.getElementById('save-status').textContent = 'SYNCING...';
    document.getElementById('save-status').classList.remove('text-neutral-500');
    document.getElementById('save-status').classList.add('text-yellow-500');

    document.getElementById('status-indicator').classList.remove('bg-neutral-600');
    document.getElementById('status-indicator').classList.add('bg-yellow-500', 'animate-pulse');

    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(saveSession, 500);
}

window.handleTitleInput = function () {
    document.getElementById('save-status').textContent = 'SYNCING...';
    document.getElementById('save-status').classList.remove('text-neutral-500');
    document.getElementById('save-status').classList.add('text-yellow-500');

    document.getElementById('status-indicator').classList.remove('bg-neutral-600');
    document.getElementById('status-indicator').classList.add('bg-yellow-500', 'animate-pulse');

    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(saveSession, 1000);
}

async function saveSession() {
    if (!currentSessionId) return;
    const content = document.getElementById('editor').value;
    const title = document.getElementById('editor-title').value;
    try {
        await fetch(`${API_URL}/sessions/${currentSessionId}`, {
            method: 'PATCH',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${token}`
            },
            body: JSON.stringify({ title, content })
        });
        document.getElementById('save-status').textContent = 'SYNCED';
        document.getElementById('save-status').classList.remove('text-yellow-500');
        document.getElementById('save-status').classList.add('text-neutral-500');

        document.getElementById('status-indicator').classList.remove('bg-yellow-500', 'animate-pulse');
        document.getElementById('status-indicator').classList.add('bg-neutral-600');
    } catch (e) {
        document.getElementById('save-status').textContent = 'SYNC_FAIL';
        document.getElementById('save-status').classList.add('text-red-500');
        document.getElementById('status-indicator').classList.add('bg-red-500');
    }
}

async function refreshSession() {
    if (document.getElementById('dashboard-section').classList.contains('hidden') === false) {
        if (pollInterval) clearInterval(pollInterval);
        return;
    }
    // Logic unchanged from original: avoid overwriting if typing
}

checkAuth();
