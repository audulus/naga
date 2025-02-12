use super::{
    ast::*,
    context::Context,
    error::{Error, ErrorKind},
    Parser, Result, Span,
};
use crate::{
    AddressSpace, Binding, Block, BuiltIn, Constant, Expression, GlobalVariable, Handle,
    Interpolation, LocalVariable, ResourceBinding, ScalarKind, ShaderStage, StorageAccess,
    SwizzleComponent, Type, TypeInner, VectorSize,
};

macro_rules! qualifier_arm {
    ($src:expr, $tgt:expr, $meta:expr, $msg:literal, $errors:expr $(,)?) => {{
        if $tgt.is_some() {
            $errors.push(Error {
                kind: ErrorKind::SemanticError($msg.into()),
                meta: $meta,
            })
        }

        $tgt = Some($src);
    }};
}

pub struct VarDeclaration<'a> {
    pub qualifiers: &'a [(TypeQualifier, Span)],
    pub ty: Handle<Type>,
    pub name: Option<String>,
    pub init: Option<Handle<Constant>>,
    pub meta: Span,
}

/// Information about a builtin used in [`add_builtin`](Parser::add_builtin)
struct BuiltInData {
    /// The type of the builtin
    inner: TypeInner,
    /// The builtin class associated with
    builtin: BuiltIn,
    /// Wether it should be allowed to write to the builtin or not
    mutable: bool,
    /// The storage used for the builtin
    storage: StorageQualifier,
}

pub enum GlobalOrConstant {
    Global(Handle<GlobalVariable>),
    Constant(Handle<Constant>),
}

impl Parser {
    /// Adds a builtin and returns a variable reference to it
    fn add_builtin(
        &mut self,
        ctx: &mut Context,
        body: &mut Block,
        name: &str,
        data: BuiltInData,
        meta: Span,
    ) -> Option<VariableReference> {
        let ty = self.module.types.insert(
            Type {
                name: None,
                inner: data.inner,
            },
            meta,
        );

        let handle = self.module.global_variables.append(
            GlobalVariable {
                name: Some(name.into()),
                space: AddressSpace::Private,
                binding: None,
                ty,
                init: None,
            },
            meta,
        );

        let idx = self.entry_args.len();
        self.entry_args.push(EntryArg {
            name: None,
            binding: Binding::BuiltIn(data.builtin),
            handle,
            storage: data.storage,
        });

        self.global_variables.push((
            name.into(),
            GlobalLookup {
                kind: GlobalLookupKind::Variable(handle),
                entry_arg: Some(idx),
                mutable: data.mutable,
            },
        ));

        let expr = ctx.add_expression(Expression::GlobalVariable(handle), meta, body);
        ctx.lookup_global_var_exps.insert(
            name.into(),
            VariableReference {
                expr,
                load: true,
                mutable: data.mutable,
                constant: None,
                entry_arg: Some(idx),
            },
        );

        ctx.lookup_global_var(name)
    }

