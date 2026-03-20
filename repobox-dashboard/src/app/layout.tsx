export const metadata = { title: 'repo.box — Build Dashboard' }

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body style={{ margin: 0, background: '#0a0a0a', color: '#e0e0e0' }}>
        {children}
      </body>
    </html>
  )
}
