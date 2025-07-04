import { createContext, useContext, useEffect } from "react"

type Theme = "dark"

type ThemeProviderProps = {
  children: React.ReactNode
  defaultTheme?: Theme
  storageKey?: string
}

type ThemeProviderState = {
  theme: Theme
  setTheme: (theme: Theme) => void
}

const initialState: ThemeProviderState = {
  theme: "dark",
  setTheme: () => null,
}

const ThemeProviderContext = createContext<ThemeProviderState>(initialState)

export function ThemeProvider({
  children,
  defaultTheme = "dark",
  storageKey = "snapper-theme",
  ...props
}: ThemeProviderProps) {

  useEffect(() => {
    const root = window.document.documentElement
    
    root.classList.remove("light")
    root.classList.add("dark")
    
    localStorage.setItem(storageKey, "dark")
  }, [storageKey])

  const value = {
    theme: "dark" as Theme,
    setTheme: () => {
      console.log("Theme is locked to dark mode")
    },
  }

  return (
    <ThemeProviderContext.Provider {...props} value={value}>
      {children}
    </ThemeProviderContext.Provider>
  )
}

export const useTheme = () => {
  const context = useContext(ThemeProviderContext)

  if (context === undefined)
    throw new Error("useTheme must be used within a ThemeProvider")

  return context
} 