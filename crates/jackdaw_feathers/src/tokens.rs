use bevy::color::palettes::tailwind;
use bevy::prelude::*;

// ---------------------------------------------------------------------------
// Corner radii
// ---------------------------------------------------------------------------

pub const CORNER_RADIUS: Val = Val::Px(2.0);
pub const CORNER_RADIUS_LG: Val = Val::Px(4.0);

// ---------------------------------------------------------------------------
// Accent / primary
// ---------------------------------------------------------------------------

pub const PRIMARY_COLOR: Srgba = tailwind::BLUE_500;
/// Accent blue used for selections, active tabs, and highlights.
pub const ACCENT_BLUE: Color = Color::srgb(0.0, 0.447, 0.843); // #0072D7

// ---------------------------------------------------------------------------
// Backgrounds — from Figma CSS
// ---------------------------------------------------------------------------

/// Root window background (#171717)
pub const WINDOW_BG: Color = Color::srgb(0.090, 0.090, 0.090);
/// Panel body / content background (#2A2A2A)
pub const PANEL_BG: Color = Color::srgb(0.165, 0.165, 0.165);
/// Panel header / tab bar background — same as window (#171717)
pub const PANEL_HEADER_BG: Color = Color::srgb(0.090, 0.090, 0.090);
/// Toolbar background (#2A2A2A)
pub const TOOLBAR_BG: Color = Color::srgb(0.165, 0.165, 0.165);
/// Text input / elevated input background (#404040)
pub const INPUT_BG: Color = Color::srgb(0.251, 0.251, 0.251);
/// Context menu / dropdown background (#2A2A2A with near-opaque alpha)
pub const MENU_BG: Color = Color::srgba(0.165, 0.165, 0.165, 0.98);
/// Status bar background (#2A2A2A)
pub const STATUS_BAR_BG: Color = Color::srgb(0.165, 0.165, 0.165);
/// Inactive toolbar button background
pub const TOOLBAR_BUTTON_BG: Color = Color::srgb(0.165, 0.165, 0.165);
/// General background color (for widgets)
pub const BACKGROUND_COLOR: Srgba = tailwind::ZINC_800;

// ---------------------------------------------------------------------------
// Component card backgrounds (inspector)
// ---------------------------------------------------------------------------

/// Component card body (#2A2A2A)
pub const COMPONENT_CARD_BG: Color = Color::srgb(0.165, 0.165, 0.165);
/// Component card header bar (#404040)
pub const COMPONENT_CARD_HEADER_BG: Color = Color::srgb(0.251, 0.251, 0.251);
/// Component card border — rgba(255,255,255,0.1)
pub const COMPONENT_CARD_BORDER: Color = Color::srgba(1.0, 1.0, 1.0, 0.1);

// ---------------------------------------------------------------------------
// Panel / tab styling
// ---------------------------------------------------------------------------

/// Panel header border — rgba(255,255,255,0.15)
pub const PANEL_BORDER: Color = Color::srgba(1.0, 1.0, 1.0, 0.15);
/// Active tab background — flat approximation of Figma gradient (#2A2A2A + slight white)
pub const TAB_ACTIVE_BG: Color = Color::srgb(0.175, 0.175, 0.175);
/// Active tab top border accent (#0072D7)
pub const TAB_ACTIVE_BORDER: Color = Color::srgb(0.0, 0.447, 0.843);
/// Inactive tab text (#A4A4A6)
pub const TAB_INACTIVE_TEXT: Color = Color::srgb(0.643, 0.643, 0.651);

// ---------------------------------------------------------------------------
// Viewport-specific backgrounds
// ---------------------------------------------------------------------------

/// Viewport tab bar (#1F1F24)
pub const VIEWPORT_TAB_BG: Color = Color::srgb(0.122, 0.122, 0.141);
/// Viewport control bar (#2A2A2E)
pub const VIEWPORT_CONTROL_BG: Color = Color::srgb(0.165, 0.165, 0.180);

// ---------------------------------------------------------------------------
// Elevated / input surfaces
// ---------------------------------------------------------------------------

