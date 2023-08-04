use rustc_hash::FxHashMap;

use crate::thir::{db::Thir, source::Origin, *};

/// Replacement map for references to items
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReplacementMap<Dst, Src = ()> {
	annotations: FxHashMap<AnnotationId<Src>, AnnotationId<Dst>>,
	constraints: FxHashMap<ConstraintId<Src>, ConstraintId<Dst>>,
	declarations: FxHashMap<DeclarationId<Src>, DeclarationId<Dst>>,
	enumerations: FxHashMap<EnumerationId<Src>, EnumerationId<Dst>>,
	functions: FxHashMap<FunctionId<Src>, FunctionId<Dst>>,
	outputs: FxHashMap<OutputId<Src>, OutputId<Dst>>,
}

impl<Src: Marker, Dst: Marker> ReplacementMap<Dst, Src> {
	/// Get the replacement for an item
	pub fn get_item(&self, src: ItemId<Src>) -> Option<ItemId<Dst>> {
		match src {
			ItemId::Annotation(a) => self.get_annotation(a).map(ItemId::from),
			ItemId::Constraint(c) => self.get_constraint(c).map(ItemId::from),
			ItemId::Declaration(d) => self.get_declaration(d).map(ItemId::from),
			ItemId::Enumeration(e) => self.get_enumeration(e).map(ItemId::from),
			ItemId::Function(f) => self.get_function(f).map(ItemId::from),
			ItemId::Output(o) => self.get_output(o).map(ItemId::from),
			ItemId::Solve => Some(ItemId::Solve),
		}
	}

	/// Get the replacement for this annotation ID if any
	pub fn get_annotation(&self, src: AnnotationId<Src>) -> Option<AnnotationId<Dst>> {
		self.annotations.get(&src).copied()
	}

	/// Insert an annotation ID into the replace map
	pub fn insert_annotation(&mut self, src: AnnotationId<Src>, dst: AnnotationId<Dst>) {
		self.annotations.insert(src, dst);
	}

	/// Get the replacement for this constraint ID if any
	pub fn get_constraint(&self, src: ConstraintId<Src>) -> Option<ConstraintId<Dst>> {
		self.constraints.get(&src).copied()
	}

	/// Insert an constraint ID into the replace map
	pub fn insert_constraint(&mut self, src: ConstraintId<Src>, dst: ConstraintId<Dst>) {
		self.constraints.insert(src, dst);
	}

	/// Get the replacement for this declaration ID if any
	pub fn get_declaration(&self, src: DeclarationId<Src>) -> Option<DeclarationId<Dst>> {
		self.declarations.get(&src).copied()
	}

	/// Insert an declaration ID into the replace map
	pub fn insert_declaration(&mut self, src: DeclarationId<Src>, dst: DeclarationId<Dst>) {
		self.declarations.insert(src, dst);
	}

	/// Get the replacement for this enumeration ID if any
	pub fn get_enumeration(&self, src: EnumerationId<Src>) -> Option<EnumerationId<Dst>> {
		self.enumerations.get(&src).copied()
	}

	/// Insert an enumeration ID into the replace map
	pub fn insert_enumeration(&mut self, src: EnumerationId<Src>, dst: EnumerationId<Dst>) {
		self.enumerations.insert(src, dst);
	}

	/// Get the replacement for this enum member ID if any
	pub fn get_enum_member(&self, src: EnumMemberId<Src>) -> Option<EnumMemberId<Dst>> {
		self.get_enumeration(src.enumeration_id())
			.map(|e| EnumMemberId::new(e, src.member_index()))
	}

	/// Get the replacement for this function ID if any
	pub fn get_function(&self, src: FunctionId<Src>) -> Option<FunctionId<Dst>> {
		self.functions.get(&src).copied()
	}

	/// Insert an function ID into the replace map
	pub fn insert_function(&mut self, src: FunctionId<Src>, dst: FunctionId<Dst>) {
		self.functions.insert(src, dst);
	}

	/// Get the replacement for this output ID if any
	pub fn get_output(&self, src: OutputId<Src>) -> Option<OutputId<Dst>> {
		self.outputs.get(&src).copied()
	}

	/// Insert an output ID into the replace map
	pub fn insert_output(&mut self, src: OutputId<Src>, dst: OutputId<Dst>) {
		self.outputs.insert(src, dst);
	}
}

