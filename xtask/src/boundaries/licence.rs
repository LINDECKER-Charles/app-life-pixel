use super::{Member, Violation, Workspace};

const MIT: &str = "MIT";
const AGPL: &str = "AGPL-3.0-only";
const FORMAT: &str = "life-pixel-format";
const PLAYER: &str = "life-pixel-player";

/// The MIT crates, each with the only dependencies it may have.
const MIT_CRATES: [(&str, &[&str]); 2] = [(FORMAT, &[]), (PLAYER, &[FORMAT])];

/// Every broken boundary of `workspace`: `life-pixel-format` has no dependency,
/// `life-pixel-player` depends on `life-pixel-format` only, both declare MIT, every other member
/// declares AGPL-3.0-only, and `player-js` declares MIT.
pub fn check(workspace: &Workspace) -> Vec<Violation> {
    let members = workspace.members.iter().flat_map(check_member);
    let player_js = workspace
        .player_js
        .iter()
        .filter_map(|member| check_license(member, MIT));
    members.chain(player_js).collect()
}

fn check_member(member: &Member) -> Vec<Violation> {
    let allowed = MIT_CRATES.iter().find(|(name, _)| *name == member.name);
    let expected = if allowed.is_some() { MIT } else { AGPL };
    let forbidden = allowed.into_iter().flat_map(|(_, allowed)| {
        member
            .dependencies
            .iter()
            .filter(|dependency| !allowed.contains(&dependency.as_str()))
    });
    let forbidden = forbidden.map(|dependency| Violation::ForbiddenDependency {
        package: member.name.clone(),
        dependency: dependency.clone(),
    });
    check_license(member, expected)
        .into_iter()
        .chain(forbidden)
        .collect()
}

fn check_license(member: &Member, expected: &'static str) -> Option<Violation> {
    let is_expected = member.license.as_deref() == Some(expected);
    let found = member.license.clone();
    (!is_expected).then(|| Violation::WrongLicense {
        package: member.name.clone(),
        expected,
        found,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn member(name: &str, license: &str, dependencies: &[&str]) -> Member {
        Member {
            name: name.to_owned(),
            license: Some(license.to_owned()),
            dependencies: dependencies
                .iter()
                .map(|&dependency| dependency.to_owned())
                .collect(),
        }
    }

    fn workspace(members: Vec<Member>) -> Workspace {
        Workspace {
            members,
            player_js: Some(member("player-js/package.json", MIT, &[])),
        }
    }

    fn sound_members() -> Vec<Member> {
        vec![
            member(FORMAT, MIT, &[]),
            member(PLAYER, MIT, &[FORMAT]),
            member("life-pixel-core", AGPL, &["serde"]),
            member("xtask", AGPL, &["anyhow", FORMAT]),
        ]
    }

    #[test]
    fn a_sound_workspace_passes() {
        assert_eq!(check(&workspace(sound_members())), []);
    }

    #[test]
    fn an_mit_crate_under_another_licence_fails() {
        let mut members = sound_members();
        members[0] = member(FORMAT, AGPL, &[]);

        let violations = check(&workspace(members));

        let found = Some(AGPL.to_owned());
        let package = FORMAT.to_owned();
        assert_eq!(
            violations,
            [Violation::WrongLicense {
                package,
                expected: MIT,
                found
            }]
        );
    }

    #[test]
    fn an_agpl_crate_under_another_licence_or_none_fails() {
        let mut members = sound_members();
        members[2].license = Some(MIT.to_owned());
        members[3].license = None;

        let violations = check(&workspace(members));

        assert_eq!(
            violations,
            [
                Violation::WrongLicense {
                    package: "life-pixel-core".to_owned(),
                    expected: AGPL,
                    found: Some(MIT.to_owned()),
                },
                Violation::WrongLicense {
                    package: "xtask".to_owned(),
                    expected: AGPL,
                    found: None
                },
            ]
        );
    }

    #[test]
    fn the_format_crate_with_any_dependency_fails() {
        let mut members = sound_members();
        members[0].dependencies.push("serde".to_owned());

        let violations = check(&workspace(members));

        let package = FORMAT.to_owned();
        let dependency = "serde".to_owned();
        assert_eq!(
            violations,
            [Violation::ForbiddenDependency {
                package,
                dependency
            }]
        );
    }

    #[test]
    fn the_player_depending_on_more_than_the_format_fails() {
        let mut members = sound_members();
        members[1].dependencies.push("life-pixel-core".to_owned());

        let violations = check(&workspace(members));

        let package = PLAYER.to_owned();
        let dependency = "life-pixel-core".to_owned();
        assert_eq!(
            violations,
            [Violation::ForbiddenDependency {
                package,
                dependency
            }]
        );
    }

    #[test]
    fn player_js_under_another_licence_fails() {
        let mut checked = workspace(sound_members());
        checked.player_js = Some(member("player-js/package.json", AGPL, &[]));

        let violations = check(&checked);

        let package = "player-js/package.json".to_owned();
        let found = Some(AGPL.to_owned());
        assert_eq!(
            violations,
            [Violation::WrongLicense {
                package,
                expected: MIT,
                found
            }]
        );
    }

    #[test]
    fn a_workspace_without_player_js_passes() {
        let checked = Workspace {
            members: sound_members(),
            player_js: None,
        };

        assert_eq!(check(&checked), []);
    }
}