/// Elevated background for inputs and interactive elements (#404040)
pub const ELEVATED_BG: Color = Color::srgb(0.251, 0.251, 0.251);
/// Active toolbar button background (#515151)
pub const TOOLBAR_ACTIVE_BG: Color = Color::srgb(0.318, 0.318, 0.318);

// ---------------------------------------------------------------------------
// Borders
// ---------------------------------------------------------------------------

/// Subtle border / separator (#404040)
pub const BORDER_SUBTLE: Color = Color::srgb(0.251, 0.251, 0.251);
/// Strong / emphasized border (#525252)
pub const BORDER_STRONG: Color = Color::Srgba(tailwind::ZINC_600);
/// Standard border color (for widgets)
pub const BORDER_COLOR: Srgba = tailwind::ZINC_700;

// ---------------------------------------------------------------------------
// Interaction states
// ---------------------------------------------------------------------------

/// Hovered row / item background
pub const HOVER_BG: Color = Color::srgba(1.0, 1.0, 1.0, 0.08);
/// Selected item background
pub const SELECTED_BG: Color = Color::srgba(0.0, 0.204, 0.431, 1.0);
/// Selected item border (#0072D7)
pub const SELECTED_BORDER: Color = Color::srgb(0.0, 0.447, 0.843);
/// Active / pressed background
pub const ACTIVE_BG: Color = Color::Srgba(tailwind::ZINC_600);
/// Drag-drop target highlight
pub const DROP_TARGET_BG: Color = Color::Srgba(Srgba {
    red: 0.3,
    green: 0.5,
    blue: 0.2,
    alpha: 1.0,
});
/// Drag-drop target border accent
pub const DROP_TARGET_BORDER: Color = Color::Srgba(Srgba {
    red: 0.3,
    green: 0.7,
    blue: 0.4,
    alpha: 1.0,
});
/// Root container drag-drop overlay
pub const CONTAINER_DROP_TARGET_BG: Color = Color::Srgba(Srgba {
    red: 0.2,
    green: 0.3,
    blue: 0.2,
    alpha: 0.3,
});
/// Tree connection line color
pub const CONNECTION_LINE: Color = Color::srgba(1.0, 1.0, 1.0, 0.2);

// ---------------------------------------------------------------------------
// Entity category colors (hierarchy icons)
// ---------------------------------------------------------------------------

/// Camera entity dot color (blue)
pub const CATEGORY_CAMERA: Color = Color::srgba(0.286, 0.506, 0.710, 1.0);
/// Light entity dot color (yellow)
pub const CATEGORY_LIGHT: Color = Color::srgba(1.0, 0.882, 0.0, 1.0);
/// Mesh entity dot color (orange/brown)
pub const CATEGORY_MESH: Color = Color::srgba(0.710, 0.537, 0.294, 1.0);
/// Scene root dot color (cyan)
pub const CATEGORY_SCENE: Color = Color::srgba(0.0, 0.667, 0.733, 1.0);
/// Generic entity dot color (green)
pub const CATEGORY_ENTITY: Color = Color::srgba(0.259, 0.725, 0.514, 1.0);

// ---------------------------------------------------------------------------
// Text colors
// ---------------------------------------------------------------------------

/// Primary text (#E6E6E6)
pub const TEXT_PRIMARY: Color = Color::srgb(0.902, 0.902, 0.902);
/// Secondary / dimmed text (#A4A4A6)
pub const TEXT_SECONDARY: Color = Color::srgb(0.643, 0.643, 0.651);
/// Tertiary text — breadcrumbs, field values (#CDCBCB)
pub const TEXT_TERTIARY: Color = Color::srgb(0.804, 0.796, 0.796);
/// Accent / link text
pub const TEXT_ACCENT: Color = Color::Srgba(tailwind::BLUE_400);
/// Accent hover — lighter blue
pub const TEXT_ACCENT_HOVER: Color = Color::Srgba(tailwind::BLUE_300);
/// Body text color (widget standard)
pub const TEXT_BODY_COLOR: Srgba = tailwind::ZINC_200;
/// Display text color (bright)
pub const TEXT_DISPLAY_COLOR: Srgba = tailwind::ZINC_50;
/// Muted text color
pub const TEXT_MUTED_COLOR: Srgba = tailwind::ZINC_400;

