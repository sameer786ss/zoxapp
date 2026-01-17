import { Panel, PanelGroup, PanelResizeHandle } from 'react-resizable-panels';
import { ReactNode, useState } from 'react';
import { cn } from '@/lib/utils';

interface ResizableLayoutProps {
    left: ReactNode;
    right: ReactNode;
    bottom?: ReactNode;
    showBottom?: boolean;
    defaultLeftSize?: number;
    defaultBottomSize?: number;
    minLeftSize?: number;
    minRightSize?: number;
    minBottomSize?: number;
}

export function ResizableLayout({
    left,
    right,
    bottom,
    showBottom = false,
    defaultLeftSize = 40,
    defaultBottomSize = 25,
    minLeftSize = 20,
    minRightSize = 30,
    minBottomSize = 10,
}: ResizableLayoutProps) {
    return (
        <PanelGroup direction="horizontal" className="h-full">
            {/* Left Panel (Chat) */}
            <Panel
                defaultSize={defaultLeftSize}
                minSize={minLeftSize}
                className="h-full"
            >
                {showBottom && bottom ? (
                    <PanelGroup direction="vertical">
                        <Panel defaultSize={100 - defaultBottomSize} minSize={30}>
                            {left}
                        </Panel>
                        <VerticalResizeHandle />
                        <Panel defaultSize={defaultBottomSize} minSize={minBottomSize}>
                            {bottom}
                        </Panel>
                    </PanelGroup>
                ) : (
                    left
                )}
            </Panel>

            {/* Resize Handle */}
            <HorizontalResizeHandle />

            {/* Right Panel (Editor) */}
            <Panel
                defaultSize={100 - defaultLeftSize}
                minSize={minRightSize}
                className="h-full"
            >
                {right}
            </Panel>
        </PanelGroup>
    );
}

// Vertical resize handle (horizontal line)
function VerticalResizeHandle() {
    const [isDragging, setIsDragging] = useState(false);

    return (
        <PanelResizeHandle
            className="group relative"
            onDragging={setIsDragging}
        >
            <div
                className={cn(
                    "h-1 w-full flex items-center justify-center transition-colors",
                    isDragging ? 'bg-primary/30' : 'bg-transparent hover:bg-accent'
                )}
            >
                {/* Visual indicator */}
                <div
                    className={cn(
                        "h-0.5 w-12 rounded-full transition-all",
                        isDragging ? 'bg-primary' : 'bg-border group-hover:bg-muted-foreground'
                    )}
                />
            </div>
            {/* Larger hit area */}
            <div className="absolute inset-x-0 -top-1 -bottom-1 cursor-row-resize" />
        </PanelResizeHandle>
    );
}

// Horizontal resize handle (vertical line)
function HorizontalResizeHandle() {
    const [isDragging, setIsDragging] = useState(false);

    return (
        <PanelResizeHandle
            className="group relative"
            onDragging={setIsDragging}
        >
            <div
                className={cn(
                    "w-1 h-full flex items-center justify-center transition-colors",
                    isDragging ? 'bg-primary/30' : 'bg-transparent hover:bg-accent'
                )}
            >
                {/* Visual indicator */}
                <div
                    className={cn(
                        "w-0.5 h-12 rounded-full transition-all",
                        isDragging ? 'bg-primary' : 'bg-border group-hover:bg-muted-foreground'
                    )}
                />
            </div>
            {/* Larger hit area */}
            <div className="absolute inset-y-0 -left-1 -right-1 cursor-col-resize" />
        </PanelResizeHandle>
    );
}

// Simpler two-pane horizontal layout
interface TwoPaneLayoutProps {
    left: ReactNode;
    right: ReactNode;
    defaultLeftSize?: number;
}

export function TwoPaneLayout({ left, right, defaultLeftSize = 40 }: TwoPaneLayoutProps) {
    return (
        <PanelGroup direction="horizontal" className="h-full">
            <Panel defaultSize={defaultLeftSize} minSize={25}>
                {left}
            </Panel>
            <HorizontalResizeHandle />
            <Panel defaultSize={100 - defaultLeftSize} minSize={25}>
                {right}
            </Panel>
        </PanelGroup>
    );
}

// Three-pane layout with sidebar
interface ThreePaneLayoutProps {
    sidebar: ReactNode;
    main: ReactNode;
    panel: ReactNode;
    sidebarSize?: number;
    mainSize?: number;
}

export function ThreePaneLayout({
    sidebar,
    main,
    panel,
    sidebarSize = 15,
    mainSize = 45,
}: ThreePaneLayoutProps) {
    return (
        <PanelGroup direction="horizontal" className="h-full">
            <Panel defaultSize={sidebarSize} minSize={10} maxSize={25}>
                {sidebar}
            </Panel>
            <HorizontalResizeHandle />
            <Panel defaultSize={mainSize} minSize={25}>
                {main}
            </Panel>
            <HorizontalResizeHandle />
            <Panel defaultSize={100 - sidebarSize - mainSize} minSize={25}>
                {panel}
            </Panel>
        </PanelGroup>
    );
}

export default ResizableLayout;