    pub(crate) fn lookup_variable(
        &mut self,
        ctx: &mut Context,
        body: &mut Block,
        name: &str,
        meta: Span,
    ) -> Option<VariableReference> {
        if let Some(local_var) = ctx.lookup_local_var(name) {
            return Some(local_var);
        }
        if let Some(global_var) = ctx.lookup_global_var(name) {
            return Some(global_var);
        }

        let data = match name {
            "gl_Position" => BuiltInData {
                inner: TypeInner::Vector {
                    size: VectorSize::Quad,
                    kind: ScalarKind::Float,
                    width: 4,
                },
                builtin: BuiltIn::Position,
                mutable: true,
                storage: StorageQualifier::Output,
            },
            "gl_FragCoord" => BuiltInData {
                inner: TypeInner::Vector {
                    size: VectorSize::Quad,
                    kind: ScalarKind::Float,
                    width: 4,
                },
                builtin: BuiltIn::Position,
                mutable: false,
                storage: StorageQualifier::Input,
            },
            "gl_GlobalInvocationID"
            | "gl_NumWorkGroups"
            | "gl_WorkGroupSize"
            | "gl_WorkGroupID"
            | "gl_LocalInvocationID" => BuiltInData {
                inner: TypeInner::Vector {
                    size: VectorSize::Tri,
                    kind: ScalarKind::Uint,
                    width: 4,
                },
                builtin: match name {
                    "gl_GlobalInvocationID" => BuiltIn::GlobalInvocationId,
                    "gl_NumWorkGroups" => BuiltIn::NumWorkGroups,
                    "gl_WorkGroupSize" => BuiltIn::WorkGroupSize,
                    "gl_WorkGroupID" => BuiltIn::WorkGroupId,
                    "gl_LocalInvocationID" => BuiltIn::LocalInvocationId,
                    _ => unreachable!(),
                },
                mutable: false,
                storage: StorageQualifier::Input,
            },
            "gl_FrontFacing" => BuiltInData {
                inner: TypeInner::Scalar {
                    kind: ScalarKind::Bool,
                    width: crate::BOOL_WIDTH,
                },
                builtin: BuiltIn::FrontFacing,
                mutable: false,
                storage: StorageQualifier::Input,
            },
            "gl_PointSize" | "gl_FragDepth" => BuiltInData {
                inner: TypeInner::Scalar {
                    kind: ScalarKind::Float,
                    width: 4,
                },
                builtin: match name {
                    "gl_PointSize" => BuiltIn::PointSize,
                    "gl_FragDepth" => BuiltIn::FragDepth,
                    _ => unreachable!(),
                },
                mutable: true,
                storage: StorageQualifier::Output,
            },
            "gl_ClipDistance" | "gl_CullDistance" => {
                let base = self.module.types.insert(
                    Type {
                        name: None,
                        inner: TypeInner::Scalar {
                            kind: ScalarKind::Float,
                            width: 4,
                        },
                    },
                    meta,
                );

                BuiltInData {
                    inner: TypeInner::Array {
                        base,
                        size: crate::ArraySize::Dynamic,
                        stride: 4,
                    },
                    builtin: match name {
                        "gl_ClipDistance" => BuiltIn::PointSize,
                        "gl_CullDistance" => BuiltIn::FragDepth,
                        _ => unreachable!(),
                    },
                    mutable: self.meta.stage == ShaderStage::Vertex,
                    storage: StorageQualifier::Output,
                }
            }
            _ => {
                let builtin = match name {
                    "gl_BaseVertex" => BuiltIn::BaseVertex,
                    "gl_BaseInstance" => BuiltIn::BaseInstance,
                    "gl_PrimitiveID" => BuiltIn::PrimitiveIndex,
                    "gl_InstanceIndex" => BuiltIn::InstanceIndex,
                    "gl_VertexIndex" => BuiltIn::VertexIndex,
                    "gl_SampleID" => BuiltIn::SampleIndex,
                    "gl_LocalInvocationIndex" => BuiltIn::LocalInvocationIndex,
                    _ => return None,
                };

                BuiltInData {
                    inner: TypeInner::Scalar {
                        kind: ScalarKind::Uint,
                        width: 4,
                    },
                    builtin,
                    mutable: false,
                    storage: StorageQualifier::Input,
                }
            }
        };

        self.add_builtin(ctx, body, name, data, meta)
    }

