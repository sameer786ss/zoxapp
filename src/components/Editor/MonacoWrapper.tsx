import { useState, useEffect, useRef, useCallback } from 'react';
import Editor, { OnMount } from '@monaco-editor/react';
import { editor } from 'monaco-editor';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { useAgentStore, selectActiveFile } from '@/stores/useAgentStore';
import { X, FileText, Circle, Save } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

interface MonacoWrapperProps {
    className?: string;
}

export default function MonacoWrapper({ className }: MonacoWrapperProps) {
    const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);
    const [isLoading, setIsLoading] = useState(false);

    const openFiles = useAgentStore((state) => state.openFiles);
    const activeFileIndex = useAgentStore((state) => state.activeFileIndex);
    const activeFile = useAgentStore(selectActiveFile);
    const closeFile = useAgentStore((state) => state.closeFile);
    const setActiveFile = useAgentStore((state) => state.setActiveFile);
    const updateFileContent = useAgentStore((state) => state.updateFileContent);
    const markFileSaved = useAgentStore((state) => state.markFileSaved);

    // Load file content when active file changes
    useEffect(() => {
        if (!activeFile || activeFile.content !== undefined) return;

        const loadContent = async () => {
            setIsLoading(true);
            try {
                const content = await invoke<string>('read_workspace_file', {
                    path: activeFile.path
                });
                updateFileContent(activeFileIndex, content);
            } catch (err) {
                console.error('Failed to load file:', err);
            } finally {
                setIsLoading(false);
            }
        };

        loadContent();
    }, [activeFile?.path, activeFileIndex]);

    // Listen for workspace file changes
    useEffect(() => {
        let unlisten: UnlistenFn | null = null;

        const setupListener = async () => {
            unlisten = await listen<string[]>('workspace-file-change', (event) => {
                // Reload file if it's currently open and was modified externally
                if (activeFile && event.payload.some(p => p.includes(activeFile.name))) {
                    invoke<string>('read_workspace_file', { path: activeFile.path })
                        .then(content => updateFileContent(activeFileIndex, content))
                        .catch(console.error);
                }
            });
        };

        setupListener();
        return () => { unlisten?.(); };
    }, [activeFile?.path, activeFileIndex]);

    const handleEditorMount: OnMount = useCallback((editor, monaco) => {
        editorRef.current = editor;

        // Define Material Dark theme
        monaco.editor.defineTheme('material-dark', {
            base: 'vs-dark',
            inherit: true,
            rules: [
                { token: 'comment', foreground: '6A9955', fontStyle: 'italic' },
                { token: 'keyword', foreground: '569CD6' },
                { token: 'string', foreground: 'CE9178' },
                { token: 'number', foreground: 'B5CEA8' },
                { token: 'type', foreground: '4EC9B0' },
                { token: 'function', foreground: 'DCDCAA' },
                { token: 'variable', foreground: '9CDCFE' },
            ],
            colors: {
                'editor.background': '#1e1e1e',
                'editor.foreground': '#d4d4d4',
                'editorCursor.foreground': '#aeafad',
                'editor.lineHighlightBackground': '#2d2d30',
                'editorLineNumber.foreground': '#858585',
                'editor.selectionBackground': '#264f78',
                'editor.inactiveSelectionBackground': '#3a3d41',
                'editorIndentGuide.background': '#404040',
                'editorIndentGuide.activeBackground': '#707070',
                'editorWidget.background': '#252526',
                'editorWidget.border': '#454545',
                'editorSuggestWidget.background': '#252526',
                'editorSuggestWidget.border': '#454545',
                'editorSuggestWidget.selectedBackground': '#062f4a',
                'scrollbar.shadow': '#00000000',
                'scrollbarSlider.background': '#79797966',
                'scrollbarSlider.hoverBackground': '#646464b3',
                'scrollbarSlider.activeBackground': '#bfbfbf66',
            },
        });

        monaco.editor.setTheme('material-dark');
        editor.updateOptions({
            fontFamily: "'Cascadia Code', 'Fira Code', 'JetBrains Mono', Consolas, monospace",
            fontSize: 14,
            lineHeight: 22,
            minimap: { enabled: true, side: 'right', size: 'proportional' },
            smoothScrolling: true,
            cursorBlinking: 'smooth',
            cursorSmoothCaretAnimation: 'on',
            padding: { top: 16, bottom: 16 },
            scrollbar: {
                verticalScrollbarSize: 10,
                horizontalScrollbarSize: 10,
            },
            renderLineHighlight: 'line',
            wordWrap: 'on',
            formatOnPaste: true,
            formatOnType: true,
        });

        // Ctrl+S to save
        editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
            handleSave();
        });
    }, []);

    const handleEditorChange = useCallback((value: string | undefined) => {
        if (value !== undefined && activeFileIndex >= 0) {
            updateFileContent(activeFileIndex, value);
        }
    }, [activeFileIndex, updateFileContent]);

    const handleSave = async () => {
        if (!activeFile || activeFile.content === undefined) return;

        try {
            await invoke('save_workspace_file', {
                path: activeFile.path,
                content: activeFile.content
            });
            markFileSaved(activeFileIndex);
        } catch (err) {
            console.error('Failed to save:', err);
        }
    };

    return (
        <div className={cn("h-full flex flex-col bg-[#1e1e1e]", className)}>
            {/* Tab bar */}
            {openFiles.length > 0 && (
                <div className="flex items-center bg-[#252526] border-b border-[#3c3c3c] overflow-x-auto">
                    {openFiles.map((file, index) => (
                        <div
                            key={file.path}
                            className={cn(
                                "flex items-center gap-2 px-3 py-2 border-r border-[#3c3c3c] cursor-pointer min-w-0 group",
                                index === activeFileIndex
                                    ? "bg-[#1e1e1e] text-white"
                                    : "bg-[#2d2d30] text-gray-400 hover:bg-[#2a2a2a]"
                            )}
                            onClick={() => setActiveFile(index)}
                        >
                            <FileText className="w-4 h-4 shrink-0 text-blue-400" />
                            <span className="text-xs truncate max-w-[120px]">{file.name}</span>

                            {/* Modified indicator */}
                            {file.isModified && (
                                <Circle className="w-2 h-2 fill-amber-400 text-amber-400 shrink-0" />
                            )}

                            {/* Close button */}
                            <button
                                onClick={(e) => {
                                    e.stopPropagation();
                                    closeFile(index);
                                }}
                                className="opacity-0 group-hover:opacity-100 hover:bg-[#3c3c3c] rounded p-0.5 transition-opacity"
                            >
                                <X className="w-3 h-3" />
                            </button>
                        </div>
                    ))}
                </div>
            )}

            {/* Editor */}
            <div className="flex-1 overflow-hidden">
                {isLoading ? (
                    <div className="h-full flex items-center justify-center text-gray-500">
                        <div className="text-center">
                            <div className="animate-spin w-8 h-8 border-2 border-primary border-t-transparent rounded-full mx-auto mb-2" />
                            <p className="text-sm">Loading...</p>
                        </div>
                    </div>
                ) : activeFile ? (
                    <Editor
                        height="100%"
                        language={activeFile.language}
                        value={activeFile.content || ''}
                        theme="material-dark"
                        options={{
                            readOnly: false,
                            automaticLayout: true,
                        }}
                        onChange={handleEditorChange}
                        onMount={handleEditorMount}
                        loading={
                            <div className="h-full flex items-center justify-center text-gray-500">
                                <div className="animate-pulse">Loading editor...</div>
                            </div>
                        }
                    />
                ) : (
                    <EmptyEditor />
                )}
            </div>

            {/* Status bar */}
            {activeFile && (
                <div className="flex items-center justify-between px-3 py-1 bg-[#007acc] text-white text-xs">
                    <div className="flex items-center gap-3">
                        <span>{activeFile.language}</span>
                        {activeFile.isModified && (
                            <span className="text-amber-200">Modified</span>
                        )}
                    </div>
                    <div className="flex items-center gap-3">
                        <span className="opacity-70">{activeFile.source === 'agent' ? 'Agent' : 'User'}</span>
                        <Button
                            size="sm"
                            variant="ghost"
                            className="h-5 px-2 text-white hover:bg-white/20"
                            onClick={handleSave}
                            disabled={!activeFile.isModified}
                        >
                            <Save className="w-3 h-3 mr-1" />
                            Save
                        </Button>
                    </div>
                </div>
            )}
        </div>
    );
}

function EmptyEditor() {
    return (
        <div className="h-full flex items-center justify-center text-gray-500">
            <div className="text-center">
                <FileText className="w-12 h-12 mx-auto mb-3 opacity-30" />
                <p className="text-sm">No file open</p>
                <p className="text-xs text-gray-600 mt-1">
                    Ask ZOX to create or edit a file
                </p>
            </div>
        </div>
    );
}
