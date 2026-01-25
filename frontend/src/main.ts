import './style.css'
import { createIcons, Lock, UserPlus, Database, PlusSquare, FileCode, ChevronRight, ArrowLeft, HardDrive } from 'lucide'
import { OpenAPI } from './api/generated/core/OpenAPI';
import { ControllersAuthService } from './api/generated/services/ControllersAuthService';
import { ControllersSessionsService } from './api/generated/services/ControllersSessionsService';


const API_URL = import.meta.env.VITE_API_URL;
OpenAPI.BASE = API_URL.replace(/\/api$/, '');
OpenAPI.TOKEN = async () => localStorage.getItem('token') || '';

let token: string | null = localStorage.getItem('token');
let currentSessionId: string | null = null;
let saveTimeout: any = null;
let pollInterval: any = null;

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

declare global { // Extend window object
    interface Window {
        toggleAuth: () => void;
        register: () => Promise<void>;
        login: (emailIn?: string, passwordIn?: string) => Promise<void>;
        logout: () => void;
        showDashboard: () => Promise<void>;
        createSession: () => Promise<void>;
        openSession: (id: string) => Promise<void>;
        handleInput: () => void;
        handleTitleInput: () => void;
    }
}

window.toggleAuth = function () {
    document.getElementById('login-form')!.classList.toggle('hidden');
    document.getElementById('register-form')!.classList.toggle('hidden');
    document.getElementById('error-msg')!.classList.add('hidden');
    // Re-init icons just in case
    setTimeout(() => initIcons(), 50);
}

function handleApiError(e: any) {
    let msg = e.message || 'Unknown error';
    if (e.body) {
        if (typeof e.body === 'string') {
            msg = e.body;
        } else if (e.body.description) {
            msg = e.body.description;
        } else if (e.body.error) {
            msg = e.body.error;
        }
    }
    showError(msg);
}

window.register = async function () {
    const nameInput = document.getElementById('reg-name') as HTMLInputElement;
    const emailInput = document.getElementById('reg-email') as HTMLInputElement;
    const passwordInput = document.getElementById('reg-password') as HTMLInputElement;

    const name = nameInput.value;
    const email = emailInput.value;
    const password = passwordInput.value;

    try {
        await ControllersAuthService.register({ name, email, password });
        await window.login(email, password);
    } catch (e: any) {
        handleApiError(e);
    }
}

window.login = async function (emailIn, passwordIn) {
    const emailInput = document.getElementById('login-email') as HTMLInputElement;
    const passwordInput = document.getElementById('login-password') as HTMLInputElement;

    const email = emailIn || emailInput.value;
    const password = passwordIn || passwordInput.value;

    try {
        const data = await ControllersAuthService.login({ email, password });
        token = data.token;
        localStorage.setItem('token', token!);
        window.showDashboard();
    } catch (e: any) {
        handleApiError(e);
    }
}

window.logout = function () {
    token = null;
    localStorage.removeItem('token');
    document.getElementById('auth-section')!.classList.remove('hidden');
    document.getElementById('dashboard-section')!.classList.add('hidden');
    document.getElementById('editor-section')!.classList.add('hidden');

    // Allow animation to play nicely by resetting forms
    document.getElementById('login-form')!.classList.remove('hidden');
    document.getElementById('register-form')!.classList.add('hidden');
}

function showError(msg: string) {
    const el = document.getElementById('error-msg')!;
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
        displayMsg = 'Server unreachable';
    }

    // Normalized error display (no uppercase, no prefix)
    const finalMsg = displayMsg;

    el.textContent = finalMsg;
    el.classList.remove('hidden');

    // Auto-hide after 10 seconds unless it's a critical error
    setTimeout(() => {
        if (el.textContent === finalMsg) {
            el.classList.add('hidden');
        }
    }, 10000);
}

function checkAuth() {
    if (token) window.showDashboard();
}

window.showDashboard = async function () {
    document.getElementById('auth-section')!.classList.add('hidden');
    document.getElementById('dashboard-section')!.classList.remove('hidden');
    document.getElementById('editor-section')!.classList.add('hidden');

    // Stop polling if active
    if (pollInterval) clearInterval(pollInterval);

    loadSessions();

    // Re-render icons for new visible sections
    setTimeout(() => initIcons(), 50);
}

