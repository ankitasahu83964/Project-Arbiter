fn main() {
    slint_build::compile("ui/forge.slint").unwrap();

    // Embed ui/icon.ico as a Windows PE resource so the exe shows the correct
    // icon in Explorer, the taskbar, and the Alt+Tab switcher.
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../arbiter-data/icon.ico");
        res.set("FileDescription", "Arbiter Forge Control Interface");
        res.set("ProductName", "Project Arbiter");
        res.set("OriginalFilename", "arbiter-forge.exe");
        res.set("LegalCopyright", "Copyright (c) 2026");
        let pkg_ver =
            std::env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION must be set by Cargo");
        let parts: Vec<&str> = pkg_ver.split('.').collect();
        let major = parts.first().unwrap_or(&"2");
        let minor = parts.get(1).unwrap_or(&"1");
        let win_ver = format!("0.{major}.{minor}.0");

        res.set("ProductVersion", &win_ver);
        res.set("FileVersion", &win_ver);
        res.set_manifest(
            r#"
        <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
          <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
            <security>
              <requestedPrivileges>
                <requestedExecutionLevel level="asInvoker" uiAccess="false" />
              </requestedPrivileges>
            </security>
          </trustInfo>
        </assembly>
        "#,
        );
        res.compile()
            .expect("Failed to embed Windows icon resource");
    }
}
