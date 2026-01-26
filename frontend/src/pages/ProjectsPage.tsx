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
            <div className="min-h-screen bg-industrial-steel-950 flex items-center justify-center">
                <div className="text-industrial-steel-400 font-mono uppercase tracking-wider">Loading...</div>
            </div>
        );
    }

    return (
        <div className="min-h-screen bg-industrial-steel-950 text-neutral-100 metal-texture">
            {/* Header */}
            <header className="border-b border-industrial-concrete bg-industrial-steel-900/80 backdrop-blur-sm">
                <div className="max-w-7xl mx-auto px-6 py-4 flex items-center justify-between">
                    <h1 className="industrial-headline text-2xl">INTERLOCK</h1>
                    <div className="flex items-center gap-4">
                        <span className="text-sm text-industrial-steel-400 font-mono">{user?.email}</span>
                        <button
                            onClick={logout}
                            className="px-4 py-2 text-xs bg-industrial-steel-800 hover:bg-industrial-steel-700 border border-industrial-concrete rounded-sm transition-colors uppercase tracking-wide font-mono"
                        >
                            Logout
                        </button>
                    </div>
                </div>
            </header>

            {/* Main Content */}
            <main className="max-w-7xl mx-auto px-6 py-8 scanlines">
                <div className="flex items-center justify-between mb-8">
                    <div>
                        <h2 className="industrial-headline text-2xl mb-2">PROJECTS</h2>
                        <p className="text-industrial-steel-400 text-sm font-mono">Manage your workspaces</p>
                    </div>
                    <button
                        onClick={() => setShowCreateModal(true)}
                        className="px-6 py-2 industrial-btn rounded-sm font-medium"
                    >
                        + New Project
                    </button>
                </div>

                {/* Projects Grid */}
                {projects.length === 0 ? (
                    <div className="p-16 border border-dashed border-industrial-concrete bg-industrial-steel-900/20 text-center rounded-sm">
                        <p className="text-industrial-steel-500 text-sm uppercase tracking-widest font-bold">No projects found</p>
                        <p className="text-industrial-steel-600 text-xs mt-2 font-mono">Create your first project to get started</p>
                    </div>
                ) : (
                    <div className="grid gap-4">
                        {projects.map((project) => (
                            <div
                                key={project.id}
                                onClick={() => openProject(project.id)}
                                className="industrial-card group flex items-center justify-between p-6 rounded-sm cursor-pointer"
                            >
                                <div className="flex items-center gap-4">
                                    <div className="p-3 bg-industrial-steel-950 border border-industrial-concrete group-hover:border-industrial-copper-500/50 rounded-sm transition-colors">
                                        <svg className="w-6 h-6 text-industrial-steel-500 group-hover:text-industrial-copper-500 transition-colors" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                                        </svg>
                                    </div>
                                    <div>
                                        <h3 className="text-lg font-bold text-neutral-200 group-hover:text-industrial-copper-400 transition-colors tracking-wide">{project.name}</h3>
                                        <p className="text-sm text-industrial-steel-500 font-mono">
                                            {project.description || 'No description'}
                                        </p>
                                    </div>
                                </div>
                                <svg className="w-5 h-5 text-industrial-steel-600 group-hover:text-industrial-copper-500 transition-colors" fill="none" viewBox="0 0 24 24" stroke="currentColor">
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
                    className="fixed inset-0 bg-black/90 flex items-center justify-center p-4 z-50 backdrop-blur-sm"
                    onClick={() => setShowCreateModal(false)}
                >
                    <div
                        className="industrial-panel rounded-sm p-6 w-full max-w-md"
                        onClick={(e) => e.stopPropagation()}
                    >
                        <h3 className="industrial-headline text-xl mb-4">Create New Project</h3>
                        <form onSubmit={handleCreateProject} className="space-y-4">
                            <div>
                                <label className="block text-xs font-bold text-industrial-steel-300 mb-2 uppercase tracking-wider">
                                    Project Name
                                </label>
                                <input
                                    type="text"
                                    value={newProjectName}
                                    onChange={(e) => setNewProjectName(e.target.value)}
                                    className="w-full px-4 py-2 industrial-input rounded-sm"
                                    required
                                    autoFocus
                                    placeholder="Enter project name"
                                />
                            </div>
                            <div>
                                <label className="block text-xs font-bold text-industrial-steel-300 mb-2 uppercase tracking-wider">
                                    Description (optional)
                                </label>
                                <textarea
                                    value={newProjectDescription}
                                    onChange={(e) => setNewProjectDescription(e.target.value)}
                                    className="w-full px-4 py-2 industrial-input rounded-sm resize-none"
                                    rows={3}
                                    placeholder="Enter description"
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