async function loadSessions() {
    try {
        // Services automatically use the token from OpenAPI.TOKEN
        const sessions = await ControllersSessionsService.list();
        const list = document.getElementById('session-list')!;

        if (sessions.length === 0) {
            list.innerHTML = `
             <div class="p-8 border border-dashed border-neutral-800 bg-neutral-900/20 text-center flex flex-col items-center justify-center gap-2">
                <i data-lucide="hard-drive" class="text-neutral-700 w-8 h-8"></i>
                <span class="text-neutral-600 text-xs uppercase tracking-widest">No active sessions</span>
             </div>`;
        } else {
            list.innerHTML = sessions.map(s => `
                <div class="group flex items-center justify-between p-4 bg-neutral-900/40 border border-neutral-800 hover:border-orange-500/50 hover:bg-neutral-900/80 transition-all cursor-pointer" onclick="openSession('${s.id}')">
                    <div class="flex items-center gap-3">
                        <div class="p-2 bg-neutral-950 border border-neutral-800 group-hover:border-orange-500/30 rounded-sm">
                            <i data-lucide="file-code" class="w-4 h-4 text-neutral-500 group-hover:text-orange-500 transition-colors"></i>
                        </div>
                        <div>
                            <h3 class="text-sm font-bold text-neutral-200 group-hover:text-white font-mono">${s.title || 'Untitled Session'}</h3>
                            <p class="text-[10px] text-neutral-600 uppercase">Last Sync: ${new Date(s.updated_at).toLocaleTimeString()}</p>
                        </div>
                    </div>
                    <i data-lucide="chevron-right" class="w-4 h-4 text-neutral-700 group-hover:text-orange-500 transition-colors"></i>
                </div>
            `).join('');
        }
        initIcons();
    } catch (e: any) {
        if (e.status === 401) {
            window.logout();
            return;
        }
        handleApiError(e);
    }
}

window.createSession = async function () {
    const input = document.getElementById('new-session-title') as HTMLInputElement;
    const title = input.value;
    if (!title) return;
    try {
        await ControllersSessionsService.add({ title, content: '' });
        input.value = '';
        loadSessions();
    } catch (e: any) {
        handleApiError(e);
    }
}

window.openSession = async function (id: string) {
    currentSessionId = id;
    try {
        const session = await ControllersSessionsService.getOne(id);
        const titleInput = document.getElementById('editor-title') as HTMLInputElement;
        const editor = document.getElementById('editor') as HTMLTextAreaElement;

        titleInput.value = session.title || 'Untitled';
        editor.value = session.content || '';

        document.getElementById('dashboard-section')!.classList.add('hidden');
        document.getElementById('editor-section')!.classList.remove('hidden');

        // Re-init icons
        setTimeout(() => initIcons(), 50);

        // Start polling
        pollInterval = setInterval(refreshSession, 2000);
    } catch (e: any) {
        handleApiError(e);
    }
}

window.handleInput = function () {
    const saveStatus = document.getElementById('save-status')!;
    const statusIndicator = document.getElementById('status-indicator')!;

    saveStatus.textContent = 'Saving...';
    saveStatus.classList.remove('text-neutral-500');
    saveStatus.classList.add('text-yellow-500');

    statusIndicator.classList.remove('bg-neutral-600');
    statusIndicator.classList.add('bg-yellow-500', 'animate-pulse');

    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(saveSession, 500);
}

window.handleTitleInput = function () {
    const saveStatus = document.getElementById('save-status')!;
    const statusIndicator = document.getElementById('status-indicator')!;

    saveStatus.textContent = 'Saving...';
    saveStatus.classList.remove('text-neutral-500');
    saveStatus.classList.add('text-yellow-500');

    statusIndicator.classList.remove('bg-neutral-600');
    statusIndicator.classList.add('bg-yellow-500', 'animate-pulse');

    clearTimeout(saveTimeout);
    saveTimeout = setTimeout(saveSession, 1000);
}

async function saveSession() {
    if (!currentSessionId) return;
    const editor = document.getElementById('editor') as HTMLTextAreaElement;
    const titleInput = document.getElementById('editor-title') as HTMLInputElement;
    const saveStatus = document.getElementById('save-status')!;
    const statusIndicator = document.getElementById('status-indicator')!;

    const content = editor.value;
    const title = titleInput.value;
    try {
        await ControllersSessionsService.update(currentSessionId, { title, content });
        saveStatus.textContent = 'Saved';
        saveStatus.classList.remove('text-yellow-500');
        saveStatus.classList.add('text-neutral-500');

        statusIndicator.classList.remove('bg-yellow-500', 'animate-pulse');
        statusIndicator.classList.add('bg-neutral-600');
    } catch (e) {
        saveStatus.textContent = 'Save Failed';
        saveStatus.classList.add('text-red-500');
        statusIndicator.classList.add('bg-red-500');
    }
}

async function refreshSession() {
    if (document.getElementById('dashboard-section')!.classList.contains('hidden') === false) {
        if (pollInterval) clearInterval(pollInterval);
        return;
    }
    // Logic unchanged from original: avoid overwriting if typing
}

checkAuth();
