/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::android::AndroidHandler;
use crate::capabilities::{FirefoxOptions, ProfileType};
use crate::logging;
use crate::prefs;
use mozprofile::preferences::Pref;
use mozprofile::profile::{PrefFile, Profile};
use mozrunner::runner::{FirefoxProcess, FirefoxRunner, Runner, RunnerProcess};
use std::env;
use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::time;
use webdriver::error::{ErrorStatus, WebDriverError, WebDriverResult};

// Status of the browser process.
pub(crate) enum BrowserStatus {
    Exited(Option<i32>),
    Running,
}

/// A running Gecko instance.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum Browser {
    Local(LocalBrowser),
    Remote(RemoteBrowser),

    /// An existing browser instance not controlled by GeckoDriver
    Existing(u16),
}

impl Browser {
    pub(crate) fn close(self, wait_for_shutdown: bool) -> WebDriverResult<()> {
        match self {
            Browser::Local(x) => x.close(wait_for_shutdown),
            Browser::Remote(x) => x.close(wait_for_shutdown),
            Browser::Existing(_) => Ok(()),
        }
    }

    pub(crate) fn marionette_port(&mut self) -> WebDriverResult<Option<u16>> {
        match self {
            Browser::Local(x) => x.marionette_port(),
            Browser::Remote(x) => x.marionette_port(),
            Browser::Existing(x) => Ok(Some(*x)),
        }
    }

    pub(crate) fn check_status(&mut self) -> Option<(u32, BrowserStatus)> {
        match self {
            Browser::Local(x) => Some(x.check_status()),
            Browser::Remote(x) => Some(x.check_status()),
            Browser::Existing(_) => None,
        }
    }

    pub(crate) fn update_marionette_port(&mut self, port: u16) {
        match self {
            Browser::Local(x) => x.update_marionette_port(port),
            Browser::Remote(x) => x.update_marionette_port(port),
            Browser::Existing(x) => {
                if port != *x {
                    error!(
                        "Cannot re-assign Marionette port when connected to an existing browser"
                    );
                }
            }
        }
    }
}

#[derive(Debug)]
/// A local Firefox process, running on this (host) device.
pub(crate) struct LocalBrowser {
    marionette_port: u16,
    prefs_backup: Option<PrefsBackup>,
    process: FirefoxProcess,
    pub(crate) profile_path: Option<PathBuf>,
}

impl LocalBrowser {
    pub(crate) fn new(
        options: FirefoxOptions,
        marionette_port: u16,
        jsdebugger: bool,
        system_access: bool,
        profile_root: Option<&Path>,
    ) -> WebDriverResult<LocalBrowser> {
        let binary = options.binary.ok_or_else(|| {
            WebDriverError::new(
                ErrorStatus::SessionNotCreated,
                "Expected browser binary location, but unable to find \
             binary in default location, no \
             'moz:firefoxOptions.binary' capability provided, and \
             no binary flag set on the command line",
            )
        })?;

        let is_custom_profile = matches!(options.profile, ProfileType::Path(_));

        let mut profile = match options.profile {
            ProfileType::Named => None,
            ProfileType::Path(x) => Some(x),
            ProfileType::Temporary => Some(Profile::new(profile_root)?),
        };

        let (profile_path, prefs_backup) = if let Some(ref mut profile) = profile {
            let profile_path = profile.path.clone();
            let prefs_backup = set_prefs(
                marionette_port,
                profile,
                is_custom_profile,
                options.prefs,
                jsdebugger,
            )
            .map_err(|e| {
                WebDriverError::new(
                    ErrorStatus::SessionNotCreated,
                    format!("Failed to set preferences: {}", e),
                )
            })?;
            (Some(profile_path), prefs_backup)
        } else {
            warn!("Unable to set geckodriver prefs when using a named profile");
            (None, None)
        };

        let mut runner = FirefoxRunner::new(&binary, profile);

        runner.arg("--marionette");
        if jsdebugger {
            runner.arg("--jsdebugger");
        }
        if system_access {
            runner.arg("--remote-allow-system-access");
        }
        if let Some(args) = options.args.as_ref() {
            runner.args(args);
        }

        // https://developer.mozilla.org/docs/Environment_variables_affecting_crash_reporting
        runner
            .env("MOZ_CRASHREPORTER", "1")
            .env("MOZ_CRASHREPORTER_NO_REPORT", "1")
            .env("MOZ_CRASHREPORTER_SHUTDOWN", "1");

        let process = match runner.start() {
            Ok(process) => process,
            Err(e) => {
                if let Some(backup) = prefs_backup {
                    backup.restore();
                }
                return Err(WebDriverError::new(
                    ErrorStatus::SessionNotCreated,
                    format!("Failed to start browser {}: {}", binary.display(), e),
                ));
            }
        };

        Ok(LocalBrowser {
            marionette_port,
            prefs_backup,
            process,
            profile_path,
        })
    }

