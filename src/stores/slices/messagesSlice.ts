//! Messages State Slice
//! 
//! Chat messages and tool approval state.

import { StateCreator } from 'zustand';

export type MessageRole = 'user' | 'model' | 'tool';

export interface Message {
    id: string;
    role: MessageRole;
    content: string;
    timestamp: number;
}

export interface PendingTool {
    name: string;
    args: string;
}

export interface MessagesSlice {
    messages: Message[];
    pendingTool: PendingTool | null;

    addMessage: (msg: Omit<Message, 'id' | 'timestamp'>) => void;
    updateStreamingMessage: (content: string) => void;
    setPendingTool: (tool: PendingTool | null) => void;
    clearHistory: () => void;
}

export const createMessagesSlice: StateCreator<MessagesSlice> = (set, get) => ({
    messages: [],
    pendingTool: null,

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

        if (lastIdx >= 0 && state.messages[lastIdx].role === 'model') {
            const newMessages = [...state.messages];
            newMessages[lastIdx] = {
                ...newMessages[lastIdx],
                content
            };
            set({ messages: newMessages });
        } else {
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

    setPendingTool: (pendingTool) => set({ pendingTool }),
    clearHistory: () => set({ messages: [], pendingTool: null }),
});