    pub(crate) fn field_selection(
        &mut self,
        ctx: &mut Context,
        lhs: bool,
        body: &mut Block,
        expression: Handle<Expression>,
        name: &str,
        meta: Span,
    ) -> Result<Handle<Expression>> {
        let (ty, is_pointer) = match *self.resolve_type(ctx, expression, meta)? {
            TypeInner::Pointer { base, .. } => (&self.module.types[base].inner, true),
            ref ty => (ty, false),
        };
        match *ty {
            TypeInner::Struct { ref members, .. } => {
                let index = members
                    .iter()
                    .position(|m| m.name == Some(name.into()))
                    .ok_or_else(|| Error {
                        kind: ErrorKind::UnknownField(name.into()),
                        meta,
                    })?;
                Ok(ctx.add_expression(
                    Expression::AccessIndex {
                        base: expression,
                        index: index as u32,
                    },
                    meta,
                    body,
                ))
            }
            // swizzles (xyzw, rgba, stpq)
            TypeInner::Vector { size, .. } => {
                let check_swizzle_components = |comps: &str| {
                    name.chars()
                        .map(|c| {
                            comps
                                .find(c)
                                .filter(|i| *i < size as usize)
                                .map(|i| SwizzleComponent::from_index(i as u32))
                        })
                        .collect::<Option<Vec<SwizzleComponent>>>()
                };

                let components = check_swizzle_components("xyzw")
                    .or_else(|| check_swizzle_components("rgba"))
                    .or_else(|| check_swizzle_components("stpq"));

                if let Some(components) = components {
                    if lhs {
                        let not_unique = (1..components.len())
                            .any(|i| components[i..].contains(&components[i - 1]));
                        if not_unique {
                            self.errors.push(Error {
                                kind:
                                ErrorKind::SemanticError(
                                format!(
                                    "swizzle cannot have duplicate components in left-hand-side expression for \"{:?}\"",
                                    name
                                )
                                .into(),
                            ),
                                meta ,
                            })
                        }
                    }

                    let mut pattern = [SwizzleComponent::X; 4];
                    for (pat, component) in pattern.iter_mut().zip(&components) {
                        *pat = *component;
                    }

                    // flatten nested swizzles (vec.zyx.xy.x => vec.z)
                    let mut expression = expression;
                    if let Expression::Swizzle {
                        size: _,
                        vector,
                        pattern: ref src_pattern,
                    } = ctx[expression]
                    {
                        expression = vector;
                        for pat in &mut pattern {
                            *pat = src_pattern[pat.index() as usize];
                        }
                    }

                    let size = match components.len() {
                        1 => {
                            // only single element swizzle, like pos.y, just return that component.
                            if lhs {
                                // Because of possible nested swizzles, like pos.xy.x, we have to unwrap the potential load expr.
                                if let Expression::Load { ref pointer } = ctx[expression] {
                                    expression = *pointer;
                                }
                            }
                            return Ok(ctx.add_expression(
                                Expression::AccessIndex {
                                    base: expression,
                                    index: pattern[0].index(),
                                },
                                meta,
                                body,
                            ));
                        }
                        2 => VectorSize::Bi,
                        3 => VectorSize::Tri,
                        4 => VectorSize::Quad,
                        _ => {
                            self.errors.push(Error {
                                kind: ErrorKind::SemanticError(
                                    format!("Bad swizzle size for \"{:?}\"", name).into(),
                                ),
                                meta,
                            });

                            VectorSize::Quad
                        }
                    };

                    if is_pointer {
                        // NOTE: for lhs expression, this extra load ends up as an unused expr, because the
                        // assignment will extract the pointer and use it directly anyway. Unfortunately we
                        // need it for validation to pass, as swizzles cannot operate on pointer values.
                        expression = ctx.add_expression(
                            Expression::Load {
                                pointer: expression,
                            },
                            meta,
                            body,
                        );
                    }

                    Ok(ctx.add_expression(
                        Expression::Swizzle {
                            size,
                            vector: expression,
                            pattern,
                        },
                        meta,
                        body,
                    ))
                } else {
                    Err(Error {
                        kind: ErrorKind::SemanticError(
                            format!("Invalid swizzle for vector \"{}\"", name).into(),
                        ),
                        meta,
                    })
                }
            }
            _ => Err(Error {
                kind: ErrorKind::SemanticError(
                    format!("Can't lookup field on this type \"{}\"", name).into(),
                ),
                meta,
            }),
        }
    }

