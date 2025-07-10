import { createTheme } from "@mui/material/styles";

export const theme = createTheme({
  palette: {
    mode: "light",
    primary: {
      main: "#000000",
      light: "#333333",
      dark: "#000000",
    },
    secondary: {
      main: "#666666",
      light: "#999999",
      dark: "#333333",
    },
    background: {
      default: "#ffffff",
      paper: "#fafafa",
    },
    text: {
      primary: "#000000",
      secondary: "#666666",
    },
    divider: "#e5e5e5",
  },
  typography: {
    fontFamily: '"Inter", "Roboto", "Helvetica", "Arial", sans-serif',
    h1: {
      fontSize: "2rem",
      fontWeight: 600,
      letterSpacing: "-0.02em",
    },
    h2: {
      fontSize: "1.5rem",
      fontWeight: 600,
      letterSpacing: "-0.01em",
    },
    h3: {
      fontSize: "1.25rem",
      fontWeight: 600,
    },
    h4: {
      fontSize: "1rem",
      fontWeight: 500,
    },
    body1: {
      fontSize: "0.875rem",
      lineHeight: 1.6,
    },
    body2: {
      fontSize: "0.75rem",
      lineHeight: 1.5,
    },
  },
  shape: {
    borderRadius: 8,
  },
  components: {
    MuiButton: {
      styleOverrides: {
        root: {
          textTransform: "none",
          borderRadius: 6,
          fontWeight: 500,
          boxShadow: "none",
          "&:hover": {
            boxShadow: "none",
          },
        },
        contained: {
          backgroundColor: "#000000",
          color: "#ffffff",
          "&:hover": {
            backgroundColor: "#333333",
          },
        },
        outlined: {
          borderColor: "#e5e5e5",
          color: "#000000",
          "&:hover": {
            backgroundColor: "#fafafa",
            borderColor: "#cccccc",
          },
        },
      },
    },
    MuiCard: {
      styleOverrides: {
        root: {
          boxShadow: "0 1px 3px rgba(0,0,0,0.12), 0 1px 2px rgba(0,0,0,0.24)",
          border: "1px solid #f0f0f0",
        },
      },
    },
    MuiPaper: {
      styleOverrides: {
        root: {
          backgroundImage: "none",
        },
      },
    },
    MuiAppBar: {
      styleOverrides: {
        root: {
          backgroundColor: "#ffffff",
          color: "#000000",
          boxShadow: "0 1px 0 rgba(0,0,0,0.05)",
        },
      },
    },
    MuiTextField: {
      styleOverrides: {
        root: {
          "& .MuiOutlinedInput-root": {
            "&:hover fieldset": {
              borderColor: "#cccccc",
            },
            "&.Mui-focused fieldset": {
              borderColor: "#000000",
            },
          },
        },
      },
    },
  },
}); 