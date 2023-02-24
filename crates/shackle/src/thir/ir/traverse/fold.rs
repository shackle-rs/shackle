use rustc_hash::FxHashMap;

use crate::thir::{db::Thir, source::Origin, *};

/// Replacement map for references to items
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReplacementMap {
	annotations: FxHashMap<AnnotationId, AnnotationId>,
	constraints: FxHashMap<ConstraintId, ConstraintId>,
	declarations: FxHashMap<DeclarationId, DeclarationId>,
	enumerations: FxHashMap<EnumerationId, EnumerationId>,
	functions: FxHashMap<FunctionId, FunctionId>,
	outputs: FxHashMap<OutputId, OutputId>,
}

impl ReplacementMap {
	/// Get the replacement for this annotation ID if any
	pub fn get_annotation(&self, src: AnnotationId) -> Option<AnnotationId> {
		self.annotations.get(&src).copied()
	}

	/// Insert an annotation ID into the replace map
	pub fn insert_annotation(&mut self, src: AnnotationId, dst: AnnotationId) {
		self.annotations.insert(src, dst);
	}

	/// Get the replacement for this constraint ID if any
	pub fn get_constraint(&self, src: ConstraintId) -> Option<ConstraintId> {
		self.constraints.get(&src).copied()
	}

	/// Insert an constraint ID into the replace map
	pub fn insert_constraint(&mut self, src: ConstraintId, dst: ConstraintId) {
		self.constraints.insert(src, dst);
	}

	/// Get the replacement for this declaration ID if any
	pub fn get_declaration(&self, src: DeclarationId) -> Option<DeclarationId> {
		self.declarations.get(&src).copied()
	}

	/// Insert an declaration ID into the replace map
	pub fn insert_declaration(&mut self, src: DeclarationId, dst: DeclarationId) {
		self.declarations.insert(src, dst);
	}

	/// Get the replacement for this enumeration ID if any
	pub fn get_enumeration(&self, src: EnumerationId) -> Option<EnumerationId> {
		self.enumerations.get(&src).copied()
	}

	/// Insert an enumeration ID into the replace map
	pub fn insert_enumeration(&mut self, src: EnumerationId, dst: EnumerationId) {
		self.enumerations.insert(src, dst);
	}

	/// Get the replacement for this enum member ID if any
	pub fn get_enum_member(&self, src: EnumMemberId) -> Option<EnumMemberId> {
		self.get_enumeration(src.enumeration_id())
			.map(|e| EnumMemberId::new(e, src.member_index()))
	}

	/// Get the replacement for this function ID if any
	pub fn get_function(&self, src: FunctionId) -> Option<FunctionId> {
		self.functions.get(&src).copied()
	}

	/// Insert an function ID into the replace map
	pub fn insert_function(&mut self, src: FunctionId, dst: FunctionId) {
		self.functions.insert(src, dst);
	}

	/// Get the replacement for this output ID if any
	pub fn get_output(&self, src: OutputId) -> Option<OutputId> {
		self.outputs.get(&src).copied()
	}

	/// Insert an output ID into the replace map
	pub fn insert_output(&mut self, src: OutputId, dst: OutputId) {
		self.outputs.insert(src, dst);
	}
}

/// Trait for folding a model, and adding a transformed version of the items into another model.
pub trait Folder {
	/// Get the replacement map
	fn replacement_map(&mut self) -> &mut ReplacementMap;

	/// Get the destination model into which the nodes are added
	fn model(&mut self) -> &mut Model;

	/// Add the items from the model into the destination model
	fn add_model(&mut self, db: &dyn Thir, model: &Model) {
		add_model(self, db, model)
	}

	/// Add the folded version of this item to the destination model (and also to the replacement map)
	fn add_item(&mut self, db: &dyn Thir, model: &Model, item: ItemId) {
		add_item(self, db, model, item)
	}

	/// Add the folded version of this annotation item into the destination model.
	fn add_annotation(&mut self, db: &dyn Thir, model: &Model, a: AnnotationId) {
		add_annotation(self, db, model, a);
	}

	/// Fold an annotation item.
	fn fold_annotation(&mut self, db: &dyn Thir, model: &Model, a: &Annotation) -> Annotation {
		fold_annotation(self, db, model, a)
	}

	/// Fold an annotation ID.
	fn fold_annotation_id(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		a: AnnotationId,
	) -> AnnotationId {
		fold_annotation_id(self, db, model, a)
	}

	/// Add the folded version of this constraint item into the destination model.
	fn add_constraint(&mut self, db: &dyn Thir, model: &Model, c: ConstraintId) {
		add_constraint(self, db, model, c);
	}