    fn close(mut self, wait_for_shutdown: bool) -> WebDriverResult<()> {
        if wait_for_shutdown {
            // TODO(https://bugzil.la/1443922):
            // Use toolkit.asyncshutdown.crash_timout pref
            let duration = time::Duration::from_secs(70);
            match self.process.wait(duration) {
                Ok(x) => debug!("Browser process stopped: {}", x),
                Err(e) => error!("Failed to stop browser process: {}", e),
            }
        }
        self.process.kill()?;
        Ok(())
    }

    fn marionette_port(&mut self) -> WebDriverResult<Option<u16>> {
        if self.marionette_port != 0 {
            return Ok(Some(self.marionette_port));
        }

        if let Some(profile_path) = self.profile_path.as_ref() {
            return Ok(read_marionette_port(profile_path));
        }

        // This should be impossible, but it isn't enforced
        Err(WebDriverError::new(
            ErrorStatus::SessionNotCreated,
            "Port not known when using named profile",
        ))
    }

    fn update_marionette_port(&mut self, port: u16) {
        self.marionette_port = port;
    }

    pub(crate) fn check_status(&mut self) -> (u32, BrowserStatus) {
        let pid = self.process.pid();
        let status = match self.process.try_wait() {
            Ok(Some(exit_status)) => BrowserStatus::Exited(exit_status.code()),
            Ok(None) => BrowserStatus::Running,
            Err(_) => BrowserStatus::Exited(None),
        };
        (pid, status)
    }
}

impl Drop for LocalBrowser {
    fn drop(&mut self) {
        if let Some(prefs_backup) = self.prefs_backup.take() {
            debug!("Restore user preferences");
            prefs_backup.restore();
        }

        if let Some(profile_path) = &self.profile_path {
            // Save minidump files of potential crashes from the profile if requested.
            if let Err(e) = copy_minidumps_files(profile_path) {
                error!(
                    "Failed to save crash minidumps to the specified location: {}",
                    e
                );
            }
        }
    }
}

fn read_marionette_port(profile_path: &Path) -> Option<u16> {
    let port_file = profile_path.join("MarionetteActivePort");
    let mut port_str = String::with_capacity(6);
    let mut file = match fs::File::open(&port_file) {
        Ok(file) => file,
        Err(_) => {
            trace!("Failed to open {}", &port_file.to_string_lossy());
            return None;
        }
    };
    if let Err(e) = file.read_to_string(&mut port_str) {
        trace!("Failed to read {}: {}", &port_file.to_string_lossy(), e);
        return None;
    };
    println!("Read port: {}", port_str);
    let port = port_str.parse::<u16>().ok();
    if port.is_none() {
        warn!("Failed fo convert {} to u16", &port_str);
    }
    port
}

#[derive(Debug)]
/// A remote instance, running on a (target) Android device.
pub(crate) struct RemoteBrowser {
    pub(crate) handler: AndroidHandler,
    marionette_port: u16,
    pid: u32,
    prefs_backup: Option<PrefsBackup>,
}

