import { useRef, useCallback, useState, useLayoutEffect } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import MessageBubble from './MessageBubble';
import { Message } from '@/stores/useAgentStore';
import { cn } from '@/lib/utils';
import { ChevronDown } from 'lucide-react';
import { Button } from '@/components/ui/button';

interface VirtualizedMessageListProps {
    messages: Message[];
}

export default function VirtualizedMessageList({ messages }: VirtualizedMessageListProps) {
    if (messages.length === 0) return null;

    // Use a unified sticky-scroll implementation that handles both cases
    // But since the user complained about "tool called", we likely have many messages or complex content
    // We will stick to the virtualized implementation for robustness, or simple for small lists.

    // Actually, simplifying to ONE implementation (Virtualized) is often better for consistency unless there's a huge perf penalty.
    // But let's keep the split if the simple one works well. 
    // The user's issue is likely with the Virtualized one behaving badly.

    return (
        <div className="h-full w-full relative">
            {messages.length < 20 ? (
                <SimpleList messages={messages} />
            ) : (
                <VirtList messages={messages} />
            )}
        </div>
    );
}

// --- Simple List (DOM-based) ---
function SimpleList({ messages }: { messages: Message[] }) {
    const scrollRef = useRef<HTMLDivElement>(null);
    const [showButton, setShowButton] = useState(false);
    const isSticky = useRef(true); // Default to sticky

    const scrollToBottom = () => {
        if (scrollRef.current) {
            scrollRef.current.scrollTo({ top: scrollRef.current.scrollHeight, behavior: 'auto' }); // Instant for responsiveness
        }
    };

    const handleScroll = () => {
        if (!scrollRef.current) return;
        const { scrollTop, scrollHeight, clientHeight } = scrollRef.current;
        const dist = scrollHeight - scrollTop - clientHeight;

        // Tolerance of 50px
        const atBottom = dist < 50;
        isSticky.current = atBottom;
        setShowButton(!atBottom);
    };

    // Auto-scroll on new messages or content updates
    useLayoutEffect(() => {
        if (isSticky.current) {
            scrollToBottom();
        }
    }, [messages, messages.length, messages[messages.length - 1]?.content]);

    return (
        <>
            <div
                ref={scrollRef}
                className="h-full overflow-y-auto scroll-smooth p-4 space-y-4"
                onScroll={handleScroll}
            >
                {messages.map((msg) => (
                    <MessageBubble key={msg.id} message={msg} />
                ))}
            </div>
            {showButton && (
                <ScrollToBottomButton onClick={() => {
                    scrollToBottom();
                    isSticky.current = true;
                }} />
            )}
        </>
    );
}

// --- Virtualized List (TanStack Virtual) ---
function VirtList({ messages }: { messages: Message[] }) {
    const parentRef = useRef<HTMLDivElement>(null);
    const isSticky = useRef(true);
    const [showButton, setShowButton] = useState(false);

    // Estimate size logic
    const estimateSize = useCallback((index: number) => {
        const msg = messages[index];
        if (!msg) return 100;
        const content = typeof msg.content === 'string' ? msg.content : '';
        // Rough estimation
        return 80 + (content.length * 0.5);
    }, [messages]);

    const virtualizer = useVirtualizer({
        count: messages.length,
        getScrollElement: () => parentRef.current,
        estimateSize,
        overscan: 10,
        getItemKey: (index) => messages[index]?.id || String(index),
    });

    const scrollToIndex = useCallback((index: number) => {
        try {
            virtualizer.scrollToIndex(index, { align: 'end' });
        } catch (e) {
            // Ignore virtualizer errors during initial render
        }
    }, [virtualizer]);

    // Scroll handler
    const handleScroll = () => {
        if (!parentRef.current) return;
        const { scrollTop, scrollHeight, clientHeight } = parentRef.current;
        const dist = scrollHeight - scrollTop - clientHeight;

        // If dist < 50, user is at bottom -> Sticky = true
        // If dist > 50, user scrolled up -> Sticky = false
        const atBottom = dist < 100; // slightly larger threshold for virtualization
        isSticky.current = atBottom;
        setShowButton(!atBottom);
    };

    // Effect: Scroll to bottom when messages change IF sticky
    useLayoutEffect(() => {
        if (isSticky.current && messages.length > 0) {
            // Use requestAnimationFrame to ensure virtualizer has measured new items
            requestAnimationFrame(() => {
                scrollToIndex(messages.length - 1);
            });
        }
    }, [messages.length, messages[messages.length - 1]?.content, scrollToIndex]);

    return (
        <div className="h-full relative">
            <div
                ref={parentRef}
                onScroll={handleScroll}
                className="h-full overflow-y-auto w-full"
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
                                paddingLeft: '1rem',
                                paddingRight: '1rem',
                                paddingTop: '0.5rem',
                            }}
                        >
                            <MessageBubble message={messages[virtualItem.index]} />
                        </div>
                    ))}
                </div>
            </div>

            {showButton && (
                <ScrollToBottomButton onClick={() => {
                    scrollToIndex(messages.length - 1);
                    isSticky.current = true;
                }} />
            )}
        </div>
    );
}

function ScrollToBottomButton({ onClick }: { onClick: () => void }) {
    return (
        <Button
            size="icon"
            variant="secondary"
            className={cn(
                "absolute bottom-6 right-6 rounded-full shadow-xl z-50",
                "bg-primary text-primary-foreground hover:bg-primary/90",
                "h-10 w-10 animate-in fade-in zoom-in duration-200"
            )}
            onClick={onClick}
        >
            <ChevronDown className="w-5 h-5" />
        </Button>
    );
}
