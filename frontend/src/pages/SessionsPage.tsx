import React, { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { ControllersSessionsService, ControllersProjectsService, SessionResponse, ProjectResponse } from '../api/generated';
import { useAuth } from '../contexts/AuthContext';

export function SessionsPage() {
    const { projectId } = useParams<{ projectId: string }>();
    const [project, setProject] = useState<ProjectResponse | null>(null);
    const [sessions, setSessions] = useState<SessionResponse[]>([]);
    const [loading, setLoading] = useState(true);
    const [showCreateModal, setShowCreateModal] = useState(false);
    const [newSessionTitle, setNewSessionTitle] = useState('');
    const [selectedSession, setSelectedSession] = useState<SessionResponse | null>(null);
    const { logout } = useAuth();
    const navigate = useNavigate();

    useEffect(() => {
        if (projectId) {
            loadProjectAndSessions();
        }
    }, [projectId]);

    const loadProjectAndSessions = async () => {
        if (!projectId) return;

        try {
            setLoading(true);
            const [projectData, sessionsData] = await Promise.all([
                ControllersProjectsService.getOne(projectId),
                ControllersSessionsService.list(projectId),
            ]);
            setProject(projectData);
            setSessions(sessionsData);
        } catch (error: any) {
            if (error.status === 401) {
                logout();
            } else if (error.status === 403 || error.status === 404) {
                navigate('/');
            }
        } finally {
            setLoading(false);
        }
    };

    const handleCreateSession = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!projectId) return;

        try {
            await ControllersSessionsService.add({
                title: newSessionTitle,
                content: '',
                project_id: projectId,
            });
            setShowCreateModal(false);
            setNewSessionTitle('');
            loadProjectAndSessions();
        } catch (error) {
            console.error('Failed to create session:', error);
        }
    };

    const selectSession = (session: SessionResponse) => {
        setSelectedSession(session);
    };

    const deleteSession = async (sessionId: string) => {
        if (!confirm('Are you sure you want to delete this session?')) return;

        try {
            await ControllersSessionsService.remove(sessionId);
            if (selectedSession?.id === sessionId) {
                setSelectedSession(null);
            }
            loadProjectAndSessions();
        } catch (error) {
            console.error('Failed to delete session:', error);
        }
    };

    if (loading) {
        return (
            <div className="min-h-screen bg-black flex items-center justify-center">
                <div className="text-neutral-500">Loading...</div>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-black text-white flex flex-col">
            {/* Header */}
            <header className="border-b border-neutral-800 bg-neutral-950/50 backdrop-blur-sm">
                <div className="px-6 py-4 flex items-center justify-between">
                    <div className="flex items-center gap-4">
                        <button
                            onClick={() => navigate('/')}
                            className="text-neutral-500 hover:text-white transition-colors"
                        >
                            ← Back
                        </button>
                        <h1 className="text-xl font-bold">{project?.name}</h1>
                    </div>
                    <button
                        onClick={() => setShowCreateModal(true)}
                        className="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded text-sm transition-colors"
                    >
                        + New Session
                    </button>
                </div>
            </header>

            {/* Main Layout */}
            <div className="flex flex-1 overflow-hidden">
                {/* Sessions Sidebar */}
                <div className="w-80 border-r border-neutral-800 bg-neutral-950/30 overflow-y-auto">
                    <div className="p-4">
                        <h2 className="text-sm font-bold text-neutral-400 uppercase tracking-wider mb-4">
                            Sessions
                        </h2>
                        {sessions.length === 0 ? (
                            <div className="text-center py-8 text-neutral-600 text-sm">
                                No sessions yet
                            </div>
                        ) : (
                            <div className="space-y-2">
                                {sessions.map((session) => (
                                    <div
                                        key={session.id}
                                        onClick={() => selectSession(session)}
                                        className={`group p-3 rounded border cursor-pointer transition-all ${selectedSession?.id === session.id
                                                ? 'bg-blue-600/20 border-blue-500/50'
                                                : 'bg-neutral-900/40 border-neutral-800 hover:border-neutral-700 hover:bg-neutral-900/60'
                                            }`}
                                    >
                                        <div className="flex items-start justify-between gap-2">
                                            <div className="flex-1 min-w-0">
                                                <h3 className="text-sm font-medium text-white truncate">
                                                    {session.title || 'Untitled Session'}
                                                </h3>
                                                <p className="text-xs text-neutral-600 mt-1">
                                                    {new Date(session.created_at).toLocaleDateString()}
                                                </p>
                                            </div>
                                            <button
                                                onClick={(e) => {
                                                    e.stopPropagation();
                                                    deleteSession(session.id);
                                                }}
                                                className="opacity-0 group-hover:opacity-100 text-neutral-600 hover:text-red-500 transition-all"
                                            >
                                                <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                                                </svg>
                                            </button>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        )}
                    </div>
                </div>

                {/* Content Area */}
                <div className="flex-1 overflow-y-auto">
                    {selectedSession ? (
                        <div className="p-8">
                            <div className="max-w-4xl mx-auto">
                                <h2 className="text-2xl font-bold mb-4">{selectedSession.title || 'Untitled'}</h2>
                                <div className="bg-neutral-950 border border-neutral-800 rounded-lg p-6">
                                    <pre className="text-sm text-neutral-300 whitespace-pre-wrap font-mono">
                                        {selectedSession.content || 'No content'}
                                    </pre>
                                </div>
                            </div>
                        </div>
                    ) : (
                        <div className="flex items-center justify-center h-full">
                            <div className="text-center text-neutral-600">
                                <p>Select a session to view its content</p>
                            </div>
                        </div>
                    )}
                </div>
            </div>

            {/* Create Session Modal */}
            {showCreateModal && (
                <div
                    className="fixed inset-0 bg-black/80 flex items-center justify-center p-4 z-50"
                    onClick={() => setShowCreateModal(false)}
                >
                    <div
                        className="bg-neutral-950 border border-neutral-800 rounded-lg p-6 w-full max-w-md"
                        onClick={(e) => e.stopPropagation()}
                    >
                        <h3 className="text-xl font-bold mb-4">Create New Session</h3>
                        <form onSubmit={handleCreateSession} className="space-y-4">
                            <div>
                                <label className="block text-sm font-medium text-neutral-400 mb-1">
                                    Session Title
                                </label>
                                <input
                                    type="text"
                                    value={newSessionTitle}
                                    onChange={(e) => setNewSessionTitle(e.target.value)}
                                    className="w-full px-4 py-2 bg-neutral-900 border border-neutral-800 rounded text-white focus:outline-none focus:border-blue-500"
                                    required
                                    autoFocus
                                    placeholder="e.g., Login Flow Development"
                                />
                            </div>
                            <div className="flex gap-3">
                                <button
                                    type="button"
                                    onClick={() => setShowCreateModal(false)}
                                    className="flex-1 px-4 py-2 bg-neutral-900 hover:bg-neutral-800 border border-neutral-800 rounded transition-colors"
                                >
                                    Cancel
                                </button>
                                <button
                                    type="submit"
                                    className="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded transition-colors"
                                >
                                    Create
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            )}
        </div>
    );
}