/// Trait for folding a model, and adding a transformed version of the items into another model.
pub trait Folder<'a, Dst: Marker, Src: Marker = ()> {
	/// Get the replacement map
	fn replacement_map(&mut self) -> &mut ReplacementMap<Dst, Src>;

	/// Get the destination model into which the nodes are added
	fn model(&mut self) -> &mut Model<Dst>;

	/// Add the items from the model into the destination model
	fn add_model(&mut self, db: &'a dyn Thir, model: &'a Model<Src>) {
		add_model(self, db, model)
	}

	/// Add the folded version of this item to the destination model (and also to the replacement map)
	fn add_item(&mut self, db: &'a dyn Thir, model: &'a Model<Src>, item: ItemId<Src>) {
		add_item(self, db, model, item)
	}

	/// Add the folded version of this annotation item into the destination model.
	fn add_annotation(&mut self, db: &'a dyn Thir, model: &'a Model<Src>, a: AnnotationId<Src>) {
		add_annotation(self, db, model, a);
	}

	/// Fold an annotation item.
	fn fold_annotation(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		a: &'a Annotation<Src>,
	) -> Annotation<Dst> {
		fold_annotation(self, db, model, a)
	}

	/// Fold an annotation ID.
	fn fold_annotation_id(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		a: AnnotationId<Src>,
	) -> AnnotationId<Dst> {
		fold_annotation_id(self, db, model, a)
	}

	/// Add the folded version of this constraint item into the destination model.
	fn add_constraint(&mut self, db: &'a dyn Thir, model: &'a Model<Src>, c: ConstraintId<Src>) {
		add_constraint(self, db, model, c);
	}

	/// Fold a constraint item.
	fn fold_constraint(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		c: &'a Constraint<Src>,
	) -> Constraint<Dst> {
		fold_constraint(self, db, model, c)
	}

	/// Fold a constraint ID.
	fn fold_constraint_id(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		c: ConstraintId<Src>,
	) -> ConstraintId<Dst> {
		fold_constraint_id(self, db, model, c)
	}

	/// Add the folded version of this declaration item into the destination model.
	fn add_declaration(&mut self, db: &'a dyn Thir, model: &'a Model<Src>, d: DeclarationId<Src>) {
		add_declaration(self, db, model, d);
	}

	/// Fold a declaration item.
	fn fold_declaration(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		d: &'a Declaration<Src>,
	) -> Declaration<Dst> {
		fold_declaration(self, db, model, d)
	}

	/// Fold a declaration ID.
	fn fold_declaration_id(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		d: DeclarationId<Src>,
	) -> DeclarationId<Dst> {
		fold_declaration_id(self, db, model, d)
	}

	/// Add the folded version of this enumeration item into the destination model.
	fn add_enumeration(&mut self, db: &'a dyn Thir, model: &'a Model<Src>, e: EnumerationId<Src>) {
		add_enumeration(self, db, model, e);
	}

	/// Fold an enumeration item.
	fn fold_enumeration(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		e: &'a Enumeration<Src>,
	) -> Enumeration<Dst> {
		fold_enumeration(self, db, model, e)
	}

	/// Fold an enumeration ID.
	fn fold_enumeration_id(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		e: EnumerationId<Src>,
	) -> EnumerationId<Dst> {
		fold_enumeration_id(self, db, model, e)
	}

	/// Fold an enum member ID.
	fn fold_enum_member_id(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		e: EnumMemberId<Src>,
	) -> EnumMemberId<Dst> {
		fold_enum_member_id(self, db, model, e)
	}

	/// Fold a constructor
	fn fold_constructor(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		c: &'a Constructor<Src>,
	) -> Constructor<Dst> {
		fold_constructor(self, db, model, c)
	}

	/// Add the folded version of this function item into the destination model.
	fn add_function(&mut self, db: &'a dyn Thir, model: &'a Model<Src>, f: FunctionId<Src>) {
		add_function(self, db, model, f);
	}

	/// Fold a function item.
	///
	/// Note that this doesn't fold the body, which has to be processed at the end
	/// since it may refer to items which have not been added yet.
	fn fold_function(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		f: &'a Function<Src>,
	) -> Function<Dst> {
		fold_function(self, db, model, f)
	}

	/// Fold a function ID.
	fn fold_function_id(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		f: FunctionId<Src>,
	) -> FunctionId<Dst> {
		fold_function_id(self, db, model, f)
	}

	/// Fold the body of a function.
	///
	/// This is separate because bodies must be processed once the function items
	/// have been added, since they can refer to items which have not been added to
	/// the destination model yet.
	fn fold_function_body(&mut self, db: &'a dyn Thir, model: &'a Model<Src>, f: FunctionId<Src>) {
		fold_function_body(self, db, model, f)
	}

	/// Add the folded version of this output item into the destination model.
	fn add_output(&mut self, db: &'a dyn Thir, model: &'a Model<Src>, o: OutputId<Src>) {
		add_output(self, db, model, o);
	}

	/// Fold an output item.
	fn fold_output(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		o: &'a Output<Src>,
	) -> Output<Dst> {
		fold_output(self, db, model, o)
	}

	/// Add the folded version of the solve item into the destination model.
	fn add_solve(&mut self, db: &'a dyn Thir, model: &'a Model<Src>) {
		add_solve(self, db, model)
	}

	/// Fold the solve item.
	fn fold_solve(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		s: &'a Solve<Src>,
	) -> Solve<Dst> {
		fold_solve(self, db, model, s)
	}

	/// Fold an expression
	fn fold_expression(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		expression: &'a Expression<Src>,
	) -> Expression<Dst> {
		fold_expression(self, db, model, expression)
	}

	/// Fold a boolean literal
	fn fold_boolean(
		&mut self,
		_db: &'a dyn Thir,
		_model: &'a Model<Src>,
		b: BooleanLiteral,
	) -> BooleanLiteral {
		b
	}

	/// Fold an integer literal
	fn fold_integer(
		&mut self,
		_db: &'a dyn Thir,
		_model: &'a Model<Src>,
		i: IntegerLiteral,
	) -> IntegerLiteral {
		i
	}

	/// Fold a float literal
	fn fold_float(
		&mut self,
		_db: &'a dyn Thir,
		_model: &'a Model<Src>,
		f: FloatLiteral,
	) -> FloatLiteral {
		f
	}

	/// Fold a string literal
	fn fold_string(
		&mut self,
		_db: &'a dyn Thir,
		_model: &'a Model<Src>,
		s: &'a StringLiteral,
	) -> StringLiteral {
		s.clone()
	}

	/// Fold an identifier
	fn fold_identifier(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		identifier: &'a ResolvedIdentifier<Src>,
	) -> ResolvedIdentifier<Dst> {
		fold_identifier(self, db, model, identifier)
	}

	/// Fold a callable
	fn fold_callable(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		function: &'a Callable<Src>,
	) -> Callable<Dst> {
		fold_callable(self, db, model, function)
	}

	/// Fold an array literal
	fn fold_array_literal(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		al: &'a ArrayLiteral<Src>,
	) -> ArrayLiteral<Dst> {
		fold_array_literal(self, db, model, al)
	}

	/// Fold a set literal
	fn fold_set_literal(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		sl: &'a SetLiteral<Src>,
	) -> SetLiteral<Dst> {
		fold_set_literal(self, db, model, sl)
	}

	/// Fold a tuple literal
	fn fold_tuple_literal(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		tl: &'a TupleLiteral<Src>,
	) -> TupleLiteral<Dst> {
		fold_tuple_literal(self, db, model, tl)
	}

	/// Fold a record literal
	fn fold_record_literal(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		rl: &'a RecordLiteral<Src>,
	) -> RecordLiteral<Dst> {
		fold_record_literal(self, db, model, rl)
	}

	/// Fold an array comprehension
	fn fold_array_comprehension(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		c: &'a ArrayComprehension<Src>,
	) -> ArrayComprehension<Dst> {
		fold_array_comprehension(self, db, model, c)
	}
	/// Fold a set comprehension
	fn fold_set_comprehension(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		c: &'a SetComprehension<Src>,
	) -> SetComprehension<Dst> {
		fold_set_comprehension(self, db, model, c)
	}

	/// Fold an array access
	fn fold_array_access(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		aa: &'a ArrayAccess<Src>,
	) -> ArrayAccess<Dst> {
		fold_array_access(self, db, model, aa)
	}

	/// Fold a tuple access
	fn fold_tuple_access(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		ta: &'a TupleAccess<Src>,
	) -> TupleAccess<Dst> {
		fold_tuple_access(self, db, model, ta)
	}

	/// Fold a record access
	fn fold_record_access(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		ra: &'a RecordAccess<Src>,
	) -> RecordAccess<Dst> {
		fold_record_access(self, db, model, ra)
	}

	/// Fold an if-then-else expression
	fn fold_if_then_else(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		ite: &'a IfThenElse<Src>,
	) -> IfThenElse<Dst> {
		fold_if_then_else(self, db, model, ite)
	}

	/// Fold a case expression
	fn fold_case(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		c: &'a Case<Src>,
	) -> Case<Dst> {
		fold_case(self, db, model, c)
	}

	/// Fold a call expression
	fn fold_call(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		call: &'a Call<Src>,
	) -> Call<Dst> {
		fold_call(self, db, model, call)
	}

	/// Fold a let expression
	fn fold_let(&mut self, db: &'a dyn Thir, model: &'a Model<Src>, l: &'a Let<Src>) -> Let<Dst> {
		fold_let(self, db, model, l)
	}

	/// Fold a lambda expression
	fn fold_lambda(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		l: &'a Lambda<Src>,
	) -> Lambda<Dst> {
		fold_lambda(self, db, model, l)
	}

	/// Fold a comprehension generator
	fn fold_generator(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		generator: &'a Generator<Src>,
	) -> Generator<Dst> {
		fold_generator(self, db, model, generator)
	}

	/// Fold a domain
	fn fold_domain(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		domain: &'a Domain<Src>,
	) -> Domain<Dst> {
		fold_domain(self, db, model, domain)
	}

	/// Fold a case pattern
	fn fold_pattern(
		&mut self,
		db: &'a dyn Thir,
		model: &'a Model<Src>,
		pattern: &'a Pattern<Src>,
	) -> Pattern<Dst> {
		fold_pattern(self, db, model, pattern)
	}
}