impl RemoteBrowser {
    pub(crate) fn new(
        options: FirefoxOptions,
        marionette_port: u16,
        websocket_port: Option<u16>,
        system_access: bool,
        profile_root: Option<&Path>,
    ) -> WebDriverResult<RemoteBrowser> {
        let android_options = options.android.unwrap();

        let handler = AndroidHandler::new(
            &android_options,
            marionette_port,
            system_access,
            websocket_port,
        )?;

        // Profile management.
        let (mut profile, is_custom_profile) = match options.profile {
            ProfileType::Named => {
                return Err(WebDriverError::new(
                    ErrorStatus::SessionNotCreated,
                    "Cannot use a named profile on Android",
                ));
            }
            ProfileType::Path(x) => (x, true),
            ProfileType::Temporary => (Profile::new(profile_root)?, false),
        };

        let prefs_backup = set_prefs(
            handler.marionette_target_port,
            &mut profile,
            is_custom_profile,
            options.prefs,
            false,
        )
        .map_err(|e| {
            WebDriverError::new(
                ErrorStatus::SessionNotCreated,
                format!("Failed to set preferences: {}", e),
            )
        })?;

        handler.prepare(&profile, options.args, options.env.unwrap_or_default())?;

        let pid = handler.launch()?;

        Ok(RemoteBrowser {
            handler,
            marionette_port,
            pid,
            prefs_backup,
        })
    }

    fn close(&self, wait_for_shutdown: bool) -> WebDriverResult<()> {
        if wait_for_shutdown {
            // TODO(https://bugzil.la/1443922):
            // Use toolkit.asyncshutdown.crash_timeout pref
            let timeout = time::Duration::from_secs(70);
            let poll_interval = time::Duration::from_millis(100);
            let start = time::Instant::now();

            debug!(
                "Waiting {}s for Android process {} (package {}) to exit",
                timeout.as_secs(),
                self.pid,
                &self.handler.process.package
            );

            loop {
                let (_, status) = self.check_status();
                if matches!(status, BrowserStatus::Exited(_)) {
                    debug!(
                        "Android package {} has exited",
                        &self.handler.process.package
                    );
                    break;
                }

                if start.elapsed() >= timeout {
                    warn!(
                        "Timed out waiting for Android package {} to exit",
                        &self.handler.process.package
                    );
                    break;
                }

                std::thread::sleep(poll_interval);
            }
        }
        self.handler.force_stop()?;
        Ok(())
    }

    fn marionette_port(&mut self) -> WebDriverResult<Option<u16>> {
        Ok(Some(self.marionette_port))
    }

    fn update_marionette_port(&mut self, port: u16) {
        self.marionette_port = port;
    }

    pub(crate) fn check_status(&self) -> (u32, BrowserStatus) {
        let command = format!("kill -0 {} 2>/dev/null; echo $?", self.pid);
        let status = match self
            .handler
            .process
            .device
            .execute_host_shell_command(&command)
        {
            Ok(output) if output.trim() != "0" => BrowserStatus::Exited(None),
            Err(e) => {
                warn!("Failed to check browser status via adb: {}", e);
                BrowserStatus::Running
            }
            _ => BrowserStatus::Running,
        };
        (self.pid, status)
    }
}

impl Drop for RemoteBrowser {
    fn drop(&mut self) {
        // Restore preferences which had custom values set.
        if let Some(prefs_backup) = self.prefs_backup.take() {
            prefs_backup.restore();
        }

        // Save minidump files of potential crashes from the profile if requested.
        if let Err(e) = self.handler.copy_minidumps_files() {
            error!(
                "Failed to save crash minidumps to the specified location: {}",
                e
            );
        }
    }
}

