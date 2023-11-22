use ::drink::DEFAULT_GAS_LIMIT;
use anyhow::{anyhow, Context, Error, Result};
use ink_primitives::AccountId;

use super::*;
use crate::{ExecCall, InstantiateCall, ReadCall, UploadCall};