    pub(crate) fn add_global_var(
        &mut self,
        ctx: &mut Context,
        body: &mut Block,
        VarDeclaration {
            qualifiers,
            ty,
            name,
            init,
            meta,
        }: VarDeclaration,
    ) -> Result<GlobalOrConstant> {
        let mut storage = StorageQualifier::AddressSpace(AddressSpace::Private);
        let mut interpolation = None;
        let mut set = None;
        let mut binding = None;
        let mut location = None;
        let mut sampling = None;
        let mut layout = None;
        let mut precision = None;
        let mut access = StorageAccess::all();

        for &(ref qualifier, meta) in qualifiers {
            match *qualifier {
                TypeQualifier::StorageQualifier(s) => {
                    if StorageQualifier::AddressSpace(AddressSpace::PushConstant) == storage
                        && s == StorageQualifier::AddressSpace(AddressSpace::Uniform)
                    {
                        // Ignore the Uniform qualifier if the space was already set to PushConstant
                        continue;
                    } else if StorageQualifier::AddressSpace(AddressSpace::Private) != storage {
                        self.errors.push(Error {
                            kind: ErrorKind::SemanticError(
                                "Cannot use more than one storage qualifier per declaration".into(),
                            ),
                            meta,
                        });
                    }

                    storage = s;
                }
                TypeQualifier::Interpolation(i) => qualifier_arm!(
                    i,
                    interpolation,
                    meta,
                    "Cannot use more than one interpolation qualifier per declaration",
                    self.errors
                ),
                TypeQualifier::Binding(r) => qualifier_arm!(
                    r,
                    binding,
                    meta,
                    "Cannot use more than one binding per declaration",
                    self.errors
                ),
                TypeQualifier::Set(s) => qualifier_arm!(
                    s,
                    set,
                    meta,
                    "Cannot use more than one binding per declaration",
                    self.errors
                ),
                TypeQualifier::Location(l) => qualifier_arm!(
                    l,
                    location,
                    meta,
                    "Cannot use more than one binding per declaration",
                    self.errors
                ),
                TypeQualifier::Sampling(s) => qualifier_arm!(
                    s,
                    sampling,
                    meta,
                    "Cannot use more than one sampling qualifier per declaration",
                    self.errors
                ),
                TypeQualifier::Layout(ref l) => qualifier_arm!(
                    l,
                    layout,
                    meta,
                    "Cannot use more than one layout qualifier per declaration",
                    self.errors
                ),
                TypeQualifier::Precision(ref p) => qualifier_arm!(
                    p,
                    precision,
                    meta,
                    "Cannot use more than one precision qualifier per declaration",
                    self.errors
                ),
                TypeQualifier::StorageAccess(a) => access &= a,
                _ => {
                    self.errors.push(Error {
                        kind: ErrorKind::SemanticError("Qualifier not supported in globals".into()),
                        meta,
                    });
                }
            }
        }

        match storage {
            StorageQualifier::AddressSpace(AddressSpace::PushConstant) => {
                if set.is_some() {
                    self.errors.push(Error {
                        kind: ErrorKind::SemanticError(
                            "set cannot be used to decorate push constant".into(),
                        ),
                        meta,
                    })
                }
            }
            StorageQualifier::AddressSpace(AddressSpace::Uniform)
            | StorageQualifier::AddressSpace(AddressSpace::Storage { .. }) => {
                if binding.is_none() {
                    self.errors.push(Error {
                        kind: ErrorKind::SemanticError(
                            "uniform/buffer blocks require layout(binding=X)".into(),
                        ),
                        meta,
                    })
                }
            }
            _ => {
                if set.is_some() || binding.is_some() {
                    self.errors.push(Error {
                        kind: ErrorKind::SemanticError(
                            "set/binding can only be applied to uniform/buffer blocks".into(),
                        ),
                        meta,
                    })
                }
            }
        }

        if (sampling.is_some() || interpolation.is_some()) && location.is_none() {
            return Err(Error {
                kind: ErrorKind::SemanticError(
                    "Sampling and interpolation qualifiers can only be used in in/out variables"
                        .into(),
                ),
                meta,
            });
        }

        if let Some(location) = location {
            let input = storage == StorageQualifier::Input;
            let interpolation = interpolation.or_else(|| {
                let kind = self.module.types[ty].inner.scalar_kind()?;
                Some(match kind {
                    ScalarKind::Float => Interpolation::Perspective,
                    _ => Interpolation::Flat,
                })
            });

            let handle = self.module.global_variables.append(
                GlobalVariable {
                    name: name.clone(),
                    space: AddressSpace::Private,
                    binding: None,
                    ty,
                    init,
                },
                meta,
            );

            let idx = self.entry_args.len();
            self.entry_args.push(EntryArg {
                name: name.clone(),
                binding: Binding::Location {
                    location,
                    interpolation,
                    sampling,
                },
                handle,
                storage,
            });

            if let Some(name) = name {
                let lookup = GlobalLookup {
                    kind: GlobalLookupKind::Variable(handle),
                    entry_arg: Some(idx),
                    mutable: !input,
                };
                ctx.add_global(self, &name, lookup, body);

                self.global_variables.push((name, lookup));
            }

            return Ok(GlobalOrConstant::Global(handle));
        } else if let StorageQualifier::Const = storage {
            let init = init.ok_or_else(|| Error {
                kind: ErrorKind::SemanticError("const values must have an initializer".into()),
                meta,
            })?;
            if let Some(name) = name {
                let lookup = GlobalLookup {
                    kind: GlobalLookupKind::Constant(init, ty),
                    entry_arg: None,
                    mutable: false,
                };
                ctx.add_global(self, &name, lookup, body);

                self.global_variables.push((name, lookup));
            }
            return Ok(GlobalOrConstant::Constant(init));
        }

        let space = match self.module.types[ty].inner {
            TypeInner::Image { .. } => AddressSpace::Handle,
            TypeInner::Sampler { .. } => AddressSpace::Handle,
            _ => {
                if let StorageQualifier::AddressSpace(AddressSpace::Storage { .. }) = storage {
                    AddressSpace::Storage { access }
                } else {
                    match storage {
                        StorageQualifier::AddressSpace(space) => space,
                        _ => AddressSpace::Private,
                    }
                }
            }
        };

        let handle = self.module.global_variables.append(
            GlobalVariable {
                name: name.clone(),
                space,
                binding: binding.map(|binding| ResourceBinding {
                    group: set.unwrap_or(0),
                    binding,
                }),
                ty,
                init,
            },
            meta,
        );

        if let Some(name) = name {
            let lookup = GlobalLookup {
                kind: GlobalLookupKind::Variable(handle),
                entry_arg: None,
                mutable: true,
            };
            ctx.add_global(self, &name, lookup, body);

            self.global_variables.push((name, lookup));
        }

        Ok(GlobalOrConstant::Global(handle))
    }