fn alloc_expression<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	db: &'a dyn Thir,
	origin: impl Into<Origin>,
	value: impl ExpressionBuilder<U>,
	folder: &mut F,
) -> Expression<U> {
	Expression::new(db, folder.model(), origin, value)
}

/// Fold the model by adding folded versions of the items into the destination model.
///
/// First, we add each folded item into the destination model, except for function bodies.
/// The function bodies are then added once all of the items are made available.
pub fn add_model<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
) {
	// Add items to the destination model
	for item in model.top_level_items() {
		folder.add_item(db, model, item);
	}
	// Now that all items have been added, we can process function bodies
	for (f, i) in model.all_functions() {
		if i.body().is_some() {
			folder.fold_function_body(db, model, f);
		}
	}
}

/// Fold an item
pub fn add_item<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	item: ItemId<T>,
) {
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
pub fn add_annotation<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	a: AnnotationId<T>,
) -> AnnotationId<U> {
	let annotation = folder.fold_annotation(db, model, &model[a]);
	let idx = folder
		.model()
		.add_annotation(Item::new(annotation, model[a].origin()));
	folder.replacement_map().insert_annotation(a, idx);
	idx
}

/// Fold an annotation item
pub fn fold_annotation<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	a: &'a Annotation<T>,
) -> Annotation<U> {
	folder.fold_constructor(db, model, a).into()
}

