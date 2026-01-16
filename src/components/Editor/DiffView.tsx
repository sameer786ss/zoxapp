import { useState, useMemo } from 'react';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Badge } from '@/components/ui/badge';
import { Plus, Minus, ChevronDown, ChevronRight, FileCode } from 'lucide-react';

interface DiffLine {
    type: 'added' | 'removed' | 'unchanged' | 'header';
    content: string;
    lineNumber?: number;
    oldLineNumber?: number;
    newLineNumber?: number;
}

interface DiffViewProps {
    oldContent: string;
    newContent: string;
    fileName?: string;
    language?: string;
}

/**
 * Unified diff view component for showing file changes
 */
export default function DiffView({ oldContent, newContent, fileName, language }: DiffViewProps) {
    const [collapsed, setCollapsed] = useState(false);

    const diffLines = useMemo(() => {
        return computeDiff(oldContent, newContent);
    }, [oldContent, newContent]);

    const stats = useMemo(() => {
        let added = 0;
        let removed = 0;
        diffLines.forEach(line => {
            if (line.type === 'added') added++;
            if (line.type === 'removed') removed++;
        });
        return { added, removed };
    }, [diffLines]);

    return (
        <div className="border border-border rounded-lg overflow-hidden">
            {/* Header */}
            <div
                className="flex items-center justify-between px-3 py-2 bg-secondary/30 border-b border-border cursor-pointer hover:bg-secondary/50 transition-colors"
                onClick={() => setCollapsed(!collapsed)}
            >
                <div className="flex items-center gap-2">
                    {collapsed ? (
                        <ChevronRight className="w-4 h-4 text-muted-foreground" />
                    ) : (
                        <ChevronDown className="w-4 h-4 text-muted-foreground" />
                    )}
                    <FileCode className="w-4 h-4 text-muted-foreground" />
                    <span className="text-sm font-medium">{fileName || 'Diff'}</span>
                    {language && (
                        <Badge variant="outline" className="text-[10px] h-4">
                            {language}
                        </Badge>
                    )}
                </div>
                <div className="flex items-center gap-2">
                    <Badge variant="secondary" className="text-[10px] h-5 gap-1 bg-green-500/20 text-green-400">
                        <Plus className="w-3 h-3" />
                        {stats.added}
                    </Badge>
                    <Badge variant="secondary" className="text-[10px] h-5 gap-1 bg-red-500/20 text-red-400">
                        <Minus className="w-3 h-3" />
                        {stats.removed}
                    </Badge>
                </div>
            </div>

            {/* Diff Content */}
            {!collapsed && (
                <ScrollArea className="max-h-[400px]">
                    <div className="font-mono text-xs">
                        {diffLines.map((line, index) => (
                            <div
                                key={index}
                                className={`flex ${getDiffLineClass(line.type)} px-2 py-0.5`}
                            >
                                <span className="w-12 text-right pr-2 text-muted-foreground select-none shrink-0">
                                    {line.oldLineNumber || ''}
                                </span>
                                <span className="w-12 text-right pr-2 text-muted-foreground select-none shrink-0">
                                    {line.newLineNumber || ''}
                                </span>
                                <span className="w-4 text-center select-none shrink-0">
                                    {line.type === 'added' ? '+' : line.type === 'removed' ? '-' : ' '}
                                </span>
                                <span className="flex-1 whitespace-pre overflow-x-auto">
                                    {line.content}
                                </span>
                            </div>
                        ))}
                    </div>
                </ScrollArea>
            )}
        </div>
    );
}

function getDiffLineClass(type: DiffLine['type']): string {
    switch (type) {
        case 'added':
            return 'bg-green-500/10 text-green-300';
        case 'removed':
            return 'bg-red-500/10 text-red-300';
        case 'header':
            return 'bg-blue-500/10 text-blue-300';
        default:
            return '';
    }
}

/**
 * Simple line-by-line diff computation
 */
function computeDiff(oldContent: string, newContent: string): DiffLine[] {
    const oldLines = oldContent.split('\n');
    const newLines = newContent.split('\n');
    const result: DiffLine[] = [];

    // Use simple LCS-based diff for now
    const lcs = longestCommonSubsequence(oldLines, newLines);

    let oldIdx = 0;
    let newIdx = 0;
    let lcsIdx = 0;

    while (oldIdx < oldLines.length || newIdx < newLines.length) {
        if (lcsIdx < lcs.length && oldIdx < oldLines.length && oldLines[oldIdx] === lcs[lcsIdx]) {
            if (newIdx < newLines.length && newLines[newIdx] === lcs[lcsIdx]) {
                // Line unchanged
                result.push({
                    type: 'unchanged',
                    content: oldLines[oldIdx],
                    oldLineNumber: oldIdx + 1,
                    newLineNumber: newIdx + 1,
                });
                oldIdx++;
                newIdx++;
                lcsIdx++;
            } else {
                // Line added in new
                result.push({
                    type: 'added',
                    content: newLines[newIdx],
                    newLineNumber: newIdx + 1,
                });
                newIdx++;
            }
        } else if (oldIdx < oldLines.length) {
            // Line removed from old
            result.push({
                type: 'removed',
                content: oldLines[oldIdx],
                oldLineNumber: oldIdx + 1,
            });
            oldIdx++;
        } else if (newIdx < newLines.length) {
            // Line added in new
            result.push({
                type: 'added',
                content: newLines[newIdx],
                newLineNumber: newIdx + 1,
            });
            newIdx++;
        }
    }

    return result;
}

/**
 * Compute longest common subsequence of lines
 */
function longestCommonSubsequence(a: string[], b: string[]): string[] {
    const m = a.length;
    const n = b.length;
    const dp: number[][] = Array(m + 1).fill(null).map(() => Array(n + 1).fill(0));

    for (let i = 1; i <= m; i++) {
        for (let j = 1; j <= n; j++) {
            if (a[i - 1] === b[j - 1]) {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = Math.max(dp[i - 1][j], dp[i][j - 1]);
            }
        }
    }

    // Backtrack to find LCS
    const lcs: string[] = [];
    let i = m, j = n;
    while (i > 0 && j > 0) {
        if (a[i - 1] === b[j - 1]) {
            lcs.unshift(a[i - 1]);
            i--;
            j--;
        } else if (dp[i - 1][j] > dp[i][j - 1]) {
            i--;
        } else {
            j--;
        }
    }

    return lcs;
}
