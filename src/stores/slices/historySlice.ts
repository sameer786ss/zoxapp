//! History State Slice
//! 
//! Conversation history management.

import { StateCreator } from 'zustand';

export interface ConversationMeta {
    id: string;
    title: string;
    createdAt: string;
    updatedAt: string;
    messageCount: number;
    mode: string;
}

export interface HistorySlice {
    conversations: ConversationMeta[];
    currentConversationId: string | null;

    setConversations: (conversations: ConversationMeta[]) => void;
    setCurrentConversation: (id: string | null) => void;
}

export const createHistorySlice: StateCreator<HistorySlice> = (set) => ({
    conversations: [],
    currentConversationId: null,

    setConversations: (conversations) => set({ conversations }),
    setCurrentConversation: (currentConversationId) => set({ currentConversationId }),
});