/// Fold an annotation ID
pub fn fold_annotation_id<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	_db: &'a dyn Thir,
	_model: &'a Model<T>,
	a: AnnotationId<T>,
) -> AnnotationId<U> {
	folder
		.replacement_map()
		.get_annotation(a)
		.expect("Annotation has not been added to destination model")
}

/// Add the folded version of this constraint item into the destination model.
pub fn add_constraint<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	c: ConstraintId<T>,
) -> ConstraintId<U> {
	let constraint = folder.fold_constraint(db, model, &model[c]);
	let idx = folder
		.model()
		.add_constraint(Item::new(constraint, model[c].origin()));
	folder.replacement_map().insert_constraint(c, idx);
	idx
}

/// Fold a constraint item
pub fn fold_constraint<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	c: &'a Constraint<T>,
) -> Constraint<U> {
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
pub fn fold_constraint_id<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	_db: &'a dyn Thir,
	_model: &'a Model<T>,
	a: ConstraintId<T>,
) -> ConstraintId<U> {
	folder
		.replacement_map()
		.get_constraint(a)
		.expect("Constraint has not been added to destination model")
}

/// Add the folded version of this declaration item into the destination model.
pub fn add_declaration<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	d: DeclarationId<T>,
) -> DeclarationId<U> {
	let declaration = folder.fold_declaration(db, model, &model[d]);
	let idx = folder
		.model()
		.add_declaration(Item::new(declaration, model[d].origin()));
	folder.replacement_map().insert_declaration(d, idx);
	idx
}

