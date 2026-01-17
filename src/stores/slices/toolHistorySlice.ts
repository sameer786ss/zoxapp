//! Tool History Slice
//! 
//! Tracks all tool executions for debugging and auditing.

import { StateCreator } from 'zustand';

export interface ToolExecution {
    id: string;
    tool: string;
    args: string;
    result: string;
    approved: boolean;
    timestamp: number;
    durationMs?: number;
}

export interface ToolHistorySlice {
    toolExecutions: ToolExecution[];

    addToolExecution: (execution: Omit<ToolExecution, 'id' | 'timestamp'>) => void;
    clearToolHistory: () => void;
    getRecentExecutions: (count?: number) => ToolExecution[];
}

const MAX_TOOL_HISTORY = 100;

export const createToolHistorySlice: StateCreator<ToolHistorySlice> = (set, get) => ({
    toolExecutions: [],

    addToolExecution: (execution) => set((state) => {
        const newExecution: ToolExecution = {
            ...execution,
            id: crypto.randomUUID(),
            timestamp: Date.now(),
        };

        const executions = [newExecution, ...state.toolExecutions].slice(0, MAX_TOOL_HISTORY);
        return { toolExecutions: executions };
    }),

    clearToolHistory: () => set({ toolExecutions: [] }),

    getRecentExecutions: (count = 10) => {
        return get().toolExecutions.slice(0, count);
    },
});
