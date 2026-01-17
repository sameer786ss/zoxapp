//! Editor State Slice
//! 
//! Multi-file editor state management.

import { StateCreator } from 'zustand';

export interface EditorFile {
    path: string;
    name: string;
    language: string;
    content?: string;
    isModified: boolean;
    source: 'user' | 'agent';
}

export interface EditorSlice {
    openFiles: EditorFile[];
    activeFileIndex: number;

    openFile: (path: string, source: 'user' | 'agent', content?: string) => void;
    closeFile: (index: number) => void;
    setActiveFile: (index: number) => void;
    updateFileContent: (index: number, content: string) => void;
    markFileSaved: (index: number) => void;
}

// Detect language from file extension
function detectLanguage(path: string): string {
    const ext = path.split('.').pop()?.toLowerCase() ?? '';
    const langMap: Record<string, string> = {
        'ts': 'typescript', 'tsx': 'typescript',
        'js': 'javascript', 'jsx': 'javascript',
        'py': 'python', 'rs': 'rust', 'go': 'go',
        'java': 'java', 'cpp': 'cpp', 'c': 'c',
        'html': 'html', 'css': 'css', 'scss': 'scss',
        'json': 'json', 'yaml': 'yaml', 'yml': 'yaml',
        'md': 'markdown', 'sql': 'sql', 'sh': 'shell',
    };
    return langMap[ext] ?? 'plaintext';
}

// Get filename from path
function getFileName(path: string): string {
    return path.split(/[/\\]/).pop() ?? path;
}

export const createEditorSlice: StateCreator<EditorSlice> = (set, get) => ({
    openFiles: [],
    activeFileIndex: -1,

    openFile: (path, source, content) => {
        const state = get();
        const existingIndex = state.openFiles.findIndex(f => f.path === path);

        if (existingIndex >= 0) {
            set({ activeFileIndex: existingIndex });
            return;
        }

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
            activeFileIndex: state.openFiles.length,
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
});