/// Fold a declaration item
pub fn fold_declaration<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	d: &'a Declaration<T>,
) -> Declaration<U> {
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
		let def = folder.fold_expression(db, model, def);
		declaration.set_definition(def);
		declaration.validate(db);
	}
	declaration
}

/// Fold a declaration ID
pub fn fold_declaration_id<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	_db: &'a dyn Thir,
	_model: &'a Model<T>,
	d: DeclarationId<T>,
) -> DeclarationId<U> {
	folder
		.replacement_map()
		.get_declaration(d)
		.expect("Declaration has not been added to destination model")
}

/// Add the folded version of this enumeration item into the destination model.
pub fn add_enumeration<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	e: EnumerationId<T>,
) -> EnumerationId<U> {
	let enumeration = folder.fold_enumeration(db, model, &model[e]);
	let idx = folder
		.model()
		.add_enumeration(Item::new(enumeration, model[e].origin()));
	folder.replacement_map().insert_enumeration(e, idx);
	idx
}

/// Fold an enumeration item
pub fn fold_enumeration<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	e: &'a Enumeration<T>,
) -> Enumeration<U> {
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
pub fn fold_enumeration_id<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	_db: &'a dyn Thir,
	_model: &'a Model<T>,
	a: EnumerationId<T>,
) -> EnumerationId<U> {
	folder
		.replacement_map()
		.get_enumeration(a)
		.expect("Enumeration has not been added to destination model")
}

/// Fold an enum member ID
pub fn fold_enum_member_id<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	e: EnumMemberId<T>,
) -> EnumMemberId<U> {
	EnumMemberId::new(
		folder.fold_enumeration_id(db, model, e.enumeration_id()),
		e.member_index(),
	)
}

/// Fold a constructor
pub fn fold_constructor<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	c: &'a Constructor<T>,
) -> Constructor<U> {
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
pub fn add_function<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	f: FunctionId<T>,
) -> FunctionId<U> {
	let function = folder.fold_function(db, model, &model[f]);
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
pub fn fold_function<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	f: &'a Function<T>,
) -> Function<U> {
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
pub fn fold_function_id<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	_db: &'a dyn Thir,
	_model: &'a Model<T>,
	f: FunctionId<T>,
) -> FunctionId<U> {
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
pub fn fold_function_body<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	f: FunctionId<T>,
) {
	let dst = folder.fold_function_id(db, model, f);
	let folded = folder.fold_expression(db, model, model[f].body().unwrap());
	let function = &mut folder.model()[dst];
	function.set_body(folded);
	function.validate(db);
}

/// Add the folded version of this output item into the destination model.
pub fn add_output<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	o: OutputId<T>,
) -> OutputId<U> {
	let output = folder.fold_output(db, model, &model[o]);
	let idx = folder
		.model()
		.add_output(Item::new(output, model[o].origin()));
	folder.replacement_map().insert_output(o, idx);
	idx
}

/// Fold an output item
pub fn fold_output<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	o: &'a Output<T>,
) -> Output<U> {
	let mut output = Output::new(folder.fold_expression(db, model, o.expression()));
	if let Some(section) = o.section() {
		output.set_section(folder.fold_expression(db, model, section));
	}
	output
}

/// Fold an output ID
pub fn fold_output_id<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	_db: &'a dyn Thir,
	_model: &'a Model<T>,
	a: OutputId<T>,
) -> OutputId<U> {
	folder
		.replacement_map()
		.get_output(a)
		.expect("Output item has not been added to destination model")
}

/// Add the folded version of the solve item into the destination model.
pub fn add_solve<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
) {
	let solve = folder.fold_solve(db, model, model.solve().unwrap());
	folder
		.model()
		.set_solve(Item::new(solve, model.solve().unwrap().origin()));
}

