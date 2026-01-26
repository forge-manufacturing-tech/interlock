import { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';

export function LoginPage() {
    const [isRegister, setIsRegister] = useState(false);
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [name, setName] = useState('');
    const [error, setError] = useState('');
    const [loading, setLoading] = useState(false);
    const { login, register, token } = useAuth();
    const navigate = useNavigate();

    // Navigate when token changes (successful login/register)
    useEffect(() => {
        if (token) {
            navigate('/');
        }
    }, [token, navigate]);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError('');
        setLoading(true);

        try {
            if (isRegister) {
                await register(email, password, name);
            } else {
                await login(email, password);
            }
            // Don't navigate here - the auth state change will trigger ProtectedRoute redirect
        } catch (err: any) {
            console.error('Auth error:', err);
            let errorMessage = 'Authentication failed';

            if (err.body) {
                errorMessage = err.body.error || err.body.message || JSON.stringify(err.body);
            } else if (err.message) {
                errorMessage = err.message;
            }

            setError(errorMessage);
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="min-h-screen bg-industrial-steel-950 flex items-center justify-center p-4 metal-texture">
            <div className="w-full max-w-md">
                <div className="industrial-panel p-8 rounded-sm shadow-2xl scanlines">
                    <div className="mb-8 text-center">
                        <h1 className="industrial-headline text-3xl mb-2">INTERLOCK</h1>
                        <p className="text-industrial-steel-400 text-sm tracking-wide font-mono">
                            {isRegister ? 'CREATE NEW ACCOUNT' : 'SYSTEM ACCESS'}
                        </p>
                    </div>

                    {error && (
                        <div className="mb-4 p-3 bg-industrial-alert/10 border border-industrial-alert rounded-sm text-industrial-alert text-sm font-mono">
                            {error}
                        </div>
                    )}

                    <form onSubmit={handleSubmit} className="space-y-4">
                        {isRegister && (
                            <div>
                                <label className="block text-xs font-bold text-industrial-steel-300 mb-2 uppercase tracking-wider">
                                    Operator Name
                                </label>
                                <input
                                    type="text"
                                    value={name}
                                    onChange={(e) => setName(e.target.value)}
                                    className="w-full px-4 py-2 industrial-input rounded-sm"
                                    required={isRegister}
                                    placeholder="Enter name"
                                />
                            </div>
                        )}

                        <div>
                            <label className="block text-xs font-bold text-industrial-steel-300 mb-2 uppercase tracking-wider">
                                Email Address
                            </label>
                            <input
                                type="email"
                                value={email}
                                onChange={(e) => setEmail(e.target.value)}
                                className="w-full px-4 py-2 industrial-input rounded-sm"
                                required
                                placeholder="operator@factory.com"
                            />
                        </div>

                        <div>
                            <label className="block text-xs font-bold text-industrial-steel-300 mb-2 uppercase tracking-wider">
                                Access Code
                            </label>
                            <input
                                type="password"
                                value={password}
                                onChange={(e) => setPassword(e.target.value)}
                                className="w-full px-4 py-2 industrial-input rounded-sm"
                                required
                                placeholder="••••••••"
                            />
                        </div>

                        <button
                            type="submit"
                            disabled={loading}
                            className="w-full py-3 px-4 industrial-btn rounded-sm disabled:opacity-50 disabled:cursor-not-allowed"
                        >
                            {loading ? 'PROCESSING...' : isRegister ? 'REGISTER' : 'LOGIN'}
                        </button>
                    </form>

                    <div className="mt-6 text-center">
                        <button
                            onClick={() => {
                                setIsRegister(!isRegister);
                                setError('');
                            }}
                            className="text-industrial-copper-500 hover:text-industrial-copper-400 text-xs transition-colors uppercase tracking-wide font-mono"
                        >
                            {isRegister ? 'Access Existing Account' : 'Create New Account'}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
}