	/// Fold a constraint item.
	fn fold_constraint(&mut self, db: &dyn Thir, model: &Model, c: &Constraint) -> Constraint {
		fold_constraint(self, db, model, c)
	}

	/// Fold a constraint ID.
	fn fold_constraint_id(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		c: ConstraintId,
	) -> ConstraintId {
		fold_constraint_id(self, db, model, c)
	}

	/// Add the folded version of this declaration item into the destination model.
	fn add_declaration(&mut self, db: &dyn Thir, model: &Model, d: DeclarationId) {
		add_declaration(self, db, model, d);
	}

	/// Fold a declaration item.
	fn fold_declaration(&mut self, db: &dyn Thir, model: &Model, d: &Declaration) -> Declaration {
		fold_declaration(self, db, model, d)
	}

	/// Fold a declaration ID.
	fn fold_declaration_id(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		d: DeclarationId,
	) -> DeclarationId {
		fold_declaration_id(self, db, model, d)
	}

	/// Add the folded version of this enumeration item into the destination model.
	fn add_enumeration(&mut self, db: &dyn Thir, model: &Model, e: EnumerationId) {
		add_enumeration(self, db, model, e);
	}

	/// Fold an enumeration item.
	fn fold_enumeration(&mut self, db: &dyn Thir, model: &Model, e: &Enumeration) -> Enumeration {
		fold_enumeration(self, db, model, e)
	}

	/// Fold an enumeration ID.
	fn fold_enumeration_id(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		e: EnumerationId,
	) -> EnumerationId {
		fold_enumeration_id(self, db, model, e)
	}

	/// Fold an enum member ID.
	fn fold_enum_member_id(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		e: EnumMemberId,
	) -> EnumMemberId {
		fold_enum_member_id(self, db, model, e)
	}

	/// Fold a constructor
	fn fold_constructor(&mut self, db: &dyn Thir, model: &Model, c: &Constructor) -> Constructor {
		fold_constructor(self, db, model, c)
	}

	/// Add the folded version of this function item into the destination model.
	fn add_function(&mut self, db: &dyn Thir, model: &Model, f: FunctionId) {
		add_function(self, db, model, f);
	}

	/// Fold a function item.
	///
	/// Note that this doesn't fold the body, which has to be processed at the end
	/// since it may refer to items which have not been added yet.
	fn fold_function(&mut self, db: &dyn Thir, model: &Model, f: &Function) -> Function {
		fold_function(self, db, model, f)
	}

	/// Fold a function ID.
	fn fold_function_id(&mut self, db: &dyn Thir, model: &Model, f: FunctionId) -> FunctionId {
		fold_function_id(self, db, model, f)
	}

	/// Fold the body of a function.
	///
	/// This is separate because bodies must be processed once the function items
	/// have been added, since they can refer to items which have not been added to
	/// the destination model yet.
	fn fold_function_body(&mut self, db: &dyn Thir, model: &Model, f: FunctionId) {
		fold_function_body(self, db, model, f)
	}

	/// Add the folded version of this output item into the destination model.
	fn add_output(&mut self, db: &dyn Thir, model: &Model, o: OutputId) {
		add_output(self, db, model, o);
	}

	/// Fold an output item.
	fn fold_output(&mut self, db: &dyn Thir, model: &Model, o: &Output) -> Output {
		fold_output(self, db, model, o)
	}

	/// Add the folded version of the solve item into the destination model.
	fn add_solve(&mut self, db: &dyn Thir, model: &Model) {
		add_solve(self, db, model)
	}

	/// Fold the solve item.
	fn fold_solve(&mut self, db: &dyn Thir, model: &Model, s: &Solve) -> Solve {
		fold_solve(self, db, model, s)
	}

	/// Fold an expression
	fn fold_expression(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		expression: &Expression,
	) -> Expression {
		fold_expression(self, db, model, expression)
	}

	/// Fold a boolean literal
	fn fold_boolean(
		&mut self,
		_db: &dyn Thir,
		_model: &Model,
		b: BooleanLiteral,
	) -> BooleanLiteral {
		b
	}

	/// Fold an integer literal
	fn fold_integer(
		&mut self,
		_db: &dyn Thir,
		_model: &Model,
		i: IntegerLiteral,
	) -> IntegerLiteral {
		i
	}

	/// Fold a float literal
	fn fold_float(&mut self, _db: &dyn Thir, _model: &Model, f: FloatLiteral) -> FloatLiteral {
		f
	}

	/// Fold a string literal
	fn fold_string(&mut self, _db: &dyn Thir, _model: &Model, s: &StringLiteral) -> StringLiteral {
		s.clone()
	}

