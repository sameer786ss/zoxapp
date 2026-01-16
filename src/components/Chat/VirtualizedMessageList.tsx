import { useRef, useEffect, useCallback, useState, useLayoutEffect } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import MessageBubble from './MessageBubble';
import { useAgentStore, Message } from '@/stores/useAgentStore';
import { cn } from '@/lib/utils';
import { ChevronDown } from 'lucide-react';
import { Button } from '@/components/ui/button';

interface VirtualizedMessageListProps {
    messages: Message[];
}

export default function VirtualizedMessageList({ messages }: VirtualizedMessageListProps) {
    const scrollContainerRef = useRef<HTMLDivElement>(null);

    const [showScrollButton, setShowScrollButton] = useState(false);
    const prevMessagesLengthRef = useRef(messages.length);
    const prevLastContentRef = useRef('');

    // Get streaming state from store
    const isStreaming = useAgentStore((state) => state.isStreaming);



    // Track if we should auto-scroll
    const shouldAutoScrollRef = useRef(true);

    // Handle scroll events to detect user intent
    const handleScroll = useCallback(() => {
        const container = scrollContainerRef.current;
        if (!container) return;

        const { scrollTop, scrollHeight, clientHeight } = container;
        const distanceFromBottom = scrollHeight - scrollTop - clientHeight;

        // If user scrolls up significantly (>50px), disable auto-scroll
        const isAtBottomNow = distanceFromBottom < 50;

        setShowScrollButton(!isAtBottomNow);

        // Update auto-scroll preference:
        // - If at bottom, re-enable auto-scroll
        // - If scrolled up, disable it (user wants to read history)
        if (isAtBottomNow) {
            shouldAutoScrollRef.current = true;
        } else if (distanceFromBottom > 100) {
            shouldAutoScrollRef.current = false;
        }
    }, []);

    // Scroll to bottom helper
    const scrollToBottom = useCallback((instant = false) => {
        const container = scrollContainerRef.current;
        if (!container) return;

        if (instant) {
            container.scrollTop = container.scrollHeight;
        } else {
            container.scrollTo({
                top: container.scrollHeight,
                behavior: 'smooth'
            });
        }
        shouldAutoScrollRef.current = true; // enhancing: manual scroll-to-bottom re-enables lock
    }, []);

    // Auto-scroll when new message is added
    useEffect(() => {
        if (messages.length > prevMessagesLengthRef.current) {
            // New message added - ALWAYS snap to bottom for new messages
            shouldAutoScrollRef.current = true;
            scrollToBottom(false);
        }
        prevMessagesLengthRef.current = messages.length;
    }, [messages.length, scrollToBottom]);

    // Auto-scroll during streaming (content updates)
    useLayoutEffect(() => {
        if (!isStreaming) return;

        const lastMessage = messages[messages.length - 1];
        if (!lastMessage) return;

        const lastContent = typeof lastMessage.content === 'string'
            ? lastMessage.content
            : '';

        const contentChanged = lastContent.length !== prevLastContentRef.current.length;

        // Only scroll if content changed AND we are in "stick to bottom" mode
        if (contentChanged && shouldAutoScrollRef.current) {
            requestAnimationFrame(() => {
                const container = scrollContainerRef.current;
                if (container) {
                    container.scrollTop = container.scrollHeight;
                }
            });
        }

        prevLastContentRef.current = lastContent;
    }, [messages, isStreaming]);

    // Initial scroll to bottom
    useEffect(() => {
        scrollToBottom(true);
    }, []);

    // Simple list for few messages (no virtualization needed)
    if (messages.length < 20) {
        return (
            <div className="relative h-full">
                <div
                    ref={scrollContainerRef}
                    onScroll={handleScroll}
                    className="h-full overflow-y-auto scroll-smooth"
                >
                    <div className="space-y-1 p-2">
                        {messages.map((msg) => (
                            <MessageBubble key={msg.id} message={msg} />
                        ))}
                    </div>
                </div>

                {/* Scroll to bottom button */}
                {showScrollButton && (
                    <ScrollToBottomButton onClick={() => {

                        setShowScrollButton(false);
                        scrollToBottom(true);
                    }} />
                )}
            </div>
        );
    }

    // Virtualized list for many messages
    return (
        <VirtualizedList
            messages={messages}
            scrollContainerRef={scrollContainerRef}
            handleScroll={handleScroll}
            showScrollButton={showScrollButton}
            onScrollToBottom={() => {

                setShowScrollButton(false);
                scrollToBottom(true);
            }}
        />
    );
}

// Separated virtualized list component
interface VirtualizedListProps {
    messages: Message[];
    scrollContainerRef: React.RefObject<HTMLDivElement | null>;
    handleScroll: () => void;
    showScrollButton: boolean;
    onScrollToBottom: () => void;
}

function VirtualizedList({
    messages,
    scrollContainerRef,
    handleScroll,
    showScrollButton,
    onScrollToBottom
}: VirtualizedListProps) {

    const estimateSize = useCallback((index: number) => {
        const msg = messages[index];
        if (!msg) return 100;

        const content = typeof msg.content === 'string' ? msg.content : '';
        const length = content.length;
        const hasCode = content.includes('```');

        if (hasCode) {
            return Math.max(200, Math.min(600, 100 + length * 0.15));
        }
        return Math.max(80, Math.min(400, 60 + length * 0.12));
    }, [messages]);

    const virtualizer = useVirtualizer({
        count: messages.length,
        getScrollElement: () => scrollContainerRef.current,
        estimateSize,
        overscan: 5,
        getItemKey: (index) => messages[index]?.id || String(index),
    });

    // Scroll to end when virtualizer updates
    // Removed to prevent conflict with parent's auto-scroll logic
    // The parent manages scroll via scrollContainerRef directly
    /*
    useEffect(() => {
        if (messages.length > 0) {
            virtualizer.scrollToIndex(messages.length - 1, { align: 'end' });
        }
    }, [messages.length]);
    */

    return (
        <div className="relative h-full">
            <div
                ref={scrollContainerRef}
                onScroll={handleScroll}
                className="h-full overflow-y-auto"
            >
                <div
                    style={{
                        height: `${virtualizer.getTotalSize()}px`,
                        width: '100%',
                        position: 'relative',
                    }}
                >
                    {virtualizer.getVirtualItems().map((virtualItem) => (
                        <div
                            key={virtualItem.key}
                            data-index={virtualItem.index}
                            ref={virtualizer.measureElement}
                            style={{
                                position: 'absolute',
                                top: 0,
                                left: 0,
                                width: '100%',
                                transform: `translateY(${virtualItem.start}px)`,
                            }}
                        >
                            <MessageBubble message={messages[virtualItem.index]} />
                        </div>
                    ))}
                </div>
            </div>

            {showScrollButton && (
                <ScrollToBottomButton onClick={onScrollToBottom} />
            )}
        </div>
    );
}

// Scroll to bottom button component
function ScrollToBottomButton({ onClick }: { onClick: () => void }) {
    return (
        <Button
            size="sm"
            variant="secondary"
            className={cn(
                "absolute bottom-4 right-4 rounded-full shadow-lg z-10",
                "bg-primary text-white hover:bg-primary/90",
                "animate-fade-in"
            )}
            onClick={onClick}
        >
            <ChevronDown className="w-4 h-4" />
        </Button>
    );
}
