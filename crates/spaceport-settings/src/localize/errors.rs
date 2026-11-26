//! The SSH host table and the backdrop a theme could not use, from
//! `error-config.ftl` and `error-backdrop.ftl`.

use crate::{BackdropProblem, ssh_hosts};
use spaceport_i18n::{Localize, fl};

impl Localize for ssh_hosts::Error {
    fn localize(&self) -> String {
        match self {
            // keep-sorted start
            Self::InvalidTarget(target) => fl!(
                "error-config-ssh-invalid-target",
                target = format!("{target:?}")
            ),
            Self::TooMany(limit) => fl!("error-config-ssh-too-many", limit = *limit),
            // keep-sorted end
        }
    }
}

impl Localize for BackdropProblem {
    fn localize(&self) -> String {
        match self {
            Self::EmptyImage => fl!("error-backdrop-empty-image"),
            Self::ImagePathEncoding => fl!("error-backdrop-image-path-encoding"),
            Self::Opacity => fl!("error-backdrop-opacity"),
            Self::RelativeImage => fl!("error-backdrop-relative-image"),
        }
    }
}