	/// Fold an identifier
	fn fold_identifier(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		identifier: &ResolvedIdentifier,
	) -> ResolvedIdentifier {
		fold_identifier(self, db, model, identifier)
	}

	/// Fold an array literal
	fn fold_array_literal(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		al: &ArrayLiteral,
	) -> ArrayLiteral {
		fold_array_literal(self, db, model, al)
	}

	/// Fold a set literal
	fn fold_set_literal(&mut self, db: &dyn Thir, model: &Model, sl: &SetLiteral) -> SetLiteral {
		fold_set_literal(self, db, model, sl)
	}

	/// Fold a tuple literal
	fn fold_tuple_literal(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		tl: &TupleLiteral,
	) -> TupleLiteral {
		fold_tuple_literal(self, db, model, tl)
	}

	/// Fold a record literal
	fn fold_record_literal(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		rl: &RecordLiteral,
	) -> RecordLiteral {
		fold_record_literal(self, db, model, rl)
	}

	/// Fold an array comprehension
	fn fold_array_comprehension(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		c: &ArrayComprehension,
	) -> ArrayComprehension {
		fold_array_comprehension(self, db, model, c)
	}
	/// Fold a set comprehension
	fn fold_set_comprehension(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		c: &SetComprehension,
	) -> SetComprehension {
		fold_set_comprehension(self, db, model, c)
	}

	/// Fold an array access
	fn fold_array_access(&mut self, db: &dyn Thir, model: &Model, aa: &ArrayAccess) -> ArrayAccess {
		fold_array_access(self, db, model, aa)
	}

	/// Fold a tuple access
	fn fold_tuple_access(&mut self, db: &dyn Thir, model: &Model, ta: &TupleAccess) -> TupleAccess {
		fold_tuple_access(self, db, model, ta)
	}

	/// Fold a record access
	fn fold_record_access(
		&mut self,
		db: &dyn Thir,
		model: &Model,
		ra: &RecordAccess,
	) -> RecordAccess {
		fold_record_access(self, db, model, ra)
	}

	/// Fold an if-then-else expression
	fn fold_if_then_else(&mut self, db: &dyn Thir, model: &Model, ite: &IfThenElse) -> IfThenElse {
		fold_if_then_else(self, db, model, ite)
	}

	/// Fold a case expression
	fn fold_case(&mut self, db: &dyn Thir, model: &Model, c: &Case) -> Case {
		fold_case(self, db, model, c)
	}

	/// Fold a call expression
	fn fold_call(&mut self, db: &dyn Thir, model: &Model, call: &Call) -> Call {
		fold_call(self, db, model, call)
	}

	/// Fold a let expression
	fn fold_let(&mut self, db: &dyn Thir, model: &Model, l: &Let) -> Let {
		fold_let(self, db, model, l)
	}

	/// Fold a lambda expression
	fn fold_lambda(&mut self, db: &dyn Thir, model: &Model, l: &Lambda) -> Lambda {
		fold_lambda(self, db, model, l)
	}

	/// Fold a comprehension generator
	fn fold_generator(&mut self, db: &dyn Thir, model: &Model, generator: &Generator) -> Generator {
		fold_generator(self, db, model, generator)
	}

	/// Fold a domain
	fn fold_domain(&mut self, db: &dyn Thir, model: &Model, domain: &Domain) -> Domain {
		fold_domain(self, db, model, domain)
	}

	/// Fold a case pattern
	fn fold_pattern(&mut self, db: &dyn Thir, model: &Model, pattern: &Pattern) -> Pattern {
		fold_pattern(self, db, model, pattern)
	}
}

fn alloc_expression<F: Folder + ?Sized>(
	db: &dyn Thir,
	origin: impl Into<Origin>,
	value: impl ExpressionBuilder,
	folder: &mut F,
) -> Expression {
	Expression::new(db, folder.model(), origin, value)
}

/// Fold the model by adding folded versions of the items into the destination model.
///
/// First, we add each folded item into the destination model, except for function bodies.
/// The function bodies are then added once all of the items are made available.
pub fn add_model<F: Folder + ?Sized>(folder: &mut F, db: &dyn Thir, model: &Model) {
	// Add items to the destination model
	for item in model.top_level_items() {
		folder.add_item(db, model, item);
	}
	// Now that all items have been added, we can process function bodies
	for (f, i) in model.functions() {
		if i.body().is_some() {
			folder.fold_function_body(db, model, f);
		}
	}
}

/// Fold an item
pub fn add_item<F: Folder + ?Sized>(folder: &mut F, db: &dyn Thir, model: &Model, item: ItemId) {
	match item {
		ItemId::Annotation(a) => folder.add_annotation(db, model, a),
		ItemId::Constraint(c) => folder.add_constraint(db, model, c),
		ItemId::Declaration(d) => folder.add_declaration(db, model, d),
		ItemId::Enumeration(e) => folder.add_enumeration(db, model, e),
		ItemId::Function(f) => folder.add_function(db, model, f),
		ItemId::Output(o) => folder.add_output(db, model, o),
		ItemId::Solve => folder.add_solve(db, model),
	}
}

/// Add the folded version of this annotation item into the destination model.
pub fn add_annotation<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	a: AnnotationId,
) -> AnnotationId {
	let annotation = folder.fold_annotation(db, model, &*model[a]);
	let idx = folder
		.model()
		.add_annotation(Item::new(annotation, model[a].origin()));
	folder.replacement_map().insert_annotation(a, idx);
	idx
}

/// Fold an annotation item
pub fn fold_annotation<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	a: &Annotation,
) -> Annotation {
	folder.fold_constructor(db, model, &**a).into()
}

/// Fold an annotation ID
pub fn fold_annotation_id<F: Folder + ?Sized>(
	folder: &mut F,
	_db: &dyn Thir,
	_model: &Model,
	a: AnnotationId,
) -> AnnotationId {
	folder
		.replacement_map()
		.get_annotation(a)
		.expect("Annotation has not been added to destination model")
}

/// Add the folded version of this constraint item into the destination model.
pub fn add_constraint<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	c: ConstraintId,
) -> ConstraintId {
	let constraint = folder.fold_constraint(db, model, &*model[c]);
	let idx = folder
		.model()
		.add_constraint(Item::new(constraint, model[c].origin()));
	folder.replacement_map().insert_constraint(c, idx);
	idx
}

/// Fold a constraint item
pub fn fold_constraint<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	c: &Constraint,
) -> Constraint {
	let mut constraint = Constraint::new(
		c.top_level(),
		folder.fold_expression(db, model, c.expression()),
	);
	constraint.annotations_mut().extend(
		c.annotations()
			.iter()
			.map(|ann| folder.fold_expression(db, model, ann)),
	);
	constraint
}

/// Fold a constraint ID
pub fn fold_constraint_id<F: Folder + ?Sized>(
	folder: &mut F,
	_db: &dyn Thir,
	_model: &Model,
	a: ConstraintId,
) -> ConstraintId {
	folder
		.replacement_map()
		.get_constraint(a)
		.expect("Constraint has not been added to destination model")
}

/// Add the folded version of this declaration item into the destination model.
pub fn add_declaration<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	d: DeclarationId,
) -> DeclarationId {
	let declaration = folder.fold_declaration(db, model, &*model[d]);
	let idx = folder
		.model()
		.add_declaration(Item::new(declaration, model[d].origin()));
	folder.replacement_map().insert_declaration(d, idx);
	idx
}

/// Fold a declaration item
pub fn fold_declaration<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	d: &Declaration,
) -> Declaration {
	let mut declaration =
		Declaration::new(d.top_level(), folder.fold_domain(db, model, d.domain()));
	if let Some(name) = d.name() {
		declaration.set_name(name);
	}
	declaration.annotations_mut().extend(
		d.annotations()
			.iter()
			.map(|ann| folder.fold_expression(db, model, ann)),
	);
	if let Some(def) = d.definition() {
		declaration.set_definition(folder.fold_expression(db, model, def));
	}
	declaration
}

/// Fold a declaration ID
pub fn fold_declaration_id<F: Folder + ?Sized>(
	folder: &mut F,
	_db: &dyn Thir,
	_model: &Model,
	d: DeclarationId,
) -> DeclarationId {
	folder
		.replacement_map()
		.get_declaration(d)
		.expect("Declaration has not been added to destination model")
}

/// Add the folded version of this enumeration item into the destination model.
pub fn add_enumeration<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	e: EnumerationId,
) -> EnumerationId {
	let enumeration = folder.fold_enumeration(db, model, &*model[e]);
	let idx = folder
		.model()
		.add_enumeration(Item::new(enumeration, model[e].origin()));
	folder.replacement_map().insert_enumeration(e, idx);
	idx
}

/// Fold an enumeration item
pub fn fold_enumeration<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	e: &Enumeration,
) -> Enumeration {
	let mut enumeration = Enumeration::new(e.enum_type());
	enumeration.annotations_mut().extend(
		e.annotations()
			.iter()
			.map(|ann| folder.fold_expression(db, model, ann)),
	);
	if let Some(def) = e.definition() {
		enumeration.set_definition(def.iter().map(|c| folder.fold_constructor(db, model, c)))
	}
	enumeration
}

