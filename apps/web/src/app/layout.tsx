import Link from "next/link";
import type { Metadata } from "next";
import "./globals.css";
import NotificationCenter from "./add-notification-center-ui";
import DarkModeToggle from "./add-dark-mode-support";

// Using system fonts to avoid external Google Fonts fetch during build

export const metadata: Metadata = {
  title: "Soroban CrashLab | Smart Contract Fuzzing",
  description:
    "Intelligent mutation testing and runtime behavior tracing for Soroban smart contracts.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body
        className={`antialiased min-h-screen flex flex-col`}
        style={{ background: "var(--background)", color: "var(--foreground)" }}
      >
        <header
          className="border-b p-6 flex items-center justify-between"
          style={{
            borderColor: "var(--header-border-color)",
            background: "var(--background)",
            color: "var(--foreground)",
          }}
        >
          <div className="font-semibold text-xl tracking-tight">
            Soroban CrashLab
          </div>
          <div className="flex items-center gap-4">
            <nav className="flex gap-4 text-sm font-medium">
              <Link
                href="/"
                className="hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
              >
                Dashboard
              </Link>
              <Link
                href="/logs"
                className="hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
              >
                Logs
              </Link>
              <Link
                href="/triage"
                className="hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
              >
                Triage
              </Link>
              <Link
                href="/trends"
                className="hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
              >
                Trends
              </Link>
              <Link
                href="/maintainer"
                className="hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
              >
                Maintainer
              </Link>
              <Link
                href="/settings/alerting"
                className="hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
              >
                Settings
              </Link>
            </nav>
            <NotificationCenter />
            <DarkModeToggle />
          </div>
        </header>
        <main className="flex-1 flex flex-col">{children}</main>
        <footer className="border-t border-black/8 dark:border-white/15 p-6 text-center text-sm text-zinc-500">
          Built for Stellar &middot; Soroban Ecosystem &middot;{" "}
          <a
            href="https://github.com/SorobanCrashLab/soroban-crashlab/milestone/1"
            className="underline hover:text-blue-600 dark:hover:text-blue-400 transition-colors"
            target="_blank"
            rel="noopener noreferrer"
          >
            Roadmap
          </a>
        </footer>
      </body>
    </html>
  );
}
