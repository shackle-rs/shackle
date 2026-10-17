//! Command line interface for Shackle

#![warn(missing_docs)]
#![warn(unused_crate_dependencies, unused_extern_crates)]
#![warn(variant_size_differences)]

use std::{
	ffi::{OsStr, OsString},
	fs::File,
	io::BufReader,
	ops::Deref,
	panic,
	path::PathBuf,
};

use clap::{Args, Parser, Subcommand, ValueEnum, crate_version};
use env_logger::{Builder, fmt::TimestampPrecision};
use humantime::Duration;
use log::warn;
use miette::{IntoDiagnostic, LabeledSpan, MietteDiagnostic, Report, Result, Severity};
use shackle::{CompileTarget, Error, Message, Model, Solver, Status, error::InternalError};
use shackle_fmt::{MiniZincFormatOptions, check_format, format_files, format_str};
use shackle_globalizer::{Finding, GlobalizeOptions, Location, globalize};
#[cfg(test)]
use tempfile as _;

/// The main function is the entry point for the `shackle` executable.
///
/// It parses the command-line arguments using a Clap parser, processes the
/// arguments, and then dispatches to called operation.
fn main() -> Result<()> {
	// Parse command line arguments
	// TODO: Should this use Miette?
	// let cli = Cli::try_parse().into_diagnostic()?;
	let cli = Cli::parse();

	// Initialise logger based on how many times the user used the "verbose"
	// flag
	let mut logger = Builder::new();
	logger
		.format_target(false)
		.format_module_path(cli.verbose >= 2)
		.filter_level(log::LevelFilter::Warn)
		.format_timestamp(match cli.verbose {
			0 => None,
			1 => Some(TimestampPrecision::Seconds),
			_ => Some(TimestampPrecision::Millis),
		})
		.parse_default_env();
	match cli.verbose {
		0 => (),
		1 => {
			logger.filter_level(log::LevelFilter::Info);
		}
		2 => {
			logger.filter_level(log::LevelFilter::Debug);
		}
		_ => {
			logger.filter_level(log::LevelFilter::Trace);
		}
	};
	logger.init();

	log::warn!(
		"Shackle is an unfinished product not ready to be used for any purpose apart from its own development."
	);

	// Dispatch to the correct subcommand
	match panic::catch_unwind(|| match cli.subcmd {
		SubCommand::Compile(c) => c.dispatch(),
		SubCommand::Solve(s) => s.dispatch(),
		SubCommand::Check(c) => c.dispatch(),
		SubCommand::Format(f) => f.dispatch(),
		SubCommand::Globalize(g) => g.dispatch(),
	}) {
		Err(_) => Err(InternalError::new("Panic occurred during execution").into()),
		Ok(res) => res,
	}
}

/// A command line interface to the shackle constraint modelling and rewriting
/// library.
#[derive(Parser)]
#[command(
    name = "shackle",
	version = crate_version!(),
)]
struct Cli {
	/// A level of verbosity, and can be used multiple times
	#[arg(short, long, action = clap::ArgAction::Count)]
	verbose: u8,
	#[command(subcommand)]
	subcmd: SubCommand,
}

#[derive(Subcommand)]
enum SubCommand {
	/// Compile a model to the intermediate form accepted by the interpreter and
	/// output the model to a file
	Compile(Box<Compile>),
	Solve(Box<Solve>),
	Check(Box<Check>),
	Format(Box<Format>),
	Globalize(Box<Globalize>),
}

/// Solve the given model instance using the given solver
#[derive(Args)]
struct Solve {
	#[arg(long)]
	statistics: bool,
	#[arg(long)]
	time_limit: Option<Duration>,
	#[command(flatten)]
	base: Compile,
	/// Additional arguments passed to MiniZinc (must follow `--`)
	#[arg(last = true)]
	minizinc_args: Vec<OsString>,
}

