#[cfg(feature = "prometheus")]
pub use prometheus_impl::*;

pub trait MaybeDefault: Sized {
    fn default_opt() -> Option<Self>;
}

impl<T: Default> MaybeDefault for T {
    #[inline]
    fn default_opt() -> Option<Self> { Some(Self::default()) }
}

#[derive(Debug, clap::Args, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct VixenConfig<M: clap::Args> {
    #[command(flatten)]
    pub yellowstone: YellowstoneConfig,

    #[command(flatten)]
    #[serde(default)]
    pub buffer: BufferConfig,

    // TODO: this doesn't show up in clap usage correctly, not sure why
    #[command(flatten)]
    #[serde(default = "OptConfig::default")]
    pub metrics: OptConfig<M>,
}

#[derive(Debug, clap::Args, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct YellowstoneConfig {
    #[arg(long, env)]
    pub endpoint: String,
    #[arg(long, env)]
    pub x_token: Option<String>,
    #[arg(long, env)]
    pub timeout: u64,
}

#[derive(Default, Debug, Clone, Copy, clap::Args, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct BufferConfig {
    pub jobs: Option<usize>,
}

#[derive(
    Default,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    clap::Args,
    serde::Deserialize,
)]
pub struct NullConfig;

impl From<Option<NullConfig>> for NullConfig {
    #[inline]
    fn from(value: Option<NullConfig>) -> Self { value.unwrap_or_default() }
}

#[derive(Debug, serde::Deserialize)]
#[repr(transparent)]
pub struct OptConfig<T>(Option<T>);

impl<T> Default for OptConfig<T> {
    #[inline]
    fn default() -> Self { Self(None) }
}

impl<T> OptConfig<T> {
    #[inline]
    pub fn opt(self) -> Option<T> { self.into() }
}

impl<T> From<T> for OptConfig<T> {
    #[inline]
    fn from(value: T) -> Self { Some(value).into() }
}

impl<T> From<Option<T>> for OptConfig<T> {
    #[inline]
    fn from(value: Option<T>) -> Self { Self(value) }
}

impl<T> From<OptConfig<T>> for Option<T> {
    #[inline]
    fn from(OptConfig(value): OptConfig<T>) -> Self { value }
}

impl<T> std::ops::Deref for OptConfig<T> {
    type Target = Option<T>;

    #[inline]
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T> std::ops::DerefMut for OptConfig<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl<T: clap::FromArgMatches> clap::FromArgMatches for OptConfig<T> {
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        T::from_arg_matches(matches).map(Into::into)
    }

    fn from_arg_matches_mut(matches: &mut clap::ArgMatches) -> Result<Self, clap::Error> {
        T::from_arg_matches(matches).map(Into::into)
    }

    fn update_from_arg_matches(&mut self, matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        self.0
            .as_mut()
            .expect("Cannot update empty OptConfig")
            .update_from_arg_matches(matches)
    }

    fn update_from_arg_matches_mut(
        &mut self,
        matches: &mut clap::ArgMatches,
    ) -> Result<(), clap::Error> {
        self.0
            .as_mut()
            .expect("Cannot update empty OptConfig")
            .update_from_arg_matches_mut(matches)
    }
}

impl<T: clap::Args> clap::Args for OptConfig<T> {
    fn group_id() -> Option<clap::Id> { T::group_id() }

    fn augment_args(cmd: clap::Command) -> clap::Command { T::augment_args(cmd) }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        T::augment_args_for_update(cmd)
    }
}

// Used for clap hacks below
#[allow(dead_code)] // Currently unused if feature 'prometheus' is disabled
fn update_clone<T: ToOwned, U: Into<T>, F: FnOnce(&mut U) -> V, V>(t: &mut T, f: F) -> V
where T::Owned: Into<U> {
    let mut u = t.to_owned().into();
    let ret = f(&mut u);
    *t = u.into();
    ret
}

#[cfg(feature = "prometheus")]
mod prometheus_impl {
    use super::MaybeDefault;
    use crate::PrivateString;

    #[derive(Debug, Clone /* TODO: used for hack */, serde::Deserialize)]
    #[serde(rename_all = "kebab-case")]
    pub struct PrometheusConfig {
        pub endpoint: String,
        pub job: String,
        pub username: String,
        pub password: PrivateString,
        pub export_interval: u64,
    }

    #[cfg(feature = "prometheus")]
    impl MaybeDefault for PrometheusConfig {
        #[inline]
        fn default_opt() -> Option<Self> { None }
    }

    // TODO: Don't use hacks to rename clap arguments
    mod clap_hacks {
        use clap::{Args, FromArgMatches};

        use crate::config::update_clone;

        #[allow(clippy::struct_field_names)]
        #[derive(clap::Args)]
        struct PrometheusConfig {
            #[arg(long, env)]
            prometheus_endpoint: String,
            #[arg(long, env)]
            prometheus_job: String,
            #[arg(long, env)]
            prometheus_user: String,
            #[arg(long, env)]
            prometheus_pass: String,
            #[arg(long, env)]
            prometheus_export_interval: u64,
        }

        impl From<super::PrometheusConfig> for PrometheusConfig {
            fn from(value: super::PrometheusConfig) -> Self {
                let super::PrometheusConfig {
                    endpoint,
                    job,
                    username,
                    password,
                    export_interval,
                } = value;
                Self {
                    prometheus_endpoint: endpoint,
                    prometheus_job: job,
                    prometheus_user: username,
                    prometheus_pass: password.into(),
                    prometheus_export_interval: export_interval,
                }
            }
        }

        impl From<PrometheusConfig> for super::PrometheusConfig {
            fn from(value: PrometheusConfig) -> Self {
                let PrometheusConfig {
                    prometheus_endpoint,
                    prometheus_job,
                    prometheus_user,
                    prometheus_pass,
                    prometheus_export_interval,
                } = value;
                Self {
                    endpoint: prometheus_endpoint,
                    job: prometheus_job,
                    username: prometheus_user,
                    password: prometheus_pass.into(),
                    export_interval: prometheus_export_interval,
                }
            }
        }

        impl FromArgMatches for super::PrometheusConfig {
            fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
                PrometheusConfig::from_arg_matches(matches).map(Into::into)
            }

            fn from_arg_matches_mut(matches: &mut clap::ArgMatches) -> Result<Self, clap::Error> {
                PrometheusConfig::from_arg_matches_mut(matches).map(Into::into)
            }

            fn update_from_arg_matches(
                &mut self,
                matches: &clap::ArgMatches,
            ) -> Result<(), clap::Error> {
                update_clone(self, |c: &mut PrometheusConfig| {
                    c.update_from_arg_matches(matches)
                })
            }

            fn update_from_arg_matches_mut(
                &mut self,
                matches: &mut clap::ArgMatches,
            ) -> Result<(), clap::Error> {
                update_clone(self, |c: &mut PrometheusConfig| {
                    c.update_from_arg_matches_mut(matches)
                })
            }
        }

        impl Args for super::PrometheusConfig {
            fn group_id() -> Option<clap::Id> { PrometheusConfig::group_id() }

            fn augment_args(cmd: clap::Command) -> clap::Command {
                PrometheusConfig::augment_args_for_update(cmd)
            }

            fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
                PrometheusConfig::augment_args_for_update(cmd)
            }
        }
    }
}
