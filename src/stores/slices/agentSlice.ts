//! Agent State Slice
//! 
//! Core agent mode and status state.

import { StateCreator } from 'zustand';

export type AgentMode = 'chat' | 'turbo';
export type AgentStatus = 'idle' | 'thinking' | 'executing' | 'awaiting_approval' | 'error';

export interface AgentSlice {
    mode: AgentMode;
    status: AgentStatus;
    isStreaming: boolean;
    thinkingText: string;
    tokenCount: number;

    setMode: (mode: AgentMode) => void;
    setStatus: (status: AgentStatus) => void;
    setStreaming: (streaming: boolean) => void;
    setThinkingText: (text: string) => void;
    setTokenCount: (count: number) => void;
}

export const createAgentSlice: StateCreator<AgentSlice> = (set, get) => ({
    mode: 'chat',
    status: 'idle',
    isStreaming: false,
    thinkingText: '',

    tokenCount: 0,

    setMode: (mode) => set({ mode }),
    setStatus: (status) => set({
        status,
        thinkingText: status === 'idle' ? '' : get().thinkingText,
        // Reset token count on idle
        tokenCount: status === 'idle' ? 0 : get().tokenCount
    }),
    setStreaming: (isStreaming) => set({ isStreaming }),
    setThinkingText: (thinkingText) => set({ thinkingText }),
    setTokenCount: (tokenCount) => set({ tokenCount }),
});
