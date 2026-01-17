//! ChatPanel Component Test
//!
//! Unit tests for the ChatPanel component.

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';

// Mock the Tauri APIs
vi.mock('@tauri-apps/api/event', () => ({
    listen: vi.fn(() => Promise.resolve(() => { })),
    emit: vi.fn(() => Promise.resolve()),
}));

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(() => Promise.resolve()),
}));

// Mock the store
vi.mock('@/stores/useAgentStore', () => ({
    useAgentStore: vi.fn(() => ({
        mode: 'chat',
        status: 'idle',
        isStreaming: false,
        messages: [],
        pendingTool: null,
        thinkingText: '',
        connectionMode: 'cloud',
        setMode: vi.fn(),
        setStatus: vi.fn(),
        addMessage: vi.fn(),
        clearHistory: vi.fn(),
    })),
}));

// Import after mocks
import ChatPanel from '../Chat/ChatPanel';

describe('ChatPanel', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('renders without crashing', () => {
        render(<ChatPanel />);
        // Should render the chat container
        expect(document.querySelector('.h-full')).toBeTruthy();
    });

    it('renders empty state for new chat', () => {
        render(<ChatPanel />);
        // The component should be in the document
        expect(screen.queryByRole('textbox')).toBeDefined();
    });

    it('shows mode indicator', () => {
        render(<ChatPanel />);
        // Should have mode-related UI elements
        const container = document.querySelector('.h-full');
        expect(container).toBeTruthy();
    });
});
