import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { ControllersAuthService } from '../api/generated';
import { OpenAPI } from '../api/generated/core/OpenAPI';

interface User {
    email: string;
    name: string;
}

interface AuthContextType {
    user: User | null;
    token: string | null;
    login: (email: string, password: string) => Promise<void>;
    register: (email: string, password: string, name: string) => Promise<void>;
    logout: () => void;
    isLoading: boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
    const [user, setUser] = useState<User | null>(() => {
        // Initialize from localStorage if exists
        const token = localStorage.getItem('token');
        return token ? { email: '', name: '' } : null;  // Placeholder until we have /current endpoint
    });
    const [token, setToken] = useState<string | null>(localStorage.getItem('token'));
    const [isLoading, setIsLoading] = useState(false);  // Changed to false since we don't fetch on mount

    useEffect(() => {
        if (token) {
            OpenAPI.TOKEN = token;
        } else {
            OpenAPI.TOKEN = undefined;
        }
    }, [token]);

    const login = async (email: string, password: string) => {
        const response = await ControllersAuthService.login({ email, password });
        const newToken = response.token;
        const newUser = { email, name: response.name };

        localStorage.setItem('token', newToken);
        OpenAPI.TOKEN = newToken;

        // Update both states together
        setUser(newUser);
        setToken(newToken);
    };

    const register = async (email: string, password: string, name: string) => {
        const response = await ControllersAuthService.register({ email, password, name });
        const newToken = response.token;
        const newUser = { email, name: response.name };

        localStorage.setItem('token', newToken);
        OpenAPI.TOKEN = newToken;

        // Update both states together
        setUser(newUser);
        setToken(newToken);
    };

    const logout = () => {
        localStorage.removeItem('token');
        setToken(null);
        setUser(null);
        OpenAPI.TOKEN = undefined;
    };

    return (
        <AuthContext.Provider value={{ user, token, login, register, logout, isLoading }}>
            {children}
        </AuthContext.Provider>
    );
}

export function useAuth() {
    const context = useContext(AuthContext);
    if (context === undefined) {
        throw new Error('useAuth must be used within an AuthProvider');
    }
    return context;
}
