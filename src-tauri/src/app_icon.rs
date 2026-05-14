/// Load the application icon at `app_path` and return a base64-encoded PNG.
/// Uses NSWorkspace.icon(forFile:) → TIFF → NSBitmapImageRep → PNG.
#[cfg(target_os = "macos")]
pub fn load_app_icon_base64(app_path: &str) -> anyhow::Result<String> {
    use anyhow::{anyhow, Context};
    use base64::Engine;
    use objc2_app_kit::{NSBitmapImageFileType, NSBitmapImageRep, NSWorkspace};
    use objc2_foundation::{NSDictionary, NSString};

    let path_ns = NSString::from_str(app_path);
    let workspace = unsafe { NSWorkspace::sharedWorkspace() };
    let image = unsafe { workspace.iconForFile(&path_ns) };
    let tiff = unsafe { image.TIFFRepresentation() }
        .ok_or_else(|| anyhow!("NSImage::TIFFRepresentation returned nil for {}", app_path))?;
    let rep = unsafe { NSBitmapImageRep::imageRepWithData(&tiff) }
        .ok_or_else(|| anyhow!("NSBitmapImageRep::imageRepWithData returned nil"))?;
    let props = NSDictionary::new();
    let png = unsafe { rep.representationUsingType_properties(NSBitmapImageFileType::PNG, &props) }
        .ok_or_else(|| anyhow!("representationUsingType:PNG returned nil"))?;
    let bytes = png.bytes();
    Ok::<String, anyhow::Error>(base64::engine::general_purpose::STANDARD.encode(bytes))
        .with_context(|| format!("encode icon for {}", app_path))
}

#[cfg(not(target_os = "macos"))]
pub fn load_app_icon_base64(_app_path: &str) -> anyhow::Result<String> {
    anyhow::bail!("app icon extraction unsupported on this platform")
}
