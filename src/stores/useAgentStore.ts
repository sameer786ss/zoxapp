import { create } from 'zustand';
import { subscribeWithSelector } from 'zustand/middleware';

export type MessageRole = 'user' | 'model' | 'tool';

export interface Message {
    id: string;
    role: MessageRole;
    content: string;
    timestamp: number;
}

export type AgentMode = 'chat' | 'turbo';
export type ConnectionMode = 'cloud' | 'offline';
export type SetupStatus = 'complete' | 'downloading_binaries' | 'downloading_model' | 'needs_setup' | null;

// GPU detection result from backend
export interface GpuInfo {
    type: 'nvidia' | 'amd' | 'intel' | 'cpu';
    name: string;
    vram_mb?: number;
}

// Download progress info
export type DownloadState = 'downloading' | 'paused' | 'resuming' | 'completed' | 'error';

export interface DownloadProgress {
    step: 'binaries' | 'model';
    percent: number;
    speed_mbps: number;
    eta_seconds: number;
    state: DownloadState;
}

// Editor file with metadata
export interface EditorFile {
    path: string;
    name: string;
    language: string;
    content?: string;
    isModified: boolean;
    source: 'user' | 'agent';
}

// Conversation metadata
export interface ConversationMeta {
    id: string;
    title: string;
    createdAt: string;
    updatedAt: string;
    messageCount: number;
    mode: string;
}

interface AgentState {
    // Mode and status
    mode: AgentMode;
    status: 'idle' | 'thinking' | 'executing' | 'awaiting_approval' | 'error';
    isStreaming: boolean;

    // Connection mode (cloud vs offline)
    connectionMode: ConnectionMode;
    isModelLoaded: boolean;
    modelLoadProgress: number | null; // 0-100 when loading/unloading, null otherwise
    setupStatus: SetupStatus;
    downloadProgress: DownloadProgress | null;
    detectedGpu: GpuInfo | null;

    // Messages
    messages: Message[];
    pendingTool: { name: string; args: string } | null;
    thinkingText: string; // Current thinking/reasoning text from model

    // Multi-file editor
    openFiles: EditorFile[];
    activeFileIndex: number;

    // Chat history
    conversations: ConversationMeta[];
    currentConversationId: string | null;

    // Actions
    setMode: (mode: AgentMode) => void;
    setStatus: (status: AgentState['status']) => void;
    setStreaming: (streaming: boolean) => void;

    // Connection mode actions
    setConnectionMode: (mode: ConnectionMode) => void;
    setModelLoaded: (loaded: boolean) => void;
    setModelLoadProgress: (progress: number | null) => void;
    setSetupStatus: (status: SetupStatus) => void;
    setDownloadProgress: (progress: DownloadProgress | null) => void;
    setDetectedGpu: (gpu: GpuInfo | null) => void;

    // Message actions
    addMessage: (msg: Omit<Message, 'id' | 'timestamp'>) => void;
    updateStreamingMessage: (content: string) => void;
    setPendingTool: (tool: { name: string; args: string } | null) => void;
    setThinkingText: (text: string) => void;
    clearHistory: () => void;

    // File actions
    openFile: (path: string, source: 'user' | 'agent', content?: string) => void;
    closeFile: (index: number) => void;
    setActiveFile: (index: number) => void;
    updateFileContent: (index: number, content: string) => void;
    markFileSaved: (index: number) => void;

    // History actions
    setConversations: (conversations: ConversationMeta[]) => void;
    setCurrentConversation: (id: string | null) => void;
}

// Detect language from file extension
function detectLanguage(path: string): string {
    const ext = path.split('.').pop()?.toLowerCase() || '';
    const langMap: Record<string, string> = {
        'ts': 'typescript', 'tsx': 'typescript',
        'js': 'javascript', 'jsx': 'javascript',
        'py': 'python', 'rs': 'rust', 'go': 'go',
        'java': 'java', 'cpp': 'cpp', 'c': 'c',
        'html': 'html', 'css': 'css', 'scss': 'scss',
        'json': 'json', 'yaml': 'yaml', 'yml': 'yaml',
        'md': 'markdown', 'sql': 'sql', 'sh': 'shell',
    };
    return langMap[ext] || 'plaintext';
}

// Get filename from path
function getFileName(path: string): string {
    return path.split(/[/\\]/).pop() || path;
}

