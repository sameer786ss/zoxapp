import Sidebar from './Sidebar';
import StatusBar from './StatusBar';
import TitleBar from './TitleBar';

interface MainLayoutProps {
    children: React.ReactNode;
}

export default function MainLayout({ children }: MainLayoutProps) {
    return (
        <div className="h-screen w-screen flex flex-col bg-transparent overflow-hidden">
            {/* Custom Title Bar */}
            <TitleBar />

            {/* Main Content Area */}
            <div className="flex-1 flex overflow-hidden">
                {/* Sidebar */}
                <Sidebar />

                {/* Content */}
                <main className="flex-1 overflow-hidden transition-all duration-normal ease-fluent">
                    {children}
                </main>
            </div>

            {/* Status Bar */}
            <StatusBar />
        </div>
    );
}
