'use client'
import { useState, useEffect, useCallback } from 'react'

export type Theme = 'dark' | 'light'
export type ThemePreference = 'dark' | 'light' | 'auto'

interface ThemeState {
  theme: Theme
  systemPreference: Theme
  userPreference: ThemePreference
  setTheme: (theme: ThemePreference) => void
  toggleTheme: () => void
}

const STORAGE_KEY = 'repobox:theme'

function getSystemPreference(): Theme {
  if (typeof window === 'undefined') return 'dark'
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

function getStoredPreference(): ThemePreference {
  if (typeof window === 'undefined') return 'auto'
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (!stored) return 'auto'
    const parsed = JSON.parse(stored)
    if (['dark', 'light', 'auto'].includes(parsed.theme)) {
      return parsed.theme
    }
    return 'auto'
  } catch {
    return 'auto'
  }
}

function storePreference(preference: ThemePreference): void {
  if (typeof window === 'undefined') return
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({
      theme: preference,
      lastUpdated: Date.now()
    }))
  } catch {
    // localStorage might be disabled, fail silently
  }
}

function applyTheme(theme: Theme): void {
  if (typeof window === 'undefined') return
  document.documentElement.setAttribute('data-theme', theme)
}

export function useTheme(): ThemeState {
  const [systemPreference, setSystemPreference] = useState<Theme>('dark')
  const [userPreference, setUserPreference] = useState<ThemePreference>('auto')

  // Calculate the effective theme
  const theme: Theme = userPreference === 'auto' ? systemPreference : userPreference

  // Initialize theme on mount
  useEffect(() => {
    const initialSystem = getSystemPreference()
    const initialUser = getStoredPreference()
    
    setSystemPreference(initialSystem)
    setUserPreference(initialUser)
    
    // Apply the initial theme immediately
    const effectiveTheme = initialUser === 'auto' ? initialSystem : initialUser
    applyTheme(effectiveTheme)
  }, [])

  // Listen for system preference changes
  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    
    function handleChange(e: MediaQueryListEvent) {
      const newSystemPref = e.matches ? 'dark' : 'light'
      setSystemPreference(newSystemPref)
      
      // If user has auto preference, update theme immediately
      if (userPreference === 'auto') {
        applyTheme(newSystemPref)
      }
    }
    
    mediaQuery.addEventListener('change', handleChange)
    return () => mediaQuery.removeEventListener('change', handleChange)
  }, [userPreference])

  // Apply theme when it changes
  useEffect(() => {
    applyTheme(theme)
  }, [theme])

  const setTheme = useCallback((newPreference: ThemePreference) => {
    setUserPreference(newPreference)
    storePreference(newPreference)
    
    // Apply theme immediately
    const newTheme = newPreference === 'auto' ? systemPreference : newPreference
    applyTheme(newTheme)
  }, [systemPreference])

  const toggleTheme = useCallback(() => {
    // Simple toggle: dark <-> light (ignoring auto for now)
    const newTheme = theme === 'dark' ? 'light' : 'dark'
    setTheme(newTheme)
  }, [theme, setTheme])

  return {
    theme,
    systemPreference,
    userPreference,
    setTheme,
    toggleTheme
  }
}