import { Sparkles } from 'lucide-react';
import WindowControls from './WindowControls';

export default function TitleBar() {
    return (
        <div
            className="title-bar h-8 flex items-center justify-between bg-transparent border-b border-border/50 shrink-0"
            data-tauri-drag-region
        >
            {/* Left side - App branding */}
            <div className="flex items-center gap-2 pl-3 pointer-events-none">
                <div className="w-4 h-4 rounded-sm bg-gradient-to-br from-blue-500 to-purple-600 flex items-center justify-center">
                    <Sparkles className="w-2.5 h-2.5 text-white" />
                </div>
                <span className="text-xs font-medium text-foreground/70">
                    Agent IDE
                </span>
            </div>

            {/* Right side - Window controls */}
            <WindowControls />
        </div>
    );
}
