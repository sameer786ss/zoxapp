import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneDark } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { Copy, Check } from 'lucide-react';
import { useState, useCallback, memo } from 'react';
import { toast } from '@/components/ui/sonner';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

interface MarkdownRendererProps {
    content: string;
    className?: string;
}

export const MarkdownRenderer = memo(function MarkdownRenderer({ content, className = '' }: MarkdownRendererProps) {
    return (
        <div className={cn("markdown-body prose prose-invert prose-sm max-w-none", className)}>
            <ReactMarkdown
                remarkPlugins={[remarkGfm]}
                components={{
                    // Code blocks with syntax highlighting
                    code({ node, inline, className, children, ...props }: any) {
                        const match = /language-(\w+)/.exec(className || '');
                        const language = match ? match[1] : '';
                        const codeString = String(children).replace(/\n$/, '');

                        if (!inline && language) {
                            return (
                                <CodeBlock
                                    code={codeString}
                                    language={language}
                                />
                            );
                        }

                        // Inline code
                        return (
                            <code
                                className="px-1.5 py-0.5 bg-muted rounded text-pink-400 text-sm font-mono"
                                {...props}
                            >
                                {children}
                            </code>
                        );
                    },

                    // Links
                    a({ href, children }) {
                        return (
                            <a
                                href={href}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="text-primary hover:text-primary/80 underline decoration-primary/30 hover:decoration-primary/50"
                            >
                                {children}
                            </a>
                        );
                    },

                    // Tables
                    table({ children }) {
                        return (
                            <div className="overflow-x-auto my-4">
                                <table className="min-w-full border border-border rounded-md overflow-hidden">
                                    {children}
                                </table>
                            </div>
                        );
                    },
                    th({ children }) {
                        return (
                            <th className="px-3 py-2 bg-muted text-left text-xs font-semibold text-foreground border-b border-border">
                                {children}
                            </th>
                        );
                    },
                    td({ children }) {
                        return (
                            <td className="px-3 py-2 text-sm text-muted-foreground border-b border-border/50">
                                {children}
                            </td>
                        );
                    },

                    // Blockquotes
                    blockquote({ children }) {
                        return (
                            <blockquote className="border-l-2 border-primary pl-4 my-4 text-muted-foreground italic">
                                {children}
                            </blockquote>
                        );
                    },

                    // Lists
                    ul({ children }) {
                        return (
                            <ul className="list-disc list-inside space-y-1 my-2 text-foreground">
                                {children}
                            </ul>
                        );
                    },
                    ol({ children }) {
                        return (
                            <ol className="list-decimal list-inside space-y-1 my-2 text-foreground">
                                {children}
                            </ol>
                        );
                    },

                    // Headings
                    h1({ children }) {
                        return <h1 className="text-xl font-bold text-foreground mt-4 mb-2">{children}</h1>;
                    },
                    h2({ children }) {
                        return <h2 className="text-lg font-bold text-foreground mt-4 mb-2">{children}</h2>;
                    },
                    h3({ children }) {
                        return <h3 className="text-md font-semibold text-foreground mt-3 mb-1">{children}</h3>;
                    },

                    // Paragraphs
                    p({ children }) {
                        return <p className="my-2 leading-relaxed whitespace-pre-wrap">{children}</p>;
                    },

                    // Horizontal rule
                    hr() {
                        return <hr className="my-4 border-border" />;
                    },

                    // Checkboxes in task lists
                    input({ type, checked }) {
                        if (type === 'checkbox') {
                            return (
                                <input
                                    type="checkbox"
                                    checked={checked}
                                    readOnly
                                    className="mr-2 accent-primary"
                                />
                            );
                        }
                        return <input type={type} />;
                    },
                }}
            >
                {content}
            </ReactMarkdown>
        </div>
    );
});

// Separate code block component with copy functionality
interface CodeBlockProps {
    code: string;
    language: string;
}

function CodeBlock({ code, language }: CodeBlockProps) {
    const [copied, setCopied] = useState(false);
    const [isExpanded, setIsExpanded] = useState(false);

    // Check if code is long (e.g., > 15 lines)
    const lineCount = code.split('\n').length;
    const isLong = lineCount > 15;
    const shouldCollapse = isLong && !isExpanded;

    const handleCopy = useCallback(() => {
        navigator.clipboard.writeText(code);
        setCopied(true);
        toast.success('Copied to clipboard');
        setTimeout(() => setCopied(false), 2000);
    }, [code]);

    return (
        <div className="relative group my-3 rounded-md border border-border overflow-hidden bg-card">
            {/* Header / Language badge */}
            <div className="flex items-center justify-between bg-muted/50 border-b border-border px-3 py-1.5">
                <div className="flex items-center gap-2">
                    <span className="text-[10px] text-muted-foreground font-mono uppercase tracking-wider font-semibold">
                        {language || 'text'}
                    </span>
                    {isLong && (
                        <span className="text-[10px] text-muted-foreground/50">
                            {lineCount} lines
                        </span>
                    )}
                </div>
                <div className="flex items-center gap-1">
                    {isLong && (
                        <Button
                            variant="ghost"
                            size="sm"
                            className="h-6 text-[10px] px-2 h-auto py-0.5"
                            onClick={() => setIsExpanded(!isExpanded)}
                        >
                            {isExpanded ? 'Collapse' : 'Expand'}
                        </Button>
                    )}
                    <Button
                        variant="ghost"
                        size="icon"
                        className="h-6 w-6"
                        onClick={handleCopy}
                        title="Copy code"
                    >
                        {copied ? (
                            <Check className="w-3.5 h-3.5 text-green-500" />
                        ) : (
                            <Copy className="w-3.5 h-3.5" />
                        )}
                    </Button>
                </div>
            </div>

            {/* Code Content */}
            <div className={cn(
                "relative transition-all duration-200",
                shouldCollapse ? "max-h-[300px] overflow-hidden" : "h-auto"
            )}>
                <SyntaxHighlighter
                    style={oneDark}
                    language={language}
                    PreTag="div"
                    customStyle={{
                        margin: 0,
                        border: 'none',
                        background: 'transparent',
                        padding: '1rem',
                        fontSize: '13px',
                        lineHeight: '1.5',
                    }}
                    codeTagProps={{
                        style: {
                            fontFamily: "'JetBrains Mono', 'Fira Code', Consolas, monospace",
                        }
                    }}
                >
                    {code}
                </SyntaxHighlighter>

                {/* Gradient fade for collapsed state */}
                {shouldCollapse && (
                    <div className="absolute inset-x-0 bottom-0 h-24 bg-gradient-to-t from-card to-transparent pointer-events-none flex items-end justify-center pb-4">
                        <div className="pointer-events-auto">
                            <Button
                                variant="secondary"
                                size="sm"
                                className="shadow-lg h-7 text-xs bg-muted/80 backdrop-blur-sm hover:bg-muted"
                                onClick={() => setIsExpanded(true)}
                            >
                                Show {lineCount - 15} more lines
                            </Button>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}

export default MarkdownRenderer;
