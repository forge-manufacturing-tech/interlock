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
            <div className="min-h-screen bg-industrial-steel-950 flex items-center justify-center">
                <div className="text-industrial-steel-400 font-mono uppercase tracking-wider">Loading...</div>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-industrial-steel-950 text-neutral-100 flex flex-col metal-texture">
            {/* Header */}
            <header className="border-b border-industrial-concrete bg-industrial-steel-900/80 backdrop-blur-sm">
                <div className="px-6 py-4 flex items-center justify-between">
                    <div className="flex items-center gap-4">
                        <button
                            onClick={() => navigate('/')}
                            className="text-industrial-steel-400 hover:text-industrial-copper-500 transition-colors font-mono text-sm uppercase tracking-wide"
                        >
                            ← Back
                        </button>
                        <h1 className="industrial-headline text-xl">{project?.name}</h1>
                    </div>
                    <button
                        onClick={() => setShowCreateModal(true)}
                        className="px-4 py-2 industrial-btn rounded-sm text-xs"
                    >
                        + New Session
                    </button>
                </div>
            </header>

            {/* Main Layout */}
            <div className="flex flex-1 overflow-hidden">
                {/* Sessions Sidebar */}
                <div className="w-80 border-r border-industrial-concrete bg-industrial-steel-900/50 overflow-y-auto scanlines">
                    <div className="p-4">
                        <h2 className="text-xs font-bold text-industrial-steel-400 uppercase tracking-widest mb-4 font-mono">
                            Work Sessions
                        </h2>
                        {sessions.length === 0 ? (
                            <div className="text-center py-8 text-industrial-steel-500 text-sm font-mono">
                                No sessions yet
                            </div>
                        ) : (
                            <div className="space-y-2">
                                {sessions.map((session) => (
                                    <div
                                        key={session.id}
                                        onClick={() => selectSession(session)}
                                        className={`group p-3 rounded-sm border cursor-pointer transition-all ${selectedSession?.id === session.id
                                            ? 'bg-industrial-copper-500/20 border-industrial-copper-500/50 shadow-glow-copper'
                                            : 'bg-industrial-steel-800/60 border-industrial-concrete hover:border-industrial-copper-500/30 hover:bg-industrial-steel-700/80'
                                            }`}
                                    >
                                        <div className="flex items-start justify-between gap-2">
                                            <div className="flex-1 min-w-0">
                                                <h3 className="text-sm font-bold text-neutral-100 truncate tracking-wide">
                                                    {session.title || 'Untitled Session'}
                                                </h3>
                                                <p className="text-xs text-industrial-steel-500 mt-1 font-mono">
                                                    {new Date(session.created_at).toLocaleDateString()}
                                                </p>
                                            </div>
                                            <button
                                                onClick={(e) => {
                                                    e.stopPropagation();
                                                    deleteSession(session.id);
                                                }}
                                                className="opacity-0 group-hover:opacity-100 text-industrial-steel-500 hover:text-industrial-alert transition-all"
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
                <div className="flex-1 overflow-y-auto bg-industrial-steel-950">
                    {selectedSession ? (
                        <div className="p-8">
                            <div className="max-w-4xl mx-auto">
                                <h2 className="industrial-headline text-2xl mb-4">{selectedSession.title || 'Untitled'}</h2>
                                <div className="industrial-panel rounded-sm p-6">
                                    <pre className="text-sm text-neutral-300 whitespace-pre-wrap font-mono leading-relaxed">
                                        {selectedSession.content || 'No content'}
                                    </pre>
                                </div>
                            </div>
                        </div>
                    ) : (
                        <div className="flex items-center justify-center h-full">
                            <div className="text-center text-industrial-steel-500">
                                <svg className="w-16 h-16 mx-auto mb-4 text-industrial-steel-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                </svg>
                                <p className="font-mono uppercase tracking-wide text-sm">Select a session to view content</p>
                            </div>
                        </div>
                    )}
                </div>
            </div>

            {/* Create Session Modal */}
            {showCreateModal && (
                <div
                    className="fixed inset-0 bg-black/90 flex items-center justify-center p-4 z-50 backdrop-blur-sm"
                    onClick={() => setShowCreateModal(false)}
                >
                    <div
                        className="industrial-panel rounded-sm p-6 w-full max-w-md"
                        onClick={(e) => e.stopPropagation()}
                    >
                        <h3 className="industrial-headline text-xl mb-4">Create New Session</h3>
                        <form onSubmit={handleCreateSession} className="space-y-4">
                            <div>
                                <label className="block text-xs font-bold text-industrial-steel-300 mb-2 uppercase tracking-wider">
                                    Session Title
                                </label>
                                <input
                                    type="text"
                                    value={newSessionTitle}
                                    onChange={(e) => setNewSessionTitle(e.target.value)}
                                    className="w-full px-4 py-2 industrial-input rounded-sm"
                                    required
                                    autoFocus
                                    placeholder="e.g., Login Flow Development"
                                />
                            </div>
                            <div className="flex gap-3 pt-2">
                                <button
                                    type="button"
                                    onClick={() => setShowCreateModal(false)}
                                    className="flex-1 px-4 py-2 bg-industrial-steel-800 hover:bg-industrial-steel-700 border border-industrial-concrete rounded-sm transition-colors uppercase tracking-wide text-sm font-bold"
                                >
                                    Cancel
                                </button>
                                <button
                                    type="submit"
                                    className="flex-1 px-4 py-2 industrial-btn rounded-sm"
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