/// Fold the solve item
pub fn fold_solve<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	s: &'a Solve<T>,
) -> Solve<U> {
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
pub fn fold_identifier<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	identifier: &'a ResolvedIdentifier<T>,
) -> ResolvedIdentifier<U> {
	match identifier {
		ResolvedIdentifier::Annotation(idx) => {
			ResolvedIdentifier::Annotation(folder.fold_annotation_id(db, model, *idx))
		}
		ResolvedIdentifier::Declaration(idx) => {
			ResolvedIdentifier::Declaration(folder.fold_declaration_id(db, model, *idx))
		}
		ResolvedIdentifier::Enumeration(idx) => {
			ResolvedIdentifier::Enumeration(folder.fold_enumeration_id(db, model, *idx))
		}
		ResolvedIdentifier::EnumerationMember(idx) => {
			ResolvedIdentifier::EnumerationMember(folder.fold_enum_member_id(db, model, *idx))
		}
	}
}

/// Fold a callable
pub fn fold_callable<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	function: &'a Callable<T>,
) -> Callable<U> {
	match function {
		Callable::Annotation(a) => Callable::Annotation(folder.fold_annotation_id(db, model, *a)),
		Callable::AnnotationDestructure(a) => {
			Callable::AnnotationDestructure(folder.fold_annotation_id(db, model, *a))
		}
		Callable::EnumConstructor(e) => {
			Callable::EnumConstructor(folder.fold_enum_member_id(db, model, *e))
		}
		Callable::EnumDestructor(e) => {
			Callable::EnumDestructor(folder.fold_enum_member_id(db, model, *e))
		}
		Callable::Expression(e) => {
			Callable::Expression(Box::new(folder.fold_expression(db, model, e)))
		}
		Callable::Function(f) => Callable::Function(folder.fold_function_id(db, model, *f)),
	}
}

/// Fold an array literal
pub fn fold_array_literal<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	al: &'a ArrayLiteral<T>,
) -> ArrayLiteral<U> {
	ArrayLiteral(
		al.0.iter()
			.map(|e| folder.fold_expression(db, model, e))
			.collect(),
	)
}

/// Fold a set literal
pub fn fold_set_literal<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	sl: &'a SetLiteral<T>,
) -> SetLiteral<U> {
	SetLiteral(
		sl.0.iter()
			.map(|e| folder.fold_expression(db, model, e))
			.collect(),
	)
}

/// Fold a tuple literal
pub fn fold_tuple_literal<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	tl: &'a TupleLiteral<T>,
) -> TupleLiteral<U> {
	TupleLiteral(
		tl.0.iter()
			.map(|e| folder.fold_expression(db, model, e))
			.collect(),
	)
}

/// Fold a record literal
pub fn fold_record_literal<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	rl: &'a RecordLiteral<T>,
) -> RecordLiteral<U> {
	RecordLiteral(
		rl.0.iter()
			.map(|(i, e)| (*i, folder.fold_expression(db, model, e)))
			.collect(),
	)
}

/// Fold an array comprehension
pub fn fold_array_comprehension<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	c: &'a ArrayComprehension<T>,
) -> ArrayComprehension<U> {
	ArrayComprehension {
		generators: c
			.generators
			.iter()
			.map(|g| folder.fold_generator(db, model, g))
			.collect(),
		indices: c
			.indices
			.as_ref()
			.map(|i| Box::new(folder.fold_expression(db, model, i))),
		template: Box::new(folder.fold_expression(db, model, &c.template)),
	}
}

/// Fold a set comprehension
pub fn fold_set_comprehension<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	c: &'a SetComprehension<T>,
) -> SetComprehension<U> {
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
pub fn fold_array_access<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	aa: &'a ArrayAccess<T>,
) -> ArrayAccess<U> {
	ArrayAccess {
		collection: Box::new(folder.fold_expression(db, model, &aa.collection)),
		indices: Box::new(folder.fold_expression(db, model, &aa.indices)),
	}
}

/// Fold a tuple access expression (does not fold the field integer)
pub fn fold_tuple_access<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	ta: &'a TupleAccess<T>,
) -> TupleAccess<U> {
	TupleAccess {
		tuple: Box::new(folder.fold_expression(db, model, &ta.tuple)),
		field: ta.field,
	}
}

/// Fold an record access expression (does not fold the field identifier)
pub fn fold_record_access<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	ra: &'a RecordAccess<T>,
) -> RecordAccess<U> {
	RecordAccess {
		record: Box::new(folder.fold_expression(db, model, &ra.record)),
		field: ra.field,
	}
}