    pub(crate) fn add_local_var(
        &mut self,
        ctx: &mut Context,
        body: &mut Block,
        decl: VarDeclaration,
    ) -> Result<Handle<Expression>> {
        #[cfg(feature = "glsl-validate")]
        if let Some(ref name) = decl.name {
            if ctx.lookup_local_var_current_scope(name).is_some() {
                self.errors.push(Error {
                    kind: ErrorKind::VariableAlreadyDeclared(name.clone()),
                    meta: decl.meta,
                })
            }
        }

        let mut mutable = true;
        let mut precision = None;

        for &(ref qualifier, meta) in decl.qualifiers {
            match *qualifier {
                TypeQualifier::StorageQualifier(StorageQualifier::Const) => {
                    if !mutable {
                        self.errors.push(Error {
                            kind: ErrorKind::SemanticError(
                                "Cannot use more than one constant qualifier per declaration"
                                    .into(),
                            ),
                            meta,
                        })
                    }

                    mutable = false;
                }
                TypeQualifier::Precision(ref p) => qualifier_arm!(
                    p,
                    precision,
                    meta,
                    "Cannot use more than one precision qualifier per declaration",
                    self.errors
                ),
                _ => self.errors.push(Error {
                    kind: ErrorKind::SemanticError("Qualifier not supported in locals".into()),
                    meta,
                }),
            }
        }

        let handle = ctx.locals.append(
            LocalVariable {
                name: decl.name.clone(),
                ty: decl.ty,
                init: decl.init,
            },
            decl.meta,
        );
        let expr = ctx.add_expression(Expression::LocalVariable(handle), decl.meta, body);

        if let Some(name) = decl.name {
            ctx.add_local_var(name, expr, mutable);
        }

        Ok(expr)
    }
}