/// Fold an enumeration ID
pub fn fold_enumeration_id<F: Folder + ?Sized>(
	folder: &mut F,
	_db: &dyn Thir,
	_model: &Model,
	a: EnumerationId,
) -> EnumerationId {
	folder
		.replacement_map()
		.get_enumeration(a)
		.expect("Enumeration has not been added to destination model")
}

/// Fold an enum member ID
pub fn fold_enum_member_id<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	e: EnumMemberId,
) -> EnumMemberId {
	EnumMemberId::new(
		folder.fold_enumeration_id(db, model, e.enumeration_id()),
		e.member_index(),
	)
}

/// Fold a constructor
pub fn fold_constructor<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	c: &Constructor,
) -> Constructor {
	Constructor {
		name: c.name,
		parameters: c.parameters.as_ref().map(|ps| {
			ps.iter()
				.map(|p| {
					folder.add_declaration(db, model, *p);
					folder.fold_declaration_id(db, model, *p)
				})
				.collect()
		}),
	}
}

/// Add the folded version of this function item into the destination model.
pub fn add_function<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	f: FunctionId,
) -> FunctionId {
	let function = folder.fold_function(db, model, &*model[f]);
	let idx = folder
		.model()
		.add_function(Item::new(function, model[f].origin()));
	folder.replacement_map().insert_function(f, idx);
	idx
}

/// Fold a function item.
///
/// Note that this doesn't fold the body, which has to be processed at the end
/// since it may refer to items which have not been added yet.
pub fn fold_function<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	f: &Function,
) -> Function {
	let mut function = Function::new(f.name(), folder.fold_domain(db, model, f.domain()));
	function.annotations_mut().extend(
		f.annotations()
			.iter()
			.map(|ann| folder.fold_expression(db, model, ann)),
	);
	function.set_parameters(f.parameters().iter().map(|p| {
		folder.add_declaration(db, model, *p);
		folder.fold_declaration_id(db, model, *p)
	}));
	function.set_type_inst_vars(f.type_inst_vars().iter().cloned());
	function
}

/// Fold an function ID
pub fn fold_function_id<F: Folder + ?Sized>(
	folder: &mut F,
	_db: &dyn Thir,
	_model: &Model,
	f: FunctionId,
) -> FunctionId {
	folder
		.replacement_map()
		.get_function(f)
		.expect("Function has not been added to destination model")
}

/// Fold the body of a function.
///
/// This is separate because bodies must be processed once the function items
/// have been added, since they can refer to items which have not been added to
/// the destination model yet.
pub fn fold_function_body<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	f: FunctionId,
) {
	let dst = folder.fold_function_id(db, model, f);
	let folded = folder.fold_expression(db, model, model[f].body().unwrap());
	folder.model()[dst].set_body(folded);
}

/// Add the folded version of this output item into the destination model.
pub fn add_output<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	o: OutputId,
) -> OutputId {
	let output = folder.fold_output(db, model, &*model[o]);
	let idx = folder
		.model()
		.add_output(Item::new(output, model[o].origin()));
	folder.replacement_map().insert_output(o, idx);
	idx
}

/// Fold an output item
pub fn fold_output<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	o: &Output,
) -> Output {
	let mut output = Output::new(folder.fold_expression(db, model, o.expression()));
	if let Some(section) = o.section() {
		output.set_section(folder.fold_expression(db, model, section));
	}
	output
}

/// Fold an output ID
pub fn fold_output_id<F: Folder + ?Sized>(
	folder: &mut F,
	_db: &dyn Thir,
	_model: &Model,
	a: OutputId,
) -> OutputId {
	folder
		.replacement_map()
		.get_output(a)
		.expect("Output item has not been added to destination model")
}

/// Add the folded version of the solve item into the destination model.
pub fn add_solve<F: Folder + ?Sized>(folder: &mut F, db: &dyn Thir, model: &Model) {
	let solve = folder.fold_solve(db, model, &**model.solve().unwrap());
	folder
		.model()
		.set_solve(Item::new(solve, model.solve().unwrap().origin()));
}

/// Fold the solve item
pub fn fold_solve<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	s: &Solve,
) -> Solve {
	let mut solve = match s.goal() {
		Goal::Maximize { objective } => {
			Solve::maximize(folder.fold_declaration_id(db, model, *objective))
		}
		Goal::Minimize { objective } => {
			Solve::minimize(folder.fold_declaration_id(db, model, *objective))
		}
		Goal::Satisfy => Solve::satisfy(),
	};
	solve.annotations_mut().extend(
		s.annotations()
			.iter()
			.map(|ann| folder.fold_expression(db, model, ann)),
	);
	solve
}