/// Fold an if-then-else expression
pub fn fold_if_then_else<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	ite: &'a IfThenElse<T>,
) -> IfThenElse<U> {
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
pub fn fold_case<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	c: &'a Case<T>,
) -> Case<U> {
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
pub fn fold_call<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	c: &'a Call<T>,
) -> Call<U> {
	Call {
		function: folder.fold_callable(db, model, &c.function),
		arguments: c
			.arguments
			.iter()
			.map(|arg| folder.fold_expression(db, model, arg))
			.collect(),
	}
}

/// Fold a let expression
pub fn fold_let<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	l: &'a Let<T>,
) -> Let<U> {
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
pub fn fold_lambda<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	l: &'a Lambda<T>,
) -> Lambda<U> {
	folder.add_function(db, model, l.0);
	Lambda(folder.fold_function_id(db, model, l.0))
}

/// Fold an expression.
///
/// First visits annotations and then calls the specific `folder.fold_foo()` method for the kind of expression
pub fn fold_expression<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	expression: &'a Expression<T>,
) -> Expression<U> {
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
pub fn fold_generator<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	generator: &'a Generator<T>,
) -> Generator<U> {
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
pub fn fold_domain<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	domain: &'a Domain<T>,
) -> Domain<U> {
	let origin = domain.origin();
	let ty = domain.ty();
	match &**domain {
		DomainData::Array(dims, elem) => {
			let opt = ty.opt(db.upcast()).unwrap();
			let dimensions = folder.fold_domain(db, model, dims);
			let element = folder.fold_domain(db, model, elem);
			Domain::array(db, origin, opt, dimensions, element)
		}
		DomainData::Bounded(e) => {
			let inst = ty.inst(db.upcast()).unwrap();
			let opt = ty.opt(db.upcast()).unwrap();
			let expression = folder.fold_expression(db, model, e);
			Domain::bounded(db, origin, inst, opt, expression)
		}
		DomainData::Record(items) => {
			let opt = ty.opt(db.upcast()).unwrap();
			let fields = items
				.iter()
				.map(|(i, d)| (*i, folder.fold_domain(db, model, d)))
				.collect::<Vec<_>>();
			Domain::record(db, origin, opt, fields)
		}
		DomainData::Set(d) => {
			let inst = ty.inst(db.upcast()).unwrap();
			let opt = ty.opt(db.upcast()).unwrap();
			let element = folder.fold_domain(db, model, d);
			Domain::set(db, origin, inst, opt, element)
		}
		DomainData::Tuple(items) => {
			let opt = ty.opt(db.upcast()).unwrap();
			let fields = items
				.iter()
				.map(|d| folder.fold_domain(db, model, d))
				.collect::<Vec<_>>();
			Domain::tuple(db, origin, opt, fields)
		}
		DomainData::Unbounded => Domain::unbounded(db, origin, ty),
	}
}

/// Fold a pattern
pub fn fold_pattern<'a, T: Marker, U: Marker, F: Folder<'a, U, T> + ?Sized>(
	folder: &mut F,
	db: &'a dyn Thir,
	model: &'a Model<T>,
	pattern: &'a Pattern<T>,
) -> Pattern<U> {
	let origin = pattern.origin();
	match &**pattern {
		PatternData::AnnotationConstructor { item, args } => {
			let item = folder.fold_annotation_id(db, model, *item);
			let args = args
				.iter()
				.map(|arg| folder.fold_pattern(db, model, arg))
				.collect::<Vec<_>>();
			Pattern::annotation_constructor(db, folder.model(), origin, item, args)
		}
		PatternData::Anonymous(ty) => Pattern::anonymous(*ty, origin),
		PatternData::EnumConstructor { member, args } => {
			let member = folder.fold_enum_member_id(db, model, *member);
			let args = args
				.iter()
				.map(|arg| folder.fold_pattern(db, model, arg))
				.collect::<Vec<_>>();
			Pattern::enum_constructor(db, folder.model(), origin, member, args)
		}
		PatternData::Expression(e) => {
			Pattern::expression(folder.fold_expression(db, model, e), origin)
		}
		PatternData::Record(fs) => {
			let fields = fs
				.iter()
				.map(|(i, p)| (*i, folder.fold_pattern(db, model, p)))
				.collect::<Vec<_>>();
			Pattern::record(db, folder.model(), origin, fields)
		}
		PatternData::Tuple(fs) => {
			let fields = fs
				.iter()
				.map(|p| folder.fold_pattern(db, model, p))
				.collect::<Vec<_>>();
			Pattern::tuple(db, folder.model(), origin, fields)
		}
	}
}