// ---------------------------------------------------------------------------
// Type-specific field label colors
// ---------------------------------------------------------------------------

/// Numeric (f32/f64/int) field label — green tint
pub const TYPE_NUMERIC: Color = Color::srgb(0.55, 0.78, 0.55);
/// Boolean field label — blue tint
pub const TYPE_BOOL: Color = Color::srgb(0.55, 0.65, 0.85);
/// String field label — orange tint
pub const TYPE_STRING: Color = Color::srgb(0.85, 0.70, 0.45);
/// Entity reference field label — white
pub const TYPE_ENTITY: Color = Color::Srgba(tailwind::ZINC_300);
/// Enum field label — purple tint
pub const TYPE_ENUM: Color = Color::srgb(0.72, 0.55, 0.82);

// ---------------------------------------------------------------------------
// XYZ axis colors (for vector field labels)
// ---------------------------------------------------------------------------

/// X axis color — red (#CC4341)
pub const AXIS_X_COLOR: Color = Color::srgb(0.800, 0.263, 0.255);
/// Y axis color — green (#269C33)
pub const AXIS_Y_COLOR: Color = Color::srgb(0.149, 0.612, 0.200);
/// Z axis color — blue (#0072D7)
pub const AXIS_Z_COLOR: Color = Color::srgb(0.0, 0.447, 0.843);

// ---------------------------------------------------------------------------
// File browser icon colors
// ---------------------------------------------------------------------------

/// Directory icon — warm yellow
pub const DIR_ICON_COLOR: Color = Color::srgb(0.9, 0.8, 0.3);
/// Generic file icon — grey
pub const FILE_ICON_COLOR: Color = Color::Srgba(tailwind::ZINC_400);

// ---------------------------------------------------------------------------
// Typography
// ---------------------------------------------------------------------------

pub const TEXT_SIZE_SM: f32 = 11.0;
pub const TEXT_SIZE: f32 = 13.0;
pub const TEXT_SIZE_LG: f32 = 13.0;
pub const TEXT_SIZE_XL: f32 = 18.0;

// Keep old names as aliases for existing code
pub const FONT_SM: f32 = TEXT_SIZE_SM;
pub const FONT_MD: f32 = TEXT_SIZE;
pub const FONT_LG: f32 = TEXT_SIZE_LG;
pub const ICON_LG: f32 = 24.0;

// ---------------------------------------------------------------------------
// Spacing
// ---------------------------------------------------------------------------

pub const SPACING_XS: f32 = 2.0;
pub const SPACING_SM: f32 = 4.0;
pub const SPACING_MD: f32 = 8.0;
pub const SPACING_LG: f32 = 12.0;

// ---------------------------------------------------------------------------
// Layout dimensions
// ---------------------------------------------------------------------------

pub const ROW_HEIGHT: f32 = 24.0;
pub const HEADER_HEIGHT: f32 = 28.0;
pub const STATUS_BAR_HEIGHT: f32 = 22.0;
pub const MENU_BAR_HEIGHT: f32 = 28.0;
pub const INPUT_HEIGHT: f32 = 28.0;

/// Panel tab bar height (Figma: 30px)
pub const PANEL_TAB_HEIGHT: f32 = 30.0;
/// Gap between panels in the layout (Figma: 4px)
pub const PANEL_GAP: f32 = 4.0;
/// Component card corner radius (Figma: 5px)
pub const COMPONENT_CARD_RADIUS: f32 = 5.0;

// ---------------------------------------------------------------------------
// Border radii (numeric)
// ---------------------------------------------------------------------------

pub const BORDER_RADIUS_SM: f32 = 3.0;
pub const BORDER_RADIUS_MD: f32 = 4.0;
pub const BORDER_RADIUS_LG: f32 = 5.0;