/// Fold an identifier
pub fn fold_identifier<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	identifier: &ResolvedIdentifier,
) -> ResolvedIdentifier {
	match identifier {
		ResolvedIdentifier::Annotation(idx) => {
			ResolvedIdentifier::Annotation(folder.fold_annotation_id(db, model, *idx))
		}
		ResolvedIdentifier::AnnotationDestructure(idx) => {
			ResolvedIdentifier::AnnotationDestructure(folder.fold_annotation_id(db, model, *idx))
		}
		ResolvedIdentifier::Declaration(idx) => {
			ResolvedIdentifier::Declaration(folder.fold_declaration_id(db, model, *idx))
		}
		ResolvedIdentifier::Enumeration(idx) => {
			ResolvedIdentifier::Enumeration(folder.fold_enumeration_id(db, model, *idx))
		}
		ResolvedIdentifier::EnumerationDestructure(idx, kind) => {
			ResolvedIdentifier::EnumerationDestructure(
				folder.fold_enum_member_id(db, model, *idx),
				*kind,
			)
		}
		ResolvedIdentifier::EnumerationMember(idx, kind) => ResolvedIdentifier::EnumerationMember(
			folder.fold_enum_member_id(db, model, *idx),
			*kind,
		),
		ResolvedIdentifier::Function(idx) => {
			ResolvedIdentifier::Function(folder.fold_function_id(db, model, *idx))
		}
		ResolvedIdentifier::PolymorphicFunction(idx, tvs) => {
			ResolvedIdentifier::PolymorphicFunction(
				folder.fold_function_id(db, model, *idx),
				tvs.clone(),
			)
		}
	}
}

/// Fold an array literal
pub fn fold_array_literal<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	al: &ArrayLiteral,
) -> ArrayLiteral {
	ArrayLiteral(
		al.0.iter()
			.map(|e| folder.fold_expression(db, model, e))
			.collect(),
	)
}

/// Fold a set literal
pub fn fold_set_literal<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	sl: &SetLiteral,
) -> SetLiteral {
	SetLiteral(
		sl.0.iter()
			.map(|e| folder.fold_expression(db, model, e))
			.collect(),
	)
}

/// Fold a tuple literal
pub fn fold_tuple_literal<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	tl: &TupleLiteral,
) -> TupleLiteral {
	TupleLiteral(
		tl.0.iter()
			.map(|e| folder.fold_expression(db, model, e))
			.collect(),
	)
}

/// Fold a record literal
pub fn fold_record_literal<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	rl: &RecordLiteral,
) -> RecordLiteral {
	RecordLiteral(
		rl.0.iter()
			.map(|(i, e)| (*i, folder.fold_expression(db, model, e)))
			.collect(),
	)
}

/// Fold an array comprehension
pub fn fold_array_comprehension<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	c: &ArrayComprehension,
) -> ArrayComprehension {
	ArrayComprehension {
		generators: c
			.generators
			.iter()
			.map(|g| folder.fold_generator(db, model, g))
			.collect(),
		indices: c
			.indices
			.as_ref()
			.map(|i| Box::new(folder.fold_expression(db, model, &i))),
		template: Box::new(folder.fold_expression(db, model, &c.template)),
	}
}

/// Fold a set comprehension
pub fn fold_set_comprehension<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	c: &SetComprehension,
) -> SetComprehension {
	SetComprehension {
		generators: c
			.generators
			.iter()
			.map(|g| folder.fold_generator(db, model, g))
			.collect(),
		template: Box::new(folder.fold_expression(db, model, &c.template)),
	}
}

/// Fold an array access expression
pub fn fold_array_access<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	aa: &ArrayAccess,
) -> ArrayAccess {
	ArrayAccess {
		collection: Box::new(folder.fold_expression(db, model, &aa.collection)),
		indices: Box::new(folder.fold_expression(db, model, &aa.indices)),
	}
}

/// Fold a tuple access expression (does not fold the field integer)
pub fn fold_tuple_access<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	ta: &TupleAccess,
) -> TupleAccess {
	TupleAccess {
		tuple: Box::new(folder.fold_expression(db, model, &ta.tuple)),
		field: ta.field,
	}
}

/// Fold an record access expression (does not fold the field identifier)
pub fn fold_record_access<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	ra: &RecordAccess,
) -> RecordAccess {
	RecordAccess {
		record: Box::new(folder.fold_expression(db, model, &ra.record)),
		field: ra.field,
	}
}

/// Fold an if-then-else expression
pub fn fold_if_then_else<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	ite: &IfThenElse,
) -> IfThenElse {
	IfThenElse {
		branches: ite
			.branches
			.iter()
			.map(|b| {
				Branch::new(
					folder.fold_expression(db, model, &b.condition),
					folder.fold_expression(db, model, &b.result),
				)
			})
			.collect(),
		else_result: Box::new(folder.fold_expression(db, model, &ite.else_result)),
	}
}

