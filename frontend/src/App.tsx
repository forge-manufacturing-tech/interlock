import React from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { AuthProvider, useAuth } from './contexts/AuthContext';
import { LoginPage } from './pages/LoginPage';
import { ProjectsPage } from './pages/ProjectsPage';
import { SessionsPage } from './pages/SessionsPage';

function ProtectedRoute({ children }: { children: React.ReactNode }) {
    const { token, isLoading } = useAuth();

    if (isLoading) {
        return (
            <div className="min-h-screen bg-industrial-steel-950 flex items-center justify-center metal-texture">
                <div className="text-industrial-steel-400 font-mono uppercase tracking-wider">Loading...</div>
            </div>
        );
    }

    return token ? <>{children}</> : <Navigate to="/login" />;
}

function PublicRoute({ children }: { children: React.ReactNode }) {
    const { isLoading } = useAuth();

    if (isLoading) {
        return (
            <div className="min-h-screen bg-industrial-steel-950 flex items-center justify-center metal-texture">
                <div className="text-industrial-steel-400 font-mono uppercase tracking-wider">Loading...</div>
            </div>
        );
    }

    // Don't redirect here - let the LoginPage handle navigation via useEffect
    return <>{children}</>;
}

function AppRoutes() {
    return (
        <Routes>
            <Route
                path="/login"
                element={
                    <PublicRoute>
                        <LoginPage />
                    </PublicRoute>
                }
            />
            <Route
                path="/"
                element={
                    <ProtectedRoute>
                        <ProjectsPage />
                    </ProtectedRoute>
                }
            />
            <Route
                path="/projects/:projectId"
                element={
                    <ProtectedRoute>
                        <SessionsPage />
                    </ProtectedRoute>
                }
            />
        </Routes>
    );
}

export function App() {
    return (
        <BrowserRouter>
            <AuthProvider>
                <AppRoutes />
            </AuthProvider>
        </BrowserRouter>
    );
}
