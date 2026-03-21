'use client'
import { useTheme } from '@/hooks/useTheme'

interface ThemeToggleProps {
  className?: string
  size?: 'sm' | 'md' | 'lg'
}

export function ThemeToggle({ className = '', size = 'md' }: ThemeToggleProps) {
  const { theme, toggleTheme } = useTheme()

  const sizeMap = {
    sm: { width: 20, height: 12, handle: 10 },
    md: { width: 24, height: 14, handle: 12 },
    lg: { width: 28, height: 16, handle: 14 }
  }

  const { width, height, handle } = sizeMap[size]

  return (
    <button
      onClick={toggleTheme}
      className={`theme-toggle ${className}`}
      aria-label={`Switch to ${theme === 'dark' ? 'light' : 'dark'} theme`}
      style={{
        background: 'none',
        border: 'none',
        cursor: 'pointer',
        padding: '4px',
        borderRadius: '50%',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        transition: 'all 200ms ease-in-out',
        ...({
          ':hover': {
            backgroundColor: 'var(--hover-bg)',
          }
        } as any)
      }}
      onMouseEnter={(e) => {
        e.currentTarget.style.backgroundColor = 'var(--hover-bg)'
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.backgroundColor = 'transparent'
      }}
    >
      <div
        style={{
          position: 'relative',
          width: `${width}px`,
          height: `${height}px`,
          borderRadius: `${height / 2}px`,
          backgroundColor: theme === 'dark' ? '#374151' : '#FEF3C7',
          transition: 'background-color 200ms ease-in-out',
          border: `1px solid ${theme === 'dark' ? '#4b5563' : '#f59e0b'}`,
        }}
      >
        <div
          style={{
            position: 'absolute',
            top: '1px',
            left: theme === 'dark' ? '2px' : `${width - handle - 2}px`,
            width: `${handle}px`,
            height: `${handle}px`,
            borderRadius: '50%',
            backgroundColor: theme === 'dark' ? '#F3F4F6' : '#F59E0B',
            transition: 'all 200ms ease-in-out',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: `${handle * 0.6}px`,
            boxShadow: '0 1px 3px rgba(0, 0, 0, 0.1)',
          }}
        >
          {theme === 'dark' ? '🌙' : '☀️'}
        </div>
      </div>
    </button>
  )
}