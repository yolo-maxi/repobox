import '@/styles/themes.css'

export const metadata = { title: 'repo.box — Build Dashboard' }

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body style={{ 
        margin: 0, 
        background: 'var(--bg-primary)', 
        color: 'var(--text-primary)',
        transition: 'background-color 200ms ease-in-out, color 200ms ease-in-out'
      }}>
        {children}
      </body>
    </html>
  )
}