/// Fold a case expression
pub fn fold_case<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	c: &Case,
) -> Case {
	Case {
		scrutinee: Box::new(folder.fold_expression(db, model, &c.scrutinee)),
		branches: c
			.branches
			.iter()
			.map(|b| {
				CaseBranch::new(
					folder.fold_pattern(db, model, &b.pattern),
					folder.fold_expression(db, model, &b.result),
				)
			})
			.collect(),
	}
}

/// Fold a call expression
pub fn fold_call<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	c: &Call,
) -> Call {
	Call {
		function: Box::new(folder.fold_expression(db, model, &c.function)),
		arguments: c
			.arguments
			.iter()
			.map(|arg| folder.fold_expression(db, model, arg))
			.collect(),
	}
}

/// Fold a let expression
pub fn fold_let<F: Folder + ?Sized>(folder: &mut F, db: &dyn Thir, model: &Model, l: &Let) -> Let {
	Let {
		items: l
			.items
			.iter()
			.map(|i| match i {
				LetItem::Constraint(c) => {
					folder.add_constraint(db, model, *c);
					LetItem::Constraint(folder.fold_constraint_id(db, model, *c))
				}
				LetItem::Declaration(d) => {
					folder.add_declaration(db, model, *d);
					LetItem::Declaration(folder.fold_declaration_id(db, model, *d))
				}
			})
			.collect(),
		in_expression: Box::new(folder.fold_expression(db, model, &l.in_expression)),
	}
}

/// Fold a lambda expression
pub fn fold_lambda<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	l: &Lambda,
) -> Lambda {
	Lambda {
		domain: Box::new(folder.fold_domain(db, model, &l.domain)),
		parameters: l
			.parameters
			.iter()
			.map(|p| {
				folder.add_declaration(db, model, *p);
				folder.fold_declaration_id(db, model, *p)
			})
			.collect(),
		body: Box::new(folder.fold_expression(db, model, &l.body)),
	}
}

/// Fold an expression.
///
/// First visits annotations and then calls the specific `folder.fold_foo()` method for the kind of expression
pub fn fold_expression<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	expression: &Expression,
) -> Expression {
	let origin = expression.origin();
	let mut e = match &**expression {
		ExpressionData::Absent => alloc_expression(db, origin, Absent, folder),
		ExpressionData::BooleanLiteral(b) => {
			alloc_expression(db, origin, folder.fold_boolean(db, model, *b), folder)
		}
		ExpressionData::IntegerLiteral(i) => {
			alloc_expression(db, origin, folder.fold_integer(db, model, *i), folder)
		}
		ExpressionData::FloatLiteral(f) => {
			alloc_expression(db, origin, folder.fold_float(db, model, *f), folder)
		}
		ExpressionData::StringLiteral(s) => {
			alloc_expression(db, origin, folder.fold_string(db, model, s), folder)
		}
		ExpressionData::Infinity => alloc_expression(db, origin, Infinity, folder),
		ExpressionData::Identifier(i) => {
			alloc_expression(db, origin, folder.fold_identifier(db, model, i), folder)
		}
		ExpressionData::ArrayLiteral(al) => {
			alloc_expression(db, origin, folder.fold_array_literal(db, model, al), folder)
		}
		ExpressionData::SetLiteral(sl) => {
			alloc_expression(db, origin, folder.fold_set_literal(db, model, sl), folder)
		}
		ExpressionData::TupleLiteral(tl) => {
			alloc_expression(db, origin, folder.fold_tuple_literal(db, model, tl), folder)
		}
		ExpressionData::RecordLiteral(rl) => alloc_expression(
			db,
			origin,
			folder.fold_record_literal(db, model, rl),
			folder,
		),
		ExpressionData::ArrayComprehension(c) => alloc_expression(
			db,
			origin,
			folder.fold_array_comprehension(db, model, c),
			folder,
		),
		ExpressionData::SetComprehension(c) => alloc_expression(
			db,
			origin,
			folder.fold_set_comprehension(db, model, c),
			folder,
		),
		ExpressionData::ArrayAccess(aa) => {
			alloc_expression(db, origin, folder.fold_array_access(db, model, aa), folder)
		}
		ExpressionData::TupleAccess(ta) => {
			alloc_expression(db, origin, folder.fold_tuple_access(db, model, ta), folder)
		}
		ExpressionData::RecordAccess(ra) => {
			alloc_expression(db, origin, folder.fold_record_access(db, model, ra), folder)
		}
		ExpressionData::IfThenElse(ite) => {
			alloc_expression(db, origin, folder.fold_if_then_else(db, model, ite), folder)
		}
		ExpressionData::Case(c) => {
			alloc_expression(db, origin, folder.fold_case(db, model, c), folder)
		}
		ExpressionData::Call(c) => {
			alloc_expression(db, origin, folder.fold_call(db, model, c), folder)
		}
		ExpressionData::Let(l) => {
			alloc_expression(db, origin, folder.fold_let(db, model, l), folder)
		}
		ExpressionData::Lambda(l) => {
			alloc_expression(db, origin, folder.fold_lambda(db, model, l), folder)
		}
	};
	e.annotations_mut().extend(
		expression
			.annotations()
			.iter()
			.map(|ann| folder.fold_expression(db, model, ann)),
	);
	e
}

