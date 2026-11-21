//! Command-line parsing, compatible with Fishstrap's flags.

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Mode {
    /// No recognised action: show the launch menu / settings.
    #[default]
    Menu,
    Settings,
    Player,
    Uninstall,
}

#[derive(Debug, Clone, Default)]
pub struct LaunchArgs {
    pub mode: Mode,
    /// Raw `roblox-player:` URI (or other data) handed to RobloxPlayerBeta.exe.
    pub roblox_args: String,
    pub channel: Option<String>,
    pub version: Option<String>,
    pub quiet: bool,
    pub no_launch: bool,
    pub force: bool,
}

impl LaunchArgs {
    pub fn parse(args: &[String]) -> Self {
        let mut a = LaunchArgs::default();
        let mut i = 0;
        if let Some(first) = args.first() {
            let lower = first.to_ascii_lowercase();
            if lower.starts_with("roblox:") || lower.starts_with("roblox-player:") {
                a.mode = Mode::Player;
                a.roblox_args = first.clone();
                i = 1;
            } else if lower.starts_with("version-") {
                a.mode = Mode::Player;
                a.version = Some(first.clone());
                i = 1;
            }
        }
        while i < args.len() {
            let flag = args[i].to_ascii_lowercase();
            // A flag's data is the next argument if it isn't itself a flag.
            let data = args.get(i + 1).filter(|d| !d.starts_with('-')).cloned();
            let mut take = || {
                if data.is_some() {
                    i += 1;
                }
                data.clone()
            };
            match flag.as_str() {
                "-player" => {
                    a.mode = Mode::Player;
                    if let Some(d) = take() {
                        a.roblox_args = d;
                    }
                }
                "-channel" => a.channel = take().filter(|s| !s.is_empty()),
                "-version" => {
                    a.version = take().filter(|s| !s.is_empty());
                    if a.version.is_some() && a.mode == Mode::Menu {
                        a.mode = Mode::Player;
                    }
                }
                "-settings" | "-preferences" | "-menu" if a.mode != Mode::Player => a.mode = Mode::Settings,
                "-uninstall" => a.mode = Mode::Uninstall,
                "-quiet" => a.quiet = true,
                "-nolaunch" => a.no_launch = true,
                "-force" => a.force = true,
                _ => {}
            }
            i += 1;
        }
        a
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn p(s: &[&str]) -> LaunchArgs {
        LaunchArgs::parse(&s.iter().map(|s| s.to_string()).collect::<Vec<_>>())
    }

    #[test]
    fn protocol_positional() {
        let a = p(&["roblox-player:1+launchmode:play+channel:zcanary"]);
        assert_eq!(a.mode, Mode::Player);
        assert!(a.roblox_args.contains("channel:zcanary"));
    }

    #[test]
    fn player_flag_with_data() {
        let a = p(&["-player", "roblox://experiences/start?placeId=1", "-channel", "zlive", "-quiet"]);
        assert_eq!(a.mode, Mode::Player);
        assert_eq!(a.roblox_args, "roblox://experiences/start?placeId=1");
        assert_eq!(a.channel.as_deref(), Some("zlive"));
        assert!(a.quiet);
    }

    #[test]
    fn bare_and_settings() {
        assert_eq!(p(&[]).mode, Mode::Menu);
        assert_eq!(p(&["-settings"]).mode, Mode::Settings);
        assert_eq!(p(&["-player"]).roblox_args, "");
        assert_eq!(p(&["-uninstall", "-quiet"]).mode, Mode::Uninstall);
    }
}
