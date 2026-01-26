import React, { useState, useEffect, useRef } from 'react';
import { ControllersChatService, MessageResponse, BlobResponse } from '../api/generated';

interface ChatInterfaceProps {
    sessionId: string;
    blobs: BlobResponse[];
    onRefreshBlobs?: () => void;
}

export function ChatInterface({ sessionId, blobs, onRefreshBlobs }: ChatInterfaceProps) {
    const [messages, setMessages] = useState<MessageResponse[]>([]);
    const [input, setInput] = useState('');
    const [loading, setLoading] = useState(false);
    const messagesEndRef = useRef<HTMLDivElement>(null);
    const scrollContainerRef = useRef<HTMLDivElement>(null);

    const scrollToBottom = () => {
        if (scrollContainerRef.current) {
            scrollContainerRef.current.scrollTop = scrollContainerRef.current.scrollHeight;
        }
    };

    useEffect(() => {
        loadMessages();
    }, [sessionId]);

    useEffect(() => {
        scrollToBottom();
    }, [messages, loading]);

    const loadMessages = async () => {
        try {
            const data = await ControllersChatService.listMessages(sessionId);
            setMessages(data);
        } catch (error) {
            console.error('Failed to load messages:', error);
        }
    };

    const handleSendMessage = async (e: React.FormEvent) => {
        e.preventDefault();
        if (!input.trim() || loading) return;

        const userMessage = input.trim();
        setInput('');
        setLoading(true);

        const tempMsg: MessageResponse = {
            id: crypto.randomUUID(),
            session_id: sessionId,
            role: 'user',
            content: userMessage,
            created_at: new Date().toISOString()
        };
        setMessages(prev => [...prev, tempMsg]);

        try {
            await ControllersChatService.chat(sessionId, { message: userMessage });
            loadMessages();
            onRefreshBlobs?.();
        } catch (error) {
            console.error('Chat error:', error);
            alert('Failed to send message');
        } finally {
            setLoading(false);
        }
    };

    const handleClearChat = async () => {
        if (!confirm('Clear all messages in this session?')) return;
        try {
            await ControllersChatService.clearMessages(sessionId);
            setMessages([]);
        } catch (error) {
            console.error('Failed to clear messages:', error);
            alert('Failed to clear messages');
        }
    };

    return (
        <div className="flex flex-col h-full industrial-panel rounded-sm overflow-hidden border-industrial-copper-500/20 shadow-glow-copper/5">
            {/* Chat Header / Context */}
            <div className="px-4 py-2 bg-industrial-steel-900/80 border-b border-industrial-concrete flex items-center justify-between">
                <div className="flex items-center gap-2">
                    <div className="w-2 h-2 bg-industrial-copper-500 rounded-full animate-pulse shadow-glow-copper" />
                    <span className="text-[10px] font-bold uppercase tracking-widest text-industrial-steel-300 font-mono">Secure-AI-Link</span>
                </div>
                <div className="flex items-center gap-4">
                    <div className="flex gap-2">
                        {blobs.slice(0, 2).map(b => (
                            <div key={b.id} className="px-2 py-0.5 bg-industrial-steel-800 rounded-full border border-industrial-concrete text-[8px] text-industrial-steel-400 font-mono truncate max-w-[80px]">
                                {b.file_name}
                            </div>
                        ))}
                    </div>
                    <button
                        onClick={handleClearChat}
                        className="text-[9px] font-mono text-industrial-steel-500 hover:text-industrial-alert uppercase tracking-widest transition-colors"
                    >
                        [Reset]
                    </button>
                </div>
            </div>

            <div
                ref={scrollContainerRef}
                className="flex-1 overflow-y-auto p-4 space-y-4 scanlines bg-industrial-steel-950/50 custom-scrollbar"
                style={{ scrollBehavior: 'smooth' }}
            >
                {messages.length === 0 && (
                    <div className="flex flex-col items-center justify-center h-full opacity-30 select-none">
                        <div className="industrial-headline text-4xl mb-2">INTERLOCK</div>
                        <div className="text-[10px] font-mono uppercase tracking-[0.3em]">Ready for secure data processing</div>
                    </div>
                )}
                {messages.map((msg) => (
                    <div key={msg.id} className={`flex ${msg.role === 'user' ? 'justify-end' : 'justify-start'}`}>
                        <div className={`max-w-[85%] p-4 rounded-sm border ${msg.role === 'user'
                            ? 'bg-industrial-copper-500/5 border-industrial-copper-500/30 text-neutral-100 shadow-glow-copper/5'
                            : 'bg-industrial-steel-900/90 border-industrial-concrete text-neutral-200'
                            }`}>
                            <div className="flex items-center justify-between mb-2">
                                <div className="text-[9px] font-black uppercase tracking-widest opacity-60 font-mono">
                                    {msg.role === 'user' ? 'OPERATOR' : 'SECURE_AI'}
                                </div>
                                <div className="text-[8px] font-mono opacity-30">
                                    {new Date(msg.created_at).toLocaleTimeString()}
                                </div>
                            </div>
                            <div className="text-sm whitespace-pre-wrap font-mono leading-relaxed tracking-tight">
                                {msg.content}
                            </div>
                        </div>
                    </div>
                ))}
                {loading && (
                    <div className="flex justify-start">
                        <div className="bg-industrial-steel-900/90 border border-industrial-concrete p-4 rounded-sm">
                            <div className="flex gap-1.5">
                                <div className="w-1.5 h-1.5 bg-industrial-copper-500 rounded-full animate-bounce" style={{ animationDelay: '0s' }} />
                                <div className="w-1.5 h-1.5 bg-industrial-copper-500 rounded-full animate-bounce" style={{ animationDelay: '0.2s' }} />
                                <div className="w-1.5 h-1.5 bg-industrial-copper-500 rounded-full animate-bounce" style={{ animationDelay: '0.4s' }} />
                            </div>
                        </div>
                    </div>
                )}
                <div ref={messagesEndRef} className="h-2" />
            </div>

            <form onSubmit={handleSendMessage} className="p-4 border-t border-industrial-concrete bg-industrial-steel-900/90">
                <div className="flex gap-2">
                    <input
                        type="text"
                        value={input}
                        onChange={(e) => setInput(e.target.value)}
                        placeholder="Type a command (e.g., 'Summarize test.xlsx')..."
                        className="flex-1 industrial-input rounded-sm px-4 py-3 font-mono text-sm focus:ring-1 focus:ring-industrial-copper-500/30"
                        disabled={loading}
                    />
                    <button
                        type="submit"
                        disabled={loading || !input.trim()}
                        className={`industrial-btn px-8 rounded-sm text-xs font-black tracking-widest transition-all ${loading ? 'opacity-50 grayscale' : 'hover:scale-[1.02] active:scale-95'}`}
                    >
                        {loading ? 'BUSY' : 'SEND'}
                    </button>
                </div>
                <div className="mt-3 flex justify-between items-center px-1">
                    <div className="flex gap-4">
                        <span className="text-[8px] font-mono text-industrial-steel-500 uppercase tracking-widest">Status: encrypted</span>
                        <span className="text-[8px] font-mono text-industrial-steel-500 uppercase tracking-widest text-industrial-copper-500/50">Gemini 2.0 Flash Lite</span>
                    </div>
                    <span className="text-[8px] font-mono text-industrial-steel-600 uppercase tracking-widest">v1.2.4-stable</span>
                </div>
            </form>
        </div>
    );
}