/// Fold a generator
pub fn fold_generator<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	generator: &Generator,
) -> Generator {
	match generator {
		Generator::Assignment {
			assignment,
			where_clause,
		} => {
			folder.add_declaration(db, model, *assignment);
			Generator::Assignment {
				assignment: folder.fold_declaration_id(db, model, *assignment),
				where_clause: where_clause
					.as_ref()
					.map(|w| folder.fold_expression(db, model, w)),
			}
		}
		Generator::Iterator {
			declarations,
			collection,
			where_clause,
		} => Generator::Iterator {
			declarations: declarations
				.iter()
				.map(|d| {
					folder.add_declaration(db, model, *d);
					folder.fold_declaration_id(db, model, *d)
				})
				.collect(),
			collection: folder.fold_expression(db, model, collection),
			where_clause: where_clause
				.as_ref()
				.map(|w| folder.fold_expression(db, model, w)),
		},
	}
}

/// Fold a domain
pub fn fold_domain<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	domain: &Domain,
) -> Domain {
	let origin = domain.origin();
	let ty = domain.ty();
	match &**domain {
		DomainData::Array(dims, elem) => {
			let dimensions = folder.fold_domain(db, model, &dims);
			let element = folder.fold_domain(db, model, &elem);
			Domain::array(db, origin, dimensions, element)
		}
		DomainData::Bounded(e) => {
			let inst = ty.inst(db.upcast()).unwrap();
			let opt = ty.opt(db.upcast()).unwrap();
			let expression = folder.fold_expression(db, model, &e);
			Domain::bounded(db, origin, inst, opt, expression)
		}
		DomainData::Record(items) => {
			let fields = items
				.iter()
				.map(|(i, d)| (*i, folder.fold_domain(db, model, d)))
				.collect::<Vec<_>>();
			Domain::record(db, origin, fields)
		}
		DomainData::Set(d) => {
			let inst = ty.inst(db.upcast()).unwrap();
			let opt = ty.opt(db.upcast()).unwrap();
			let element = folder.fold_domain(db, model, &d);
			Domain::set(db, origin, inst, opt, element)
		}
		DomainData::Tuple(items) => {
			let fields = items
				.iter()
				.map(|d| folder.fold_domain(db, model, d))
				.collect::<Vec<_>>();
			Domain::tuple(db, origin, fields)
		}
		DomainData::Unbounded => Domain::unbounded(origin, ty),
	}
}

/// Fold a pattern
pub fn fold_pattern<F: Folder + ?Sized>(
	folder: &mut F,
	db: &dyn Thir,
	model: &Model,
	pattern: &Pattern,
) -> Pattern {
	let origin = pattern.origin();
	let new_data = match &**pattern {
		PatternData::AnnotationConstructor { item, args } => PatternData::AnnotationConstructor {
			item: folder.fold_annotation_id(db, model, *item),
			args: args
				.iter()
				.map(|arg| folder.fold_pattern(db, model, arg))
				.collect(),
		},
		PatternData::Anonymous(ty) => PatternData::Anonymous(*ty),
		PatternData::EnumConstructor { member, kind, args } => PatternData::EnumConstructor {
			member: folder.fold_enum_member_id(db, model, *member),
			kind: *kind,
			args: args
				.iter()
				.map(|arg| folder.fold_pattern(db, model, arg))
				.collect(),
		},
		PatternData::Expression(e) => {
			PatternData::Expression(Box::new(folder.fold_expression(db, model, &e)))
		}
		PatternData::Record(fs) => PatternData::Record(
			fs.iter()
				.map(|(i, p)| (*i, folder.fold_pattern(db, model, p)))
				.collect(),
		),
		PatternData::Tuple(fs) => PatternData::Tuple(
			fs.iter()
				.map(|p| folder.fold_pattern(db, model, p))
				.collect(),
		),
	};
	Pattern::new(new_data, origin)
}
