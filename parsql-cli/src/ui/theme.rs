//! Modern terminal theme colors

use ratatui::style::Color;

pub struct ModernTheme;

impl ModernTheme {
    // Background colors
    pub const BG_PRIMARY: Color = Color::Rgb(24, 24, 27);      // #18181b - Main background
    pub const BG_SECONDARY: Color = Color::Rgb(39, 39, 42);    // #27272a - Secondary panels
    pub const BG_TERTIARY: Color = Color::Rgb(63, 63, 70);     // #3f3f46 - Highlighted items
    
    // Text colors
    pub const TEXT_PRIMARY: Color = Color::Rgb(244, 244, 245);   // #f4f4f5 - Primary text
    pub const TEXT_SECONDARY: Color = Color::Rgb(161, 161, 170); // #a1a1aa - Secondary text
    pub const TEXT_DIM: Color = Color::Rgb(113, 113, 122);       // #71717a - Dimmed text
    
    // Accent colors
    pub const ACCENT_PRIMARY: Color = Color::Rgb(59, 130, 246);  // #3b82f6 - Blue (primary actions)
    pub const ACCENT_SUCCESS: Color = Color::Rgb(34, 197, 94);   // #22c55e - Green (success)
    pub const ACCENT_WARNING: Color = Color::Rgb(250, 204, 21);  // #facc15 - Yellow (warning)
    pub const ACCENT_ERROR: Color = Color::Rgb(239, 68, 68);     // #ef4444 - Red (error)
    pub const ACCENT_INFO: Color = Color::Rgb(14, 165, 233);     // #0ea5e9 - Light blue (info)
    
    // Border colors
    pub const BORDER_PRIMARY: Color = Color::Rgb(63, 63, 70);    // #3f3f46
    pub const BORDER_FOCUSED: Color = Color::Rgb(59, 130, 246);  // #3b82f6
    
    // Special UI elements
    pub const COMMAND_BG: Color = Color::Rgb(30, 30, 35);        // Slightly darker for command input
    pub const SELECTION_BG: Color = Color::Rgb(59, 130, 246);    // Blue for selections
    pub const SCROLLBAR: Color = Color::Rgb(113, 113, 122);      // Dim for scrollbars
    
    // Aliases for common names
    pub const BORDER: Color = Self::BORDER_PRIMARY;
    pub const TEXT_MUTED: Color = Self::TEXT_DIM;
    pub const SUCCESS: Color = Self::ACCENT_SUCCESS;
    pub const WARNING: Color = Self::ACCENT_WARNING;
}