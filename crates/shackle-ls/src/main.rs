//! Stdio host for the embeddable language-server core.
use std::error::Error;

#[cfg(test)]
use expect_test as _;
use shackle_ls as _;
// Dependencies are used by the library target; mark them as intentionally
// shared for this deliberately thin binary host.
use {
	crossbeam_channel as _, log as _, lsp_server as _, lsp_types as _, miette as _,
	serde_json as _, shackle_diagnostics as _, shackle_fmt as _, shackle_hir as _,
	shackle_syntax as _, shackle_thir as _, shackle_ty as _, threadpool as _,
};

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
	env_logger::Builder::new()
		.format_target(false)
		.format_module_path(true)
		.filter_level(log::LevelFilter::Trace)
		.filter_module("salsa", log::LevelFilter::Warn)
		.filter_module("shackle", log::LevelFilter::Warn)
		.parse_default_env()
		.init();
	shackle_ls::run_stdio()
}
