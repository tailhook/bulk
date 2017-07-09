use std::cmp::min;
use version::{Version, Component};

#[derive(Debug, Clone, Copy)]
pub enum Bump {
    Patch,
    Minor,
    Major,
    Component(u8),
}

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        NonNumericComponent(val: String) {
            description("component we're trying to increment is non-numeric")
            display("component {:?} is non-numeric", val)
        }
    }
}

pub fn bump_version<T: AsRef<str>>(ver: &Version<T>, how: Bump)
    -> Result<Version<String>, Error>
{
    match how {
        Bump::Component(i) => {
            let mut result = Vec::new();
            let mut iter = ver.components();
            for n in iter.by_ref().take((i-1) as usize) {
                result.push(n);
            }
            while result.len() < (i-1) as usize {
                result.push(Component::Numeric(0));
            }
            let n: u64 = match iter.next() {
                Some(Component::Numeric(x)) => x+1,
                Some(Component::String(x)) => {
                    return Err(Error::NonNumericComponent(x.into()));
                },
                None => 1,
            };
            result.push(Component::Numeric(n));
            while result.len() < 3 {
                result.push(Component::Numeric(0));
            }
            let mut buf = format!("v{}", result[0]);
            for i in &result[1..] {
                use std::fmt::Write;
                write!(&mut buf, ".{}", i).unwrap();
            }
            Ok(Version(buf))
        }
        Bump::Major|Bump::Minor|Bump::Patch => {
            let idx = ver.components()
                .position(|x| matches!(x, Component::Numeric(y) if y > 0))
                // If version is v0.0.0 we consider major bump is v0.0.1
                .unwrap_or(2)
                +1; // 1-based index
            let cmp = match (idx, how) {
                (idx, Bump::Major) => min(idx, 3),
                (1, Bump::Minor) => 2,
                (_, _) => 3,
            };
            return bump_version(ver, Bump::Component(cmp as u8));
        }
    }
}

#[cfg(test)]
mod test {
    use super::bump_version;
    use super::Bump;
    use super::Bump::*;
    use version::Version;

    fn bump_component(ver: &str, bump: u8) -> String {
        format!("{}",
            bump_version(&Version(ver), Bump::Component(bump))
            .unwrap())
    }
    fn bump_semver(ver: &str, bump: Bump) -> String {
        format!("{}",
            bump_version(&Version(ver), bump)
            .unwrap())
    }
    #[test]
    fn test_cmp1() {
        assert_eq!(bump_component("v1.2.0", 1), "v2.0.0");
        assert_eq!(bump_component("v0.0.0", 1), "v1.0.0");
        assert_eq!(bump_component("v0", 1), "v1.0.0");
        assert_eq!(bump_component("v7.38.96", 1), "v8.0.0");
        assert_eq!(bump_component("v9.38.96", 1), "v10.0.0");
        assert_eq!(bump_component("v12.38.96", 1), "v13.0.0");
    }
    #[test]
    fn test_cmp2() {
        assert_eq!(bump_component("v1.2.0", 2), "v1.3.0");
        assert_eq!(bump_component("v0.0.0", 2), "v0.1.0");
        assert_eq!(bump_component("v0", 2), "v0.1.0");
        assert_eq!(bump_component("v7.38.96", 2), "v7.39.0");
        assert_eq!(bump_component("v7.9.96", 2), "v7.10.0");
        assert_eq!(bump_component("v12.38.96", 2), "v12.39.0");
    }
    #[test]
    fn test_cmp3() {
        assert_eq!(bump_component("v1.2.0", 3), "v1.2.1");
        assert_eq!(bump_component("v0.0.0", 3), "v0.0.1");
        assert_eq!(bump_component("v0", 3), "v0.0.1");
        assert_eq!(bump_component("v7.38.96", 3), "v7.38.97");
        assert_eq!(bump_component("v7.13.99", 3), "v7.13.100");
    }

    #[test]
    fn test_major() {
        assert_eq!(bump_semver("v1.2.0", Major), "v2.0.0");
        assert_eq!(bump_semver("v0.0.0", Major), "v0.0.1");
        assert_eq!(bump_semver("v0.0.99", Major), "v0.0.100");
        assert_eq!(bump_semver("v0", Major), "v0.0.1");
        assert_eq!(bump_semver("v7.38.96", Major), "v8.0.0");
        assert_eq!(bump_semver("v9.38.96", Major), "v10.0.0");
        assert_eq!(bump_semver("v12.38.96", Major), "v13.0.0");
        assert_eq!(bump_semver("v0.3.7", Major), "v0.4.0");
        assert_eq!(bump_semver("v0.9.17", Major), "v0.10.0");
    }
    #[test]
    fn test_minor() {
        assert_eq!(bump_semver("v1.2.0", Minor), "v1.3.0");
        assert_eq!(bump_semver("v0.0.0", Minor), "v0.0.1");
        assert_eq!(bump_semver("v0.0.99", Minor), "v0.0.100");
        assert_eq!(bump_semver("v0", Minor), "v0.0.1");
        assert_eq!(bump_semver("v7.38.96", Minor), "v7.39.0");
        assert_eq!(bump_semver("v9.38.96", Minor), "v9.39.0");
        assert_eq!(bump_semver("v12.38.96", Minor), "v12.39.0");
        assert_eq!(bump_semver("v0.3.7", Minor), "v0.3.8");
        assert_eq!(bump_semver("v0.9.17", Minor), "v0.9.18");
    }

    #[test]
    fn test_patch() {
        assert_eq!(bump_semver("v1.2.0", Patch), "v1.2.1");
        assert_eq!(bump_semver("v0.0.0", Patch), "v0.0.1");
        assert_eq!(bump_semver("v0.0.99", Patch), "v0.0.100");
        assert_eq!(bump_semver("v0", Patch), "v0.0.1");
        assert_eq!(bump_semver("v7.38.96", Patch), "v7.38.97");
        assert_eq!(bump_semver("v7.13.99", Patch), "v7.13.100");
    }
}