export const useAgentStore = create<AgentState>()(
    subscribeWithSelector((set, get) => ({
        mode: 'chat',
        status: 'idle',
        isStreaming: false,
        connectionMode: 'cloud',
        isModelLoaded: false,
        modelLoadProgress: null,
        setupStatus: null,
        downloadProgress: null,
        detectedGpu: null,
        messages: [],
        pendingTool: null,
        thinkingText: '',
        openFiles: [],
        activeFileIndex: -1,
        conversations: [],
        currentConversationId: null,

        setMode: (mode) => set({ mode }),
        setStatus: (status) => set({ status, thinkingText: status === 'idle' ? '' : get().thinkingText }),
        setStreaming: (isStreaming) => set({ isStreaming }),
        setPendingTool: (pendingTool) => set({ pendingTool }),
        setThinkingText: (thinkingText) => set({ thinkingText }),

        // Connection mode actions
        setConnectionMode: (connectionMode) => set({ connectionMode }),
        setModelLoaded: (isModelLoaded) => set({ isModelLoaded }),
        setModelLoadProgress: (modelLoadProgress) => set({ modelLoadProgress }),
        setSetupStatus: (setupStatus) => set({ setupStatus }),
        setDownloadProgress: (downloadProgress) => set({ downloadProgress }),
        setDetectedGpu: (detectedGpu) => set({ detectedGpu }),

        addMessage: (msg) => set((state) => ({
            messages: [...state.messages, {
                ...msg,
                id: crypto.randomUUID(),
                timestamp: Date.now()
            }]
        })),

        updateStreamingMessage: (content: string) => {
            const state = get();
            const lastIdx = state.messages.length - 1;

            // If last message is model, update it; otherwise create new
            if (lastIdx >= 0 && state.messages[lastIdx].role === 'model') {
                const newMessages = [...state.messages];
                newMessages[lastIdx] = {
                    ...newMessages[lastIdx],
                    content  // Replace with full accumulated text
                };
                set({ messages: newMessages });
            } else {
                // Create new model message
                set({
                    messages: [...state.messages, {
                        role: 'model',
                        content,
                        id: crypto.randomUUID(),
                        timestamp: Date.now()
                    }]
                });
            }
        },

        clearHistory: () => set({ messages: [], currentConversationId: null }),

        // Multi-file support
        openFile: (path, source, content) => {
            const state = get();

            // Check if file is already open
            const existingIndex = state.openFiles.findIndex(f => f.path === path);
            if (existingIndex >= 0) {
                // Just switch to it
                set({ activeFileIndex: existingIndex });
                return;
            }

            // Add new file
            const newFile: EditorFile = {
                path,
                name: getFileName(path),
                language: detectLanguage(path),
                content,
                isModified: false,
                source,
            };

            set({
                openFiles: [...state.openFiles, newFile],
                activeFileIndex: state.openFiles.length, // New file becomes active
            });
        },

        closeFile: (index) => {
            const state = get();
            const newFiles = state.openFiles.filter((_, i) => i !== index);
            let newActiveIndex = state.activeFileIndex;

            if (index <= state.activeFileIndex) {
                newActiveIndex = Math.max(0, state.activeFileIndex - 1);
            }
            if (newFiles.length === 0) {
                newActiveIndex = -1;
            }

            set({ openFiles: newFiles, activeFileIndex: newActiveIndex });
        },

        setActiveFile: (index) => set({ activeFileIndex: index }),

        updateFileContent: (index, content) => {
            const state = get();
            set({
                openFiles: state.openFiles.map((f, i) =>
                    i === index ? { ...f, content, isModified: true } : f
                )
            });
        },

        markFileSaved: (index) => {
            const state = get();
            set({
                openFiles: state.openFiles.map((f, i) =>
                    i === index ? { ...f, isModified: false } : f
                )
            });
        },

        // History
        setConversations: (conversations) => set({ conversations }),
        setCurrentConversation: (id) => set({ currentConversationId: id }),
    }))
);

// Selectors
export const selectActiveFile = (state: AgentState) =>
    state.activeFileIndex >= 0 ? state.openFiles[state.activeFileIndex] : null;

export const selectIsModelStreaming = (state: AgentState) =>
    state.isStreaming && state.messages[state.messages.length - 1]?.role === 'model';