impl Solve {
	/// The dispatch method checks the validity of the user input and then call
	/// the corresponding functions in the modelling libraries.
	pub fn dispatch(&self) -> Result<()> {
		let (model, data) = self.base.sort_files()?;
		let slv = self.base.solver()?;

		// Construct model, typecheck, and compile into program
		let mut model = Model::from_file(model);
		model.set_target(self.base.target.into());
		let mut program = model.compile(&slv)?;

		program.add_data_files(data.iter().map(|f| f.deref()))?;

		// Set program options
		if let Some(time_limit) = self.time_limit {
			program = program.with_time_limit(time_limit.into());
		}
		program = program
			.with_statistics(self.statistics)
			.with_minizinc_args(self.minizinc_args.clone());

		// Run resulting program and show results
		let display_fn = |x: &Message| {
			print!("{}", x);
			Ok(())
		};
		let status = program.run(display_fn)?;
		match status {
			Status::Infeasible => println!("=====UNSATISFIABLE====="),
			Status::Satisfied => {}
			Status::Optimal | Status::AllSolutions => println!("=========="),
			Status::Unknown => println!("=====UNKNOWN====="),
		}

		// Compilation succeeded
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use miette::{GraphicalReportHandler, GraphicalTheme};
	use shackle::SourceFile;

	use super::*;

	#[test]
	fn finding_reports_match_the_number_of_constraints() {
		let src = SourceFile::new("m.mzn".into(), "constraint a != b;\n".to_owned());
		let at = Location {
			src,
			span: (0, 17).into(),
		};
		let finding = |n, replacement: &str| Finding {
			locations: vec![at.clone(); n],
			context: None,
			replacement: replacement.to_owned(),
		};
		let text = |f: &Finding| {
			let report = finding_report(f);
			(report.to_string(), report.help().unwrap().to_string())
		};
		assert_eq!(
			text(&finding(1, "true")),
			(
				"This constraint is redundant".to_owned(),
				"remove it, or mark it with `redundant_constraint` if it helps the solver"
					.to_owned()
			)
		);
		assert_eq!(
			text(&finding(2, "true")),
			(
				"These constraints are redundant".to_owned(),
				"remove them, or mark them with `redundant_constraint` if they help the solver"
					.to_owned()
			)
		);
		assert_eq!(
			text(&finding(2, "all_different(x)")).0,
			"These constraints can be replaced by a global constraint"
		);
	}

	#[test]
	fn suggestion_marks_the_constraints_and_the_context() {
		let src = SourceFile::new(
			"m.mzn".into(),
			"constraint a != b;\nconstraint b != c;\n".to_owned(),
		);
		let at = |offset: usize| Location {
			src: src.clone(),
			span: (offset, 17).into(),
		};
		let finding = Finding {
			locations: vec![at(0)],
			context: Some(at(19)),
			replacement: "all_different([a, b, c])".to_owned(),
		};
		let report = suggestion(
			&finding,
			"This constraint can be replaced",
			"replace with it",
		);
		let mut out = String::new();
		GraphicalReportHandler::new_themed(GraphicalTheme::unicode_nocolor())
			.with_width(80)
			.render_report(&mut out, report.as_ref())
			.unwrap();
		expect_test::expect![[r#"
    shackle::globalize

      ☞ This constraint can be replaced
       ╭─[m.mzn:1:1]
     1 │ constraint a != b;
       · ─────────────────
     2 │ constraint b != c;
       · ────────┬────────
       ·         ╰── needed for this
       ╰────
      help: replace with it
"#]]
		.assert_eq(&out);
	}

	fn compile_args(files: &[&str]) -> Compile {
		Compile {
			solver: "gecode".to_owned(),
			files: files.iter().map(PathBuf::from).collect(),
			target: CompileTargetFlag::MiniZinc,
		}
	}

	#[test]
	fn sort_files_splits_model_from_data_files() {
		let compile = compile_args(&["instance.dzn", "model.mzn", "values.json"]);

		let (model, data) = compile.sort_files().unwrap();

		assert_eq!(model, PathBuf::from("model.mzn"));
		assert_eq!(
			data,
			[PathBuf::from("instance.dzn"), PathBuf::from("values.json")]
		);
	}

	#[test]
	fn sort_files_accepts_eprime_models() {
		let compile = compile_args(&["model.eprime", "instance.dzn"]);

		let (model, data) = compile.sort_files().unwrap();

		assert_eq!(model, PathBuf::from("model.eprime"));
		assert_eq!(data, [PathBuf::from("instance.dzn")]);
	}

	#[test]
	fn sort_files_rejects_multiple_model_files() {
		let compile = compile_args(&["first.mzn", "second.eprime"]);

		let err = compile.sort_files().unwrap_err().to_string();

		assert!(err.contains("detected multiple model files"));
		assert!(err.contains("first.mzn"));
		assert!(err.contains("second.eprime"));
	}

	#[test]
	fn sort_files_rejects_unsupported_file_types() {
		let compile = compile_args(&["model.mzn", "notes.txt"]);

		let err = compile.sort_files().unwrap_err().to_string();

		assert!(err.contains("unsupported file type"));
		assert!(err.contains("notes.txt"));
	}

	#[test]
	fn sort_files_requires_a_model_file() {
		let compile = compile_args(&["instance.dzn", "values.json"]);

		let err = compile.sort_files().unwrap_err().to_string();

		assert!(err.contains("no model file detected"));
	}

	#[test]
	fn solver_resolves_registered_solver_tags() {
		let compile = Compile {
			solver: "test-solver".to_owned(),
			files: vec![PathBuf::from("model.mzn")],
			target: CompileTargetFlag::MiniZinc,
		};

		assert!(compile.solver().is_ok());
	}

	#[test]
	fn boolean_flags_convert_to_bool() {
		assert!(bool::from(BooleanFlag::On));
		assert!(!bool::from(BooleanFlag::Off));
	}

	#[test]
	fn solve_forwards_arguments_after_double_dash() {
		let cli = Cli::try_parse_from([
			"shackle",
			"solve",
			"model.mzn",
			"--",
			"--all-solutions",
			"--num-solutions",
			"0",
		])
		.unwrap();

		let SubCommand::Solve(solve) = cli.subcmd else {
			panic!("expected solve subcommand");
		};
		assert_eq!(
			solve.minizinc_args,
			["--all-solutions", "--num-solutions", "0"].map(OsString::from)
		);
	}
}

/// Check model files for correctness
#[derive(Args)]
struct Check {
	/// Check whether all the data required to create a complete problem
	/// instance is provided
	#[arg(long)]
	check_complete: bool,
	#[command(flatten)]
	base: Compile,
}

impl Check {
	/// The dispatch method checks the validity of the user input and then call
	/// the corresponding functions in the modelling libraries.
	pub fn dispatch(&self) -> Result<()> {
		let (model, data) = self.base.sort_files()?;

		let slv = self.base.solver()?;
		let model = Model::from_file(model);
		let errors = model.check(&slv, &data, self.check_complete);

		if errors.is_empty() {
			Ok(())
		} else {
			Err(Error::try_from(errors).unwrap().into())
		}
	}
}

/// Suggest global constraints that can replace groups of constraints in the
/// model. Suggestions are checked by sampling solutions for each data file.
#[derive(Args)]
struct Globalize {
	/// MiniZinc solver used to sample solutions
	#[arg(long, default_value_t = GlobalizeOptions::default().solver)]
	solver: String,
	/// Maximum number of constraint parts in a group (conjunctions and
	/// `forall`s are split into parts)
	#[arg(short = 'M', long, default_value_t = GlobalizeOptions::default().max_group)]
	max_group: usize,
	/// Number of solutions sampled for each check
	#[arg(short = 'N', long, default_value_t = GlobalizeOptions::default().samples)]
	samples: usize,
	/// Time limit for the solver in each MiniZinc run (a run is stopped
	/// after 10 times this limit)
	#[arg(long, default_value_t = GlobalizeOptions::default().time_limit.into())]
	time_limit: Duration,
	/// Random seed for the solver
	#[arg(long, default_value_t = GlobalizeOptions::default().seed)]
	seed: u64,
	/// Only suggest globals whose name contains one of these (comma separated)
	#[arg(short = 'f', long, value_delimiter = ',')]
	filter_in: Vec<String>,
	/// Never suggest globals whose name contains one of these (comma separated)
	#[arg(short = 'e', long, value_delimiter = ',')]
	filter_ex: Vec<String>,
	/// Skip a global if a group gives more argument lists for it
	#[arg(long, default_value_t = GlobalizeOptions::default().max_args)]
	max_args: usize,
	/// Do not abandon pairs of constraints where one implies the other on the
	/// samples
	#[arg(long)]
	no_implies_check: bool,
	/// Number of MiniZinc runs at the same time
	#[arg(short = 'p', long, default_value_t = GlobalizeOptions::default().jobs)]
	jobs: usize,
	/// Skip the pass that finds redundant constraints and 0/1 arrays that
	/// represent integers
	#[arg(long)]
	no_initial_pass: bool,
	/// Print the suggestions as one JSON object, not as diagnostics
	#[arg(long)]
	json: bool,
	/// The model file and data files
	#[arg(required = true)]
	files: Vec<PathBuf>,
}

impl Globalize {
	/// Run the globalizer and print its suggestions
	pub fn dispatch(&self) -> Result<()> {
		let (model, data) = sort_files(&self.files)?;
		let options = GlobalizeOptions {
			solver: self.solver.clone(),
			max_group: self.max_group,
			samples: self.samples,
			time_limit: self.time_limit.into(),
			seed: self.seed,
			include: self.filter_in.clone(),
			exclude: self.filter_ex.clone(),
			max_args: self.max_args,
			implies_check: !self.no_implies_check,
			initial_pass: !self.no_initial_pass,
			jobs: self.jobs,
		};
		let result = globalize(&Model::from_file(model), &data, &options)?;
		if self.json {
			let findings = |fs: &[Finding]| fs.iter().map(finding_json).collect::<Vec<_>>();
			let viewpoints = result
				.viewpoints
				.iter()
				.map(|vp| {
					let mut json = finding_json(&vp.suggestion);
					json["name"] = vp.name.clone().into();
					json["findings"] = findings(&vp.findings).into();
					json
				})
				.collect::<Vec<_>>();
			let json = serde_json::json!({
				"viewpoints": viewpoints,
				"findings": findings(&result.findings),
				"calls": result.calls,
			});
			println!("{json}");
			return Ok(());
		}
		// In the order of the model: a viewpoint (at its 0/1 array) with the
		// suggestions that use it, and the other suggestions
		let mut reports = result
			.findings
			.iter()
			.map(|f| (&f.locations[0], vec![finding_report(f)]))
			.collect::<Vec<_>>();
		for vp in &result.viewpoints {
			let help = format!("add the viewpoint:\n{}", code(&vp.suggestion.replacement));
			let message = "This 0/1 array represents an integer array";
			let mut block = vec![suggestion(&vp.suggestion, message, help)];
			block.extend(vp.findings.iter().map(finding_report));
			reports.push((&vp.suggestion.locations[0], block));
		}
		reports.sort_by(|a, b| a.0.cmp(b.0));
		for report in reports.into_iter().flat_map(|(_, block)| block) {
			println!("{report:?}");
		}
		println!(
			"Suggestions: {}, viewpoints: {}, MiniZinc runs: {}",
			result.findings.len()
				+ result
					.viewpoints
					.iter()
					.map(|vp| vp.findings.len())
					.sum::<usize>(),
			result.viewpoints.len(),
			result.calls
		);
		Ok(())
	}
}

/// A suggested replacement (or redundant constraints) as a diagnostic
fn finding_report(finding: &Finding) -> Report {
	let (these, are, them, they_help) = if finding.locations.len() == 1 {
		("This constraint", "is", "it", "it helps")
	} else {
		("These constraints", "are", "them", "they help")
	};
	let (message, help) = if finding.replacement == "true" {
		(
			format!("{these} {are} redundant"),
			format!(
				"remove {them}, or mark {them} with `redundant_constraint` if {they_help} the solver"
			),
		)
	} else {
		(
			format!("{these} can be replaced by a global constraint"),
			format!(
				"replace with:\n{}",
				code(&format!("constraint {};", finding.replacement))
			),
		)
	};
	suggestion(finding, &message, help)
}

/// MiniZinc items formatted for a help message
fn code(items: &str) -> String {
	let options = MiniZincFormatOptions {
		line_width: 72,
		use_tabs: false,
		indent_size: 2,
		..Default::default()
	};
	format_str(items, &options).map_or_else(|_| items.to_owned(), |f| f.trim_end().to_owned())
}

/// A finding as JSON, with locations as in MiniZinc's JSON messages
fn finding_json(finding: &Finding) -> serde_json::Value {
	let location = |loc: &Location| {
		let ((l1, c1), (l2, c2)) = (loc.start(), loc.end());
		serde_json::json!({
			"filename": loc.src.name(),
			"firstLine": l1,
			"firstColumn": c1,
			"lastLine": l2,
			"lastColumn": c2,
		})
	};
	serde_json::json!({
		"locations": finding.locations.iter().map(location).collect::<Vec<_>>(),
		"context": finding.context.as_ref().map(location),
		"replacement": finding.replacement,
	})
}

/// A finding as a miette diagnostic. Labels mark the locations in the file of
/// the first one, and the help lists the others.
fn suggestion(finding: &Finding, message: &str, help: impl Into<String>) -> Report {
	let src = &finding.locations[0].src;
	let mut labels = Vec::new();
	let mut elsewhere = Vec::new();
	for loc in &finding.locations {
		if loc.src == *src {
			labels.push(LabeledSpan::new_with_span(None, loc.span));
		} else {
			elsewhere.push(loc.to_string());
		}
	}
	if let Some(context) = &finding.context {
		let text = Some("needed for this".to_owned());
		if context.src == *src {
			labels.push(LabeledSpan::new_with_span(text, context.span));
		} else {
			elsewhere.push(format!("{context} (needed for this)"));
		}
	}
	let mut help = help.into();
	if !elsewhere.is_empty() {
		help = format!("{help}\nalso at: {}", elsewhere.join(", "));
	}
	let diagnostic = MietteDiagnostic::new(message)
		.with_code("shackle::globalize")
		.with_severity(Severity::Advice)
		.with_labels(labels)
		.with_help(help);
	Report::new(diagnostic).with_source_code(src.clone())
}

/// Compile the given model to a shackle intermediate format
#[derive(Args)]
pub struct Compile {
	#[arg(long, default_value = "gecode")]
	solver: String,
	#[arg(required = true)]
	files: Vec<PathBuf>,
	/// Compilation target
	#[arg(long, default_value = "microzinc")]
	target: CompileTargetFlag,
}

impl Compile {
	/// Sort through the files in the command line arguments and split the model
	/// from the data
	pub fn sort_files(&self) -> Result<(PathBuf, Vec<PathBuf>)> {
		sort_files(&self.files)
	}

	/// Resolve shackle [`Solver`] from the solver command line flag
	pub fn solver(&self) -> Result<Solver> {
		match shackle::Solver::lookup(self.solver.as_str()) {
			Some(slv) => Ok(slv),
			None => Err(Report::msg(format!(
				"no solver has been registered with the tag `{}'",
				self.solver,
			))),
		}
	}

	/// The dispatch method checks the validity of the user input and then call
	/// the corresponding functions in the modelling libraries.
	pub fn dispatch(&self) -> Result<()> {
		let (model, _data) = self.sort_files()?;

		let filename = model.with_extension("shackle.mzn");

		let slv = self.solver()?;
		let mut model = Model::from_file(model);
		model.set_target(self.target.into());
		let prg = model.compile(&slv)?;

		let mut file = File::create(filename).into_diagnostic()?;
		prg.write(&mut file).into_diagnostic()
	}
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum BooleanFlag {
	/// Enabled
	#[value(alias("true"), alias("1"))]
	On,
	/// Disabled
	#[value(alias("false"), alias("0"))]
	Off,
}

impl From<BooleanFlag> for bool {
	fn from(value: BooleanFlag) -> Self {
		match value {
			BooleanFlag::On => true,
			BooleanFlag::Off => false,
		}
	}
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum CompileTargetFlag {
	/// Compile to MicroZinc
	#[value(alias("uzn"), alias("microzinc"))]
	MicroZinc,
	/// Compile to MiniZinc
	#[value(alias("mzn"), alias("minizinc"))]
	MiniZinc,
}

impl From<CompileTargetFlag> for CompileTarget {
	fn from(value: CompileTargetFlag) -> Self {
		match value {
			CompileTargetFlag::MicroZinc => CompileTarget::MicroZinc,
			CompileTargetFlag::MiniZinc => CompileTarget::MiniZinc,
		}
	}
}

/// Format MiniZinc code
#[derive(Args)]
struct Format {
	/// Files or directories to format.
	#[arg(required = true, value_hint = clap::ValueHint::AnyPath)]
	files: Vec<PathBuf>,

	/// Check formatting only without modifying files.
	#[arg(long)]
	check: bool,

	#[arg(long, value_hint = clap::ValueHint::FilePath)]
	config: Option<PathBuf>,

	/// Target maximum line length
	#[arg(long)]
	line_width: Option<usize>,
	/// Whether to indent using tabs
	#[arg(long)]
	use_tabs: Option<BooleanFlag>,
	/// Size of indent
	#[arg(long)]
	indent_size: Option<usize>,

	/// Keep parentheses (except double parentheses)
	#[arg(long)]
	keep_parentheses: Option<BooleanFlag>,
}

impl Format {
	/// The dispatch method checks the validity of the user input and runs the formatter
	pub fn dispatch(&self) -> Result<()> {
		let mut options = if let Some(config) = &self.config {
			let f = File::open(config).into_diagnostic()?;
			let reader = BufReader::new(f);
			serde_json::from_reader(reader).into_diagnostic()?
		} else {
			MiniZincFormatOptions::default()
		};

		if let Some(line_width) = self.line_width {
			options.line_width = line_width;
		}
		if let Some(use_tabs) = self.use_tabs {
			options.use_tabs = use_tabs.into();
		}
		if let Some(indent_size) = self.indent_size {
			options.indent_size = indent_size;
		}
		if let Some(keep_parentheses) = self.keep_parentheses {
			options.keep_parentheses = keep_parentheses.into();
		}

		if self.check {
			check_format(self.files.iter(), &options)?;
		} else {
			format_files(self.files.iter(), &options)?;
		}
		Ok(())
	}
}

/// Sort through the files in the command line arguments and split the model
/// from the data
fn sort_files(files: &[PathBuf]) -> Result<(PathBuf, Vec<PathBuf>)> {
	let mut model_file: Option<PathBuf> = None;
	let mut data = Vec::with_capacity(files.len().saturating_sub(1));
	for f in files.iter() {
		match f.extension().and_then(OsStr::to_str) {
			Some("mzn") | Some("eprime") => {
				if let Some(other) = model_file {
					return Err(Report::msg(format!(
						"detected multiple model files: `{}' and `{}'",
						other.display(),
						f.display()
					)));
				} else {
					model_file = Some(f.clone())
				}
			}
			Some("json") | Some("dzn") => data.push(f.clone()),
			_ => {
				return Err(Report::msg(format!(
					"file `{}' has an unsupported file type",
					f.to_str().unwrap_or_default()
				)));
			}
		}
	}
	if let Some(f) = model_file {
		Ok((f, data))
	} else {
		Err(Report::msg("no model file detected"))
	}
}
