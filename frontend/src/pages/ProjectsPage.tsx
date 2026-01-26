import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { ControllersProjectsService, ProjectResponse } from '../api/generated';
import { useAuth } from '../contexts/AuthContext';

export function ProjectsPage() {
    const [projects, setProjects] = useState<ProjectResponse[]>([]);
    const [loading, setLoading] = useState(true);
    const [showCreateModal, setShowCreateModal] = useState(false);
    const [newProjectName, setNewProjectName] = useState('');
    const [newProjectDescription, setNewProjectDescription] = useState('');
    const { logout, user } = useAuth();
    const navigate = useNavigate();

    useEffect(() => {
        loadProjects();
    }, []);

    const loadProjects = async () => {
        try {
            setLoading(true);
            const data = await ControllersProjectsService.list();
            setProjects(data);
        } catch (error: any) {
            if (error.status === 401) {
                logout();
            }
        } finally {
            setLoading(false);
        }
    };

    const handleCreateProject = async (e: React.FormEvent) => {
        e.preventDefault();
        try {
            await ControllersProjectsService.create({
                name: newProjectName,
                description: newProjectDescription || undefined,
            });
            setShowCreateModal(false);
            setNewProjectName('');
            setNewProjectDescription('');
            loadProjects();
        } catch (error) {
            console.error('Failed to create project:', error);
        }
    };

    const openProject = (projectId: string) => {
        navigate(`/projects/${projectId}`);
    };

    if (loading) {
        return (
            <div className="min-h-screen bg-black flex items-center justify-center">
                <div className="text-neutral-500">Loading...</div>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-black text-white">
            {/* Header */}
            <header className="border-b border-neutral-800 bg-neutral-950/50 backdrop-blur-sm">
                <div className="max-w-7xl mx-auto px-6 py-4 flex items-center justify-between">
                    <h1 className="text-xl font-bold">Interlock</h1>
                    <div className="flex items-center gap-4">
                        <span className="text-sm text-neutral-500">{user?.email}</span>
                        <button
                            onClick={logout}
                            className="px-4 py-2 text-sm bg-neutral-900 hover:bg-neutral-800 border border-neutral-800 rounded transition-colors"
                        >
                            Logout
                        </button>
                    </div>
                </div>
            </header>

            {/* Main Content */}
            <main className="max-w-7xl mx-auto px-6 py-8">
                <div className="flex items-center justify-between mb-8">
                    <div>
                        <h2 className="text-2xl font-bold mb-2">Projects</h2>
                        <p className="text-neutral-500 text-sm">Manage your coding workspaces</p>
                    </div>
                    <button
                        onClick={() => setShowCreateModal(true)}
                        className="px-6 py-2 bg-blue-600 hover:bg-blue-700 rounded font-medium transition-colors"
                    >
                        + New Project
                    </button>
                </div>

                {/* Projects Grid */}
                {projects.length === 0 ? (
                    <div className="p-16 border border-dashed border-neutral-800 bg-neutral-900/20 text-center rounded-lg">
                        <p className="text-neutral-600 text-sm uppercase tracking-widest">No projects found</p>
                        <p className="text-neutral-700 text-xs mt-2">Create your first project to get started</p>
                    </div>
                ) : (
                    <div className="grid gap-4">
                        {projects.map((project) => (
                            <div
                                key={project.id}
                                onClick={() => openProject(project.id)}
                                className="group flex items-center justify-between p-6 bg-neutral-900/40 border border-neutral-800 hover:border-blue-500/50 hover:bg-neutral-900/80 rounded-lg cursor-pointer transition-all"
                            >
                                <div className="flex items-center gap-4">
                                    <div className="p-3 bg-neutral-950 border border-neutral-800 group-hover:border-blue-500/30 rounded">
                                        <svg className="w-6 h-6 text-neutral-500 group-hover:text-blue-500 transition-colors" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                                        </svg>
                                    </div>
                                    <div>
                                        <h3 className="text-lg font-bold text-neutral-200 group-hover:text-white">{project.name}</h3>
                                        <p className="text-sm text-neutral-600">
                                            {project.description || 'No description'}
                                        </p>
                                    </div>
                                </div>
                                <svg className="w-5 h-5 text-neutral-700 group-hover:text-blue-500 transition-colors" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                                </svg>
                            </div>
                        ))}
                    </div>
                )}
            </main>

            {/* Create Project Modal */}
            {showCreateModal && (
                <div
                    className="fixed inset-0 bg-black/80 flex items-center justify-center p-4 z-50"
                    onClick={() => setShowCreateModal(false)}
                >
                    <div
                        className="bg-neutral-950 border border-neutral-800 rounded-lg p-6 w-full max-w-md"
                        onClick={(e) => e.stopPropagation()}
                    >
                        <h3 className="text-xl font-bold mb-4">Create New Project</h3>
                        <form onSubmit={handleCreateProject} className="space-y-4">
                            <div>
                                <label className="block text-sm font-medium text-neutral-400 mb-1">
                                    Project Name
                                </label>
                                <input
                                    type="text"
                                    value={newProjectName}
                                    onChange={(e) => setNewProjectName(e.target.value)}
                                    className="w-full px-4 py-2 bg-neutral-900 border border-neutral-800 rounded text-white focus:outline-none focus:border-blue-500"
                                    required
                                    autoFocus
                                />
                            </div>
                            <div>
                                <label className="block text-sm font-medium text-neutral-400 mb-1">
                                    Description (optional)
                                </label>
                                <textarea
                                    value={newProjectDescription}
                                    onChange={(e) => setNewProjectDescription(e.target.value)}
                                    className="w-full px-4 py-2 bg-neutral-900 border border-neutral-800 rounded text-white focus:outline-none focus:border-blue-500 resize-none"
                                    rows={3}
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