fn set_prefs(
    port: u16,
    profile: &mut Profile,
    custom_profile: bool,
    extra_prefs: Vec<(String, Pref)>,
    js_debugger: bool,
) -> WebDriverResult<Option<PrefsBackup>> {
    let prefs = profile.user_prefs().map_err(|_| {
        WebDriverError::new(
            ErrorStatus::UnknownError,
            "Unable to read profile preferences file",
        )
    })?;

    let backup_prefs = if custom_profile && prefs.path.exists() {
        Some(PrefsBackup::new(prefs)?)
    } else {
        None
    };

    for &(name, ref value) in prefs::DEFAULT.iter() {
        if !custom_profile || !prefs.contains_key(name) {
            prefs.insert(name.to_string(), (*value).clone());
        }
    }

    prefs.insert_slice(&extra_prefs[..]);

    if js_debugger {
        prefs.insert("devtools.browsertoolbox.panel", Pref::new("jsdebugger"));
        prefs.insert("devtools.debugger.remote-enabled", Pref::new(true));
        prefs.insert("devtools.chrome.enabled", Pref::new(true));
        prefs.insert("devtools.debugger.prompt-connection", Pref::new(false));
    }

    prefs.insert("marionette.port", Pref::new(port));
    prefs.insert("remote.log.level", logging::max_level().into());

    prefs.write().map_err(|e| {
        WebDriverError::new(
            ErrorStatus::UnknownError,
            format!("Unable to write Firefox profile: {}", e),
        )
    })?;
    Ok(backup_prefs)
}

#[derive(Debug)]
struct PrefsBackup {
    orig_path: PathBuf,
    backup_path: PathBuf,
}

impl PrefsBackup {
    fn new(prefs: &PrefFile) -> WebDriverResult<PrefsBackup> {
        let mut prefs_backup_path = prefs.path.clone();
        let mut counter = 0;
        while {
            let ext = if counter > 0 {
                format!("geckodriver_backup_{}", counter)
            } else {
                "geckodriver_backup".to_string()
            };
            prefs_backup_path.set_extension(ext);
            prefs_backup_path.exists()
        } {
            counter += 1
        }
        debug!("Backing up prefs to {:?}", prefs_backup_path);
        fs::copy(&prefs.path, &prefs_backup_path)?;

        Ok(PrefsBackup {
            orig_path: prefs.path.clone(),
            backup_path: prefs_backup_path,
        })
    }

    fn restore(self) {
        if self.backup_path.exists() {
            let _ = fs::rename(self.backup_path, self.orig_path);
        }
    }
}

