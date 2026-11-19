use std::fs;
use std::path::{Path, PathBuf};

use zed_extension_api::{
    self as zed,
    http_client::{fetch, HttpMethod, HttpRequest},
    settings::LspSettings,
    LanguageServerId, LanguageServerInstallationStatus, Result,
};

const SERVER_NAME: &str = "vb-ls";
const NUGET_INDEX: &str = "https://api.nuget.org/v3-flatcontainer/vb-ls/index.json";
const DEFAULT_ARGS: &[&str] = &["--stdio", "--autoLoadProjects"];

struct VbNetExtension {
    cached_dll: Option<PathBuf>,
}

impl VbNetExtension {
    fn default_args() -> Vec<String> {
        DEFAULT_ARGS.iter().map(|s| s.to_string()).collect()
    }

    /// Finds `tools/<tfm>/any/vb-ls.dll` inside an extracted vb-ls NuGet package.
    fn find_dll(package_dir: &Path) -> Option<PathBuf> {
        let tools = package_dir.join("tools");
        fs::read_dir(&tools)
            .ok()?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path().join("any").join("vb-ls.dll"))
            .find(|dll| dll.is_file())
    }

    fn latest_version() -> Result<String> {
        let request = HttpRequest::builder()
            .method(HttpMethod::Get)
            .url(NUGET_INDEX)
            .build()?;
        let response = fetch(&request)?;
        let index: serde_json::Value = serde_json::from_slice(&response.body)
            .map_err(|e| format!("invalid NuGet index for {SERVER_NAME}: {e}"))?;
        index["versions"]
            .as_array()
            .and_then(|versions| {
                versions
                    .iter()
                    .filter_map(|v| v.as_str())
                    .filter(|v| !v.contains('-'))
                    .last()
            })
            .map(str::to_string)
            .ok_or_else(|| format!("no published versions of {SERVER_NAME} found on NuGet"))
    }

    /// Returns a previously downloaded vb-ls, used when NuGet is unreachable.
    fn any_downloaded_dll() -> Option<PathBuf> {
        let mut dirs: Vec<PathBuf> = fs::read_dir(".")
            .ok()?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("vb-ls-"))
            })
            .collect();
        dirs.sort();
        dirs.iter().rev().find_map(|dir| Self::find_dll(dir))
    }

    fn ensure_downloaded(&mut self, id: &LanguageServerId) -> Result<PathBuf> {
        if let Some(dll) = &self.cached_dll {
            if dll.is_file() {
                return Ok(dll.clone());
            }
        }

        zed::set_language_server_installation_status(
            id,
            &LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let version = match Self::latest_version() {
            Ok(version) => version,
            Err(err) => {
                if let Some(dll) = Self::any_downloaded_dll() {
                    self.cached_dll = Some(dll.clone());
                    return Ok(dll);
                }
                return Err(err);
            }
        };

        let package_dir = PathBuf::from(format!("{SERVER_NAME}-{version}"));
        if Self::find_dll(&package_dir).is_none() {
            zed::set_language_server_installation_status(
                id,
                &LanguageServerInstallationStatus::Downloading,
            );
            let url = format!(
                "https://api.nuget.org/v3-flatcontainer/vb-ls/{version}/vb-ls.{version}.nupkg"
            );
            // A .nupkg is a zip archive.
            zed::download_file(
                &url,
                package_dir.to_str().unwrap_or_default(),
                zed::DownloadedFileType::Zip,
            )
            .map_err(|e| format!("failed to download {SERVER_NAME} {version}: {e}"))?;

            // Remove older downloaded versions.
            if let Ok(entries) = fs::read_dir(".") {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    if name.starts_with("vb-ls-") && *name != *package_dir.to_string_lossy() {
                        fs::remove_dir_all(entry.path()).ok();
                    }
                }
            }
        }

        let dll = Self::find_dll(&package_dir)
            .ok_or_else(|| format!("vb-ls.dll not found in {SERVER_NAME} {version} package"))?;
        let dll = std::env::current_dir()
            .map_err(|e| e.to_string())?
            .join(dll);
        self.cached_dll = Some(dll.clone());
        Ok(dll)
    }
}

impl zed::Extension for VbNetExtension {
    fn new() -> Self {
        Self { cached_dll: None }
    }

    fn language_server_command(
        &mut self,
        id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let env = worktree.shell_env();
        let binary = LspSettings::for_worktree(SERVER_NAME, worktree)
            .ok()
            .and_then(|settings| settings.binary);
        let user_args = binary.as_ref().and_then(|b| b.arguments.clone());

        // 1. An explicit path in settings: lsp."vb-ls".binary.path
        if let Some(path) = binary.and_then(|b| b.path) {
            return Ok(zed::Command {
                command: path,
                args: user_args.unwrap_or_else(Self::default_args),
                env,
            });
        }

        // 2. vb-ls installed as a dotnet tool and on PATH.
        if let Some(path) = worktree.which(SERVER_NAME) {
            return Ok(zed::Command {
                command: path,
                args: user_args.unwrap_or_else(Self::default_args),
                env,
            });
        }

        // 3. Download vb-ls from NuGet and run it with the local .NET SDK.
        let dotnet = worktree.which("dotnet").ok_or_else(|| {
            "VB.NET: `dotnet` was not found on PATH. vb-ls needs the .NET 10 SDK \
             (https://dotnet.microsoft.com/download)."
                .to_string()
        })?;
        let dll = match self.ensure_downloaded(id) {
            Ok(dll) => dll,
            Err(err) => {
                zed::set_language_server_installation_status(
                    id,
                    &LanguageServerInstallationStatus::Failed(err.clone()),
                );
                return Err(err);
            }
        };
        zed::set_language_server_installation_status(id, &LanguageServerInstallationStatus::None);

        let mut args = vec![dll.to_string_lossy().into_owned()];
        args.extend(user_args.unwrap_or_else(Self::default_args));
        Ok(zed::Command {
            command: dotnet,
            args,
            env,
        })
    }

    fn language_server_initialization_options(
        &mut self,
        _id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        Ok(LspSettings::for_worktree(SERVER_NAME, worktree)
            .ok()
            .and_then(|s| s.initialization_options))
    }

    fn language_server_workspace_configuration(
        &mut self,
        _id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        Ok(LspSettings::for_worktree(SERVER_NAME, worktree)
            .ok()
            .and_then(|s| s.settings))
    }
}

zed::register_extension!(VbNetExtension);
