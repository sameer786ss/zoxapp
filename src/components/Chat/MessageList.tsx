import { Message } from '@/stores/useAgentStore';
import MessageBubble from './MessageBubble';

interface MessageListProps {
    messages: Message[];
}

/**
 * Simple message list component - for non-virtualized use cases
 * For large message lists, use VirtualizedMessageList instead
 */
export default function MessageList({ messages }: MessageListProps) {
    if (messages.length === 0) {
        return null;
    }

    return (
        <div className="flex flex-col gap-2 p-4">
            {messages.map((message) => (
                <MessageBubble key={message.id} message={message} />
            ))}
        </div>
    );
}