fn copy_minidumps_files(profile_path: &Path) -> WebDriverResult<()> {
    let mut minidumps_path = profile_path.to_path_buf();
    minidumps_path.push("minidumps");

    match std::fs::read_dir(&minidumps_path) {
        Ok(entries) => {
            let save_path = match env::var("MINIDUMP_SAVE_PATH").map(PathBuf::from) {
                Ok(path) => path,
                Err(_) => {
                    debug!("Set MINIDUMP_SAVE_PATH to store crash minidumps.");
                    return Ok(());
                }
            };

            for result_entry in entries {
                let entry = result_entry?;
                let file_type = entry.file_type()?;

                if file_type.is_dir() {
                    continue;
                }

                let path = entry.path();
                let extension = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.to_lowercase())
                    .unwrap_or(String::from(""));

                // Copy only *.dmp and *.extra files.
                if extension == "dmp" || extension == "extra" {
                    let dest_path = save_path.join(entry.file_name());
                    fs::copy(path, &dest_path)?;

                    debug!(
                        "Copied minidump file {:?} to {:?}.",
                        entry.file_name(),
                        save_path.display()
                    );
                }
            }
        }
        Err(_) => {
            warn!(
                "Couldn't read files from minidumps folder '{}'",
                minidumps_path.display(),
            );

            return Ok(());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(unsafe_code)]

    use super::*;
    use crate::browser::read_marionette_port;
    use crate::capabilities::{FirefoxOptions, ProfileType};
    use base64::prelude::BASE64_STANDARD;
    use base64::Engine;
    use mozprofile::preferences::{Pref, PrefValue};
    use mozprofile::profile::Profile;
    use serde_json::{Map, Value};
    use std::fs::File;
    use std::io::{Read, Write};
    use std::path::Path;
    use std::sync::{LazyLock, Mutex, MutexGuard};
    use tempfile::TempDir;

    fn example_profile() -> Value {
        let mut profile_data = Vec::with_capacity(1024);
        let mut profile = File::open("src/tests/profile.zip").unwrap();
        profile.read_to_end(&mut profile_data).unwrap();
        Value::String(BASE64_STANDARD.encode(&profile_data))
    }

    // This is not a pretty test, mostly due to the nature of
    // mozprofile's and MarionetteHandler's APIs, but we have had
    // several regressions related to remote.log.level.
    #[test]
    fn test_remote_log_level() {
        let mut profile = Profile::new(None).unwrap();
        set_prefs(2828, &mut profile, false, vec![], false).ok();
        let user_prefs = profile.user_prefs().unwrap();

        let pref = user_prefs.get("remote.log.level").unwrap();
        let value = match pref.value {
            PrefValue::String(ref s) => s,
            _ => panic!(),
        };
        for (i, ch) in value.chars().enumerate() {
            if i == 0 {
                assert!(ch.is_uppercase());
            } else {
                assert!(ch.is_lowercase());
            }
        }
    }

    #[test]
    fn test_prefs() {
        let marionette_settings = Default::default();

        let encoded_profile = example_profile();
        let mut prefs: Map<String, Value> = Map::new();
        prefs.insert(
            "browser.display.background_color".into(),
            Value::String("#00ff00".into()),
        );

        let mut firefox_opts = Map::new();
        firefox_opts.insert("profile".into(), encoded_profile);
        firefox_opts.insert("prefs".into(), Value::Object(prefs));

        let mut caps = Map::new();
        caps.insert("moz:firefoxOptions".into(), Value::Object(firefox_opts));

        let opts = FirefoxOptions::from_capabilities(None, &marionette_settings, &mut caps)
            .expect("Valid profile and prefs");

        let mut profile = match opts.profile {
            ProfileType::Path(profile) => profile,
            _ => panic!("Expected ProfileType::Path"),
        };

        set_prefs(2828, &mut profile, true, opts.prefs, false).expect("set preferences");

        let prefs_set = profile.user_prefs().expect("valid user preferences");
        println!("{:#?}", prefs_set.prefs);

        assert_eq!(
            prefs_set.get("startup.homepage_welcome_url"),
            Some(&Pref::new("data:text/html,PASS"))
        );
        assert_eq!(
            prefs_set.get("browser.display.background_color"),
            Some(&Pref::new("#00ff00"))
        );
        assert_eq!(prefs_set.get("marionette.port"), Some(&Pref::new(2828)));
    }

    #[test]
    fn test_pref_backup() {
        let mut profile = Profile::new(None).unwrap();

        // Create some prefs in the profile
        let initial_prefs = profile.user_prefs().unwrap();
        initial_prefs.insert("geckodriver.example", Pref::new("example"));
        initial_prefs.write().unwrap();

        let prefs_path = initial_prefs.path.clone();

        let mut conflicting_backup_path = initial_prefs.path.clone();
        conflicting_backup_path.set_extension("geckodriver_backup");
        println!("{:?}", conflicting_backup_path);
        let mut file = File::create(&conflicting_backup_path).unwrap();
        file.write_all(b"test").unwrap();
        assert!(conflicting_backup_path.exists());

        let mut initial_prefs_data = String::new();
        File::open(&prefs_path)
            .expect("Initial prefs exist")
            .read_to_string(&mut initial_prefs_data)
            .unwrap();

        let backup = set_prefs(2828, &mut profile, true, vec![], false)
            .unwrap()
            .unwrap();
        let user_prefs = profile.user_prefs().unwrap();

        assert!(user_prefs.path.exists());
        let mut backup_path = user_prefs.path.clone();
        backup_path.set_extension("geckodriver_backup_1");

        assert!(backup_path.exists());

        // Ensure the actual prefs contain both the existing ones and the ones we added
        let pref = user_prefs.get("marionette.port").unwrap();
        assert_eq!(pref.value, PrefValue::Int(2828));

        let pref = user_prefs.get("geckodriver.example").unwrap();
        assert_eq!(pref.value, PrefValue::String("example".into()));

        // Ensure the backup prefs don't contain the new settings
        let mut backup_data = String::new();
        File::open(&backup_path)
            .expect("Backup prefs exist")
            .read_to_string(&mut backup_data)
            .unwrap();
        assert_eq!(backup_data, initial_prefs_data);

        backup.restore();

        assert!(!backup_path.exists());
        let mut final_prefs_data = String::new();
        File::open(&prefs_path)
            .expect("Initial prefs exist")
            .read_to_string(&mut final_prefs_data)
            .unwrap();
        assert_eq!(final_prefs_data, initial_prefs_data);
    }

    #[test]
    fn test_local_read_marionette_port() {
        fn create_port_file(profile_path: &Path, data: &[u8]) {
            let port_path = profile_path.join("MarionetteActivePort");
            let mut file = File::create(&port_path).unwrap();
            file.write_all(data).unwrap();
        }

        let profile_dir = TempDir::new().unwrap();
        let profile_path = profile_dir.path();
        assert_eq!(read_marionette_port(profile_path), None);
        assert_eq!(read_marionette_port(profile_path), None);
        create_port_file(profile_path, b"");
        assert_eq!(read_marionette_port(profile_path), None);
        create_port_file(profile_path, b"1234");
        assert_eq!(read_marionette_port(profile_path), Some(1234));
        create_port_file(profile_path, b"1234abc");
        assert_eq!(read_marionette_port(profile_path), None);
    }

    fn assert_minidump_files(minidumps_path: &Path, filename: &str) {
        let mut dmp_file_present = false;
        let mut extra_file_present = false;

        for result_entry in std::fs::read_dir(minidumps_path).unwrap() {
            let entry = result_entry.unwrap();

            let path: PathBuf = entry.path();
            let filename_from_path = path.file_stem().unwrap().to_str().unwrap();
            if filename == filename_from_path {
                let extension = path.extension().and_then(|ext| ext.to_str()).unwrap();

                if extension == "dmp" {
                    dmp_file_present = true;
                }

                if extension == "extra" {
                    extra_file_present = true;
                }
            }
        }

        assert!(dmp_file_present);
        assert!(extra_file_present);
    }

    static ENV_MUTEX: LazyLock<Mutex<()>> = LazyLock::new(Mutex::default);
    static MINIDUMP_KEY: &str = "MINIDUMP_SAVE_PATH";

    pub(crate) struct MinidumpEnvironment<'environment> {
        initial_environment: Option<String>,
        #[allow(dead_code)]
        guard: MutexGuard<'environment, ()>,
    }

    impl<'environment> MinidumpEnvironment<'environment> {
        pub(crate) fn new() -> MinidumpEnvironment<'environment> {
            MinidumpEnvironment {
                initial_environment: env::var(MINIDUMP_KEY).ok(),
                guard: ENV_MUTEX.lock().unwrap(),
            }
        }

        pub(crate) fn set(&self, value: Option<&str>) {
            fn set_env(key: &str, value: Option<&str>) {
                // SAFETY: Safe as long as no other threads try to modify the environment
                // This is enforced by Environment taking a mutex, so tests can't run
                // in parallel.
                unsafe {
                    match value {
                        Some(value) => env::set_var(key, value),
                        None => env::remove_var(key),
                    }
                }
            }
            set_env(MINIDUMP_KEY, value);
        }
    }

    impl Drop for MinidumpEnvironment<'_> {
        fn drop(&mut self) {
            self.set(self.initial_environment.clone().as_deref());
        }
    }

    fn create_file(folder: &Path, filename: &str) {
        let file = folder.join(filename);
        File::create(&file).unwrap();
    }

    fn create_minidump_files(profile_path: &Path, filename: &str) {
        let folder = create_minidump_folder(profile_path);

        let mut file_extensions = [".dmp", ".extra"];
        for file_extension in file_extensions.iter_mut() {
            let mut filename_with_extension: String = filename.to_owned();
            filename_with_extension.push_str(file_extension);

            create_file(&folder, &filename_with_extension);
        }
    }

    fn create_minidump_folder(profile_path: &Path) -> PathBuf {
        let minidumps_folder = profile_path.join("minidumps");
        if !minidumps_folder.is_dir() {
            fs::create_dir(&minidumps_folder).unwrap();
        }

        minidumps_folder
    }

    #[test]
    fn test_copy_minidumps() {
        let env = MinidumpEnvironment::new();

        let tmp_dir_profile = TempDir::new().unwrap();
        let profile_path = tmp_dir_profile.path();

        let filename = "test";
        create_minidump_files(profile_path, filename);

        let tmp_dir_minidumps = TempDir::new().unwrap();
        let minidumps_path = tmp_dir_minidumps.path();

        env.set(minidumps_path.to_str());
        assert!(copy_minidumps_files(profile_path).is_ok());

        assert_minidump_files(minidumps_path, filename);

        tmp_dir_profile.close().unwrap();
        tmp_dir_minidumps.close().unwrap();
    }

    #[test]
    fn test_copy_multiple_minidumps() {
        let env = MinidumpEnvironment::new();

        let tmp_dir_profile = TempDir::new().unwrap();
        let profile_path = tmp_dir_profile.path();

        let filename_1 = "test_1";
        create_minidump_files(profile_path, filename_1);

        let filename_2 = "test_2";
        create_minidump_files(profile_path, filename_2);

        let tmp_dir_minidumps = TempDir::new().unwrap();
        let minidumps_path = tmp_dir_minidumps.path();

        env.set(minidumps_path.to_str());
        assert!(copy_minidumps_files(profile_path).is_ok());

        assert_minidump_files(minidumps_path, filename_1);
        assert_minidump_files(minidumps_path, filename_1);

        tmp_dir_profile.close().unwrap();
        tmp_dir_minidumps.close().unwrap();
    }

    #[test]
    fn test_copy_minidumps_with_non_existent_manifest_path() {
        let env = MinidumpEnvironment::new();

        let tmp_dir_profile = TempDir::new().unwrap();
        let profile_path = tmp_dir_profile.path();

        create_minidump_folder(profile_path);

        env.set(Path::new("/non-existent").to_str());
        assert!(copy_minidumps_files(profile_path).is_ok());

        tmp_dir_profile.close().unwrap();
    }

    #[test]
    fn test_copy_minidumps_with_non_existent_profile_path() {
        let env = MinidumpEnvironment::new();

        let tmp_dir_profile = TempDir::new().unwrap();
        let profile_path = tmp_dir_profile.path();

        env.set(Path::new("/non-existent").to_str());
        assert!(copy_minidumps_files(profile_path).is_ok());

        tmp_dir_profile.close().unwrap();
    }

    #[test]
    fn test_copy_minidumps_with_no_minidump_files() {
        let env = MinidumpEnvironment::new();

        let tmp_dir_profile = TempDir::new().unwrap();
        let profile_path = tmp_dir_profile.path();

        let minidumps_folder = create_minidump_folder(profile_path);

        // Create a folder.
        let test_folder_binding = profile_path.join("test");
        let test_folder = test_folder_binding.as_path();
        fs::create_dir(test_folder).unwrap();

        // Create a file with non minidumps extension.
        create_file(&minidumps_folder, "test.txt");

        let tmp_dir_minidumps = TempDir::new().unwrap();
        let minidumps_path = tmp_dir_minidumps.path();

        env.set(minidumps_path.to_str());
        assert!(copy_minidumps_files(profile_path).is_ok());

        // Check that the non minidump file and the folder were not copied.
        assert!(minidumps_path.read_dir().unwrap().next().is_none());

        tmp_dir_profile.close().unwrap();
        tmp_dir_minidumps.close().unwrap();
    }
}
