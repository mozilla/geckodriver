use mozprofile::preferences::Pref;

lazy_static! {
    pub static ref DEFAULT: [(&'static str, Pref); 18] = [
        // Disable automatic downloading of new releases.
        ("app.update.auto", Pref::new(false)),

        // Disable automatically upgrading Firefox.
        ("app.update.enabled", Pref::new(false)),

        // Do not show the EULA notification.
        ("browser.EULA.override", Pref::new(true)),

        // Turn off about:newtab and make use of about:blank instead
        // for new opened tabs.
        ("browser.newtabpage.enabled", Pref::new(false)),

        // Never start the browser in offline mode.
        ("browser.offline", Pref::new(false)),

        // Disable safebrowsing components.
        ("browser.safebrowsing.blockedURIs.enabled", Pref::new(false)),
        ("browser.safebrowsing.downloads.enabled", Pref::new(false)),
        ("browser.safebrowsing.enabled", Pref::new(false)),
        ("browser.safebrowsing.forbiddenURIs.enabled", Pref::new(false)),
        ("browser.safebrowsing.malware.enabled", Pref::new(false)),
        ("browser.safebrowsing.phishing.enabled", Pref::new(false)),

        // Disable updates to search engines.
        ("browser.search.update", Pref::new(false)),

        // Don't check for the default web browser during startup.
        ("browser.shell.checkDefaultBrowser", Pref::new(false)),

        // Disable the UI tour
        ("browser.uitour.enabled", Pref::new(false)),

        // Only load extensions from the application and user profile
        // AddonManager.SCOPE_PROFILE + AddonManager.SCOPE_APPLICATION
        ("extensions.autoDisableScopes", Pref::new(0)),
        ("extensions.enabledScopes", Pref::new(5)),

        // Disable intalling any distribution extensions or add-ons
        ("extensions.installDistroAddons", Pref::new(false)),
        ("extensions.showMismatchUI", Pref::new(false)),
    ];
}
