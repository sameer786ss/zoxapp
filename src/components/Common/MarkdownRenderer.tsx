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
                        return <p className="my-2 leading-relaxed">{children}</p>;
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

    const handleCopy = useCallback(() => {
        navigator.clipboard.writeText(code);
        setCopied(true);
        toast.success('Copied to clipboard');
        setTimeout(() => setCopied(false), 2000);
    }, [code]);

    return (
        <div className="relative group my-3">
            {/* Language badge */}
            <div className="flex items-center justify-between bg-card border-t border-x border-border rounded-t-md px-3 py-1.5">
                <span className="text-[10px] text-muted-foreground font-mono uppercase tracking-wider">
                    {language}
                </span>
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

            {/* Code */}
            <SyntaxHighlighter
                style={oneDark}
                language={language}
                PreTag="div"
                customStyle={{
                    margin: 0,
                    borderRadius: '0 0 6px 6px',
                    border: '1px solid hsl(var(--border))',
                    borderTop: 'none',
                    background: 'hsl(var(--background))',
                    padding: '1rem',
                    fontSize: '13px',
                }}
                codeTagProps={{
                    style: {
                        fontFamily: "'JetBrains Mono', 'Fira Code', Consolas, monospace",
                    }
                }}
            >
                {code}
            </SyntaxHighlighter>
        </div>
    );
}

export default MarkdownRenderer;
