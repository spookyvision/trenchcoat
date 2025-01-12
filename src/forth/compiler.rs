use std::{collections::HashMap, marker::PhantomData};

use anyhow::{anyhow, Context};
use log::{error, trace};
#[cfg(feature = "tty")]
use swc_common::errors::ColorConfig;
use swc_common::{errors::Handler, sync::Lrc, SourceMap};
use swc_ecma_ast::*;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecma_utils::ExprExt;
use swc_ecma_visit::Visit;

use super::{
    util::MockRuntime,
    vm::{types::VMVec, Cell, CellData, DefaultStack, FFIOps, FuncDef, Op, VM},
};
use crate::{forth::util::pack, pixelblaze, vanillajs};

#[cfg_attr(feature = "tty", derive(clap::ValueEnum))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Flavor {
    VanillaJS,
    Pixelblaze,
    Pythonic,
}

#[derive(Debug)]
pub enum Source<'a> {
    File(Box<std::path::Path>),
    String(&'a str),
}

#[cfg(not(feature = "tty"))]
#[derive(Clone, Copy)]
struct LogEmitter;

impl std::io::Write for LogEmitter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Ok(s) = core::str::from_utf8(buf) {
            log::warn!("{s}");
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub fn compile(source: Source, flavor: Flavor) -> anyhow::Result<Vec<u8>> {
    let source_map: Lrc<SourceMap> = Default::default();
    let source_file = match &source {
        Source::File(path) => source_map
            .load_file(&path)
            .with_context(|| format!("Failed to load {source:?}"))?,
        Source::String(source) => source_map.new_source_file(
            swc_common::FileName::Custom("__trenchcc_generated.js".into()).into(),
            source.to_string(),
        ),
    };

    let lexer = Lexer::new(
        // We want to parse ecmascript
        Syntax::Es(Default::default()),
        // EsVersion defaults to es5
        Default::default(),
        StringInput::from(&*source_file),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    let handler = new_handler(source_map.clone());
    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    if let Ok(module) = parser.parse_module().map_err(|e| {
        e.clone().into_diagnostic(&handler).emit();
    }) {
        let ser = match flavor {
            // TODO: use JS "console.log" for py "print" for now, need to come up with a smarter design
            Flavor::Pythonic => emit(module, pixelblaze::ffi::FFI_FUNCS, MockRuntime::default()),
            Flavor::VanillaJS => emit(module, vanillajs::ffi::FFI_FUNCS, MockRuntime::default()),
            Flavor::Pixelblaze => emit(module, pixelblaze::ffi::FFI_FUNCS, MockRuntime::default()),
        }
        .map_err(|e| anyhow!("Compilation failed: {e:?}"))?;

        return Ok(ser);
    }

    anyhow::bail!("Compilation failed")
}

fn new_handler(source_map: Lrc<SourceMap>) -> Handler {
    #[cfg(feature = "tty")]
    {
        Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(source_map))
    }
    #[cfg(not(feature = "tty"))]
    {
        let emitter = swc_common::errors::EmitterWriter::new(
            Box::new(LogEmitter),
            Some(source_map.clone()),
            false,
            false,
        );
        Handler::with_emitter(true, false, Box::new(emitter))
    }
}

fn emit<FFI, RT>(
    module: Module,
    ffi_defs: phf::Map<&str, FFI>,
    runtime: RT,
) -> Result<Vec<u8>, postcard::Error>
where
    FFI: FFIOps<RT> + Copy + Eq + serde::Serialize,
    RT: Clone + PartialEq,
{
    let mut v = Compiler::new(
        ffi_defs
            .into_iter()
            .map(|(k, v)| (k.to_string(), *v))
            .collect::<HashMap<_, _>>(),
    );
    v.visit_module(&module);

    let vm = v.into_vm(runtime);
    println!("vm size is {}", std::mem::size_of_val(&vm));
    postcard::to_allocvec_cobs(&vm)
}

pub struct Compiler<FFI, RT> {
    stack: DefaultStack<FFI>,
    func_defs: HashMap<String, FuncDef<FFI>>,
    ffi_defs: HashMap<String, FFI>,
    inside_assignment: bool,
    _rt: PhantomData<RT>,
}

impl<FFI, RT> Compiler<FFI, RT>
where
    FFI: FFIOps<RT> + Clone + Eq,
{
    pub fn new(ffi_defs: HashMap<String, FFI>) -> Self {
        Self {
            stack: VMVec::new(),
            func_defs: HashMap::new(),
            ffi_defs,
            inside_assignment: false,
            _rt: PhantomData,
        }
    }
    fn eval_expr(&mut self, ex: &Expr) {
        match ex {
            Expr::This(_) => error!("implement me"),
            Expr::Array(_) => error!("implement me"),
            Expr::Object(_) => error!("implement me"),
            Expr::Fn(_) => error!("implement me"),
            Expr::Unary(unary_expr) => {
                match unary_expr.op {
                    UnaryOp::Minus => {
                        self.eval_expr(&unary_expr.arg);
                        self.stack.push((-1).into());
                        self.stack.push(Cell::Op(Op::Mul));
                    }
                    UnaryOp::Plus => {
                        // no-op ... right? RIGHT?
                    }
                    UnaryOp::Bang => error!("implement me"),
                    UnaryOp::Tilde => error!("implement me"),
                    UnaryOp::TypeOf => error!("implement me"),
                    UnaryOp::Void => error!("implement me"),
                    UnaryOp::Delete => error!("implement me"),
                }
            }
            Expr::Update(_) => error!("implement me"),
            Expr::Bin(bin_expr) => {
                self.eval_expr(&bin_expr.left);
                self.eval_expr(&bin_expr.right);
                let _ = match bin_expr.op {
                    BinaryOp::EqEq => self.stack.push(Cell::Op(Op::EqEq)),
                    BinaryOp::NotEq => self.stack.push(Cell::Op(Op::NotEq)),
                    BinaryOp::EqEqEq => error!("implement me"),
                    BinaryOp::NotEqEq => error!("implement me"),
                    BinaryOp::Lt => self.stack.push(Cell::Op(Op::Lt)),
                    BinaryOp::LtEq => self.stack.push(Cell::Op(Op::LtEq)),
                    BinaryOp::Gt => self.stack.push(Cell::Op(Op::Gt)),
                    BinaryOp::GtEq => self.stack.push(Cell::Op(Op::GtEq)),
                    BinaryOp::LShift => error!("implement me"),
                    BinaryOp::RShift => error!("implement me"),
                    BinaryOp::ZeroFillRShift => error!("implement me"),
                    BinaryOp::Add => self.stack.push(Cell::Op(Op::Add)),
                    BinaryOp::Sub => self.stack.push(Cell::Op(Op::Sub)),
                    BinaryOp::Mul => self.stack.push(Cell::Op(Op::Mul)),
                    BinaryOp::Div => self.stack.push(Cell::Op(Op::Div)),
                    BinaryOp::Mod => self.stack.push(Cell::Op(Op::Mod)),
                    BinaryOp::BitOr => self.stack.push(Cell::Op(Op::Or)),
                    BinaryOp::BitXor => error!("implement me"),
                    BinaryOp::BitAnd => error!("implement me"),
                    BinaryOp::LogicalOr => error!("implement me"),
                    BinaryOp::LogicalAnd => error!("implement me"),
                    BinaryOp::In => error!("implement me"),
                    BinaryOp::InstanceOf => error!("implement me"),
                    BinaryOp::Exp => error!("implement me"),
                    BinaryOp::NullishCoalescing => error!("implement me"),
                };
            }
            Expr::Assign(ass) => {
                let left = &ass.left;
                let err = format!("wat is this {left:?}",);
                let name: &BindingIdent = left.as_ident().expect(&err);
                let name = var_name(PatWrap::Ident(name));
                let right = &ass.right;
                trace!("assign {name} = {:?}", right);

                self.inside_assignment = true;
                self.eval_expr(right);
                self.inside_assignment = false;
                self.stack.push(Cell::Op(Op::SetVar(name.into())));
            }
            Expr::Member(_) => error!("implement me"),
            Expr::SuperProp(_) => error!("implement me"),
            Expr::Cond(_) => error!("implement me"),
            Expr::Call(call_expr) => {
                if !self.inside_assignment {
                    self.stack.push(Op::PopRet.into());
                } else {
                    self.stack.push(Op::Nruter.into());
                }
                for arg in &call_expr.args {
                    self.eval_expr(&arg.expr);
                }

                let callee = &call_expr.callee;
                trace!("{callee:?}");
                match callee {
                    Callee::Super(_) => error!("implement me"),
                    Callee::Import(_) => error!("implement me"),
                    Callee::Expr(call_expr) => match call_expr.as_ref() {
                        Expr::Member(me) => {
                            // TODO this always calls FFI funcs, e.g. console.log turns into ffi namespace console_log
                            // SOME DAY we might want object support lol
                            if let (Some(obj), Some(prop)) = (me.obj.as_ident(), me.prop.as_ident())
                            {
                                let obj = obj.sym.as_ref();
                                let prop = prop.sym.as_ref();
                                let func = format!("{obj}_{prop}");
                                self.stack.push(Cell::Op(Op::FFI(
                                    self.ffi_defs
                                        .get(&func)
                                        .expect("function not found!")
                                        .clone(),
                                )));
                            }
                        }
                        Expr::Ident(func_name) => {
                            let func_name = func_name.sym.as_ref();

                            // TODO FFI funcs take precedence over local definitions, which is not optimal
                            // (but also not terrible .. can reverse precedence if needed)

                            match self.ffi_defs.get(func_name) {
                                Some(ffi_func) => {
                                    trace!("add ffi call to {func_name:?}");
                                    self.stack.push(Cell::Op(Op::FFI(ffi_func.clone())));
                                }
                                None => {
                                    trace!("add call to {func_name:?}");
                                    self.stack.push(Cell::Op(Op::Call(func_name.into())));
                                }
                            };
                        }
                        Expr::This(_) => error!("implement me"),
                        Expr::Object(_) => error!("implement me"),
                        Expr::Fn(_f) => error!("implement me"),
                        _ => error!("implement me"),
                    },
                }
                // println!("call! {name:?}({args:?})");
            }
            Expr::New(_) => error!("implement me"),
            Expr::Seq(s) => {
                trace!("SEQ {s:?}");
            }
            Expr::Ident(id) => {
                trace!("ident! {id:?}");
                self.stack
                    .push(Cell::Op(Op::GetVar(id.sym.as_ref().into())));
            }
            Expr::Lit(lit) => {
                trace!("lit! {lit:?}");
                match lit {
                    Lit::Str(s) => {
                        let s = &s.value;
                        let bytes = s.as_bytes();
                        let packed = pack(&bytes);
                        self.stack.extend(packed);
                    }
                    Lit::Bool(b) => self
                        .stack
                        .push(Cell::Val(CellData::from_num(b.value as i32))),
                    Lit::Null(_) => error!("implement me"),
                    Lit::Num(num) => self.stack.push(Cell::Val(CellData::from_num(num.value))),
                    Lit::BigInt(_) => error!("implement me"),
                    Lit::Regex(_) => error!("implement me"),
                    Lit::JSXText(_) => error!("implement me"),
                };
            }
            _ => error!("implement me"),
        }
    }

    pub fn into_vm(self, rt: RT) -> VM<FFI, RT> {
        // TODO this is nonsense, maybe removing `vm` from the visitor wasn't such a smart idea after all
        // but what about the runtime param then...
        let mut vm = VM::new(self.stack, Default::default(), rt);
        for (name, func_def) in self.func_defs {
            vm.add_func(name, func_def.params(), func_def.stack());
        }
        vm
    }
}

impl<FFI, RT> Visit for Compiler<FFI, RT>
where
    RT: Clone + PartialEq,
    FFI: FFIOps<RT> + Clone + Eq,
{
    fn visit_fn_decl(&mut self, n: &FnDecl) {
        let name = n.ident.sym.as_ref();
        let mut child_visor: Compiler<_, RT> = Compiler::new(self.ffi_defs.clone());
        let func = &n.function;

        // add implicit return
        child_visor.stack.push(Cell::Null);
        child_visor.stack.push(Op::Return.into());

        if let Some(body) = &func.body {
            for s in body.stmts.iter().rev() {
                child_visor.visit_stmt(s);
            }
        }

        let params: Vec<String> = func
            .params
            .iter()
            .map(|p| var_name(PatWrap::Pat(&p.pat)))
            .collect();
        self.func_defs
            .insert(name.to_string(), FuncDef::new(&params, child_visor.stack));
    }
    // fn visit_ident(&mut self, n: &Ident) {
    //     let sym_str = n.sym.as_ref();
    //     println!("ID {sym_str}");
    // }
    fn visit_if_stmt(&mut self, n: &IfStmt) {
        self.visit_expr(&n.test);
        self.stack.push(Op::If.into());
        self.visit_stmt(&n.cons);
        if let Some(alt) = &n.alt {
            self.stack.push(Op::Else.into());
            self.visit_stmt(alt);
        }
        self.stack.push(Op::Then.into());
    }
    // fn visit_assign_expr(&mut self, n: &AssignExpr) {
    //     println!("ass ex {:?}", n);
    // }
    // fn visit_assign_op(&mut self, n: &AssignOp) {
    //     println!("ass op {:?}", n);
    // }

    // fn visit_assign_pat(&mut self, n: &AssignPat) {
    //     println!("ass pat {:?}", n);
    // }
    // fn visit_assign_pat_prop(&mut self, n: &AssignPatProp) {
    //     println!("ass pat prop {:?}", n);
    // }
    // fn visit_assign_prop(&mut self, n: &AssignProp) {
    //     println!("ass prop {:?}", n);
    // }

    fn visit_expr(&mut self, ex: &Expr) {
        self.eval_expr(ex);
    }

    fn visit_return_stmt(&mut self, n: &ReturnStmt) {
        if let Some(arg) = &n.arg {
            self.eval_expr(arg.as_expr());
            self.stack.push(Cell::Op(Op::Return));
        }
    }

    fn visit_var_decl(&mut self, n: &VarDecl) {
        // TODO make this work for > 1 decl
        for decl in n.decls.iter() {
            let name = var_name(PatWrap::Pat(&decl.name));

            trace!("<decl {name} = ");

            if let Some(init) = decl.init.as_deref() {
                self.inside_assignment = true;
                self.eval_expr(init);
                self.inside_assignment = false;
                self.stack.push(Op::SetVar(name.clone()).into());
            }
            self.stack.push(Op::DeclVar(name.clone()).into());

            trace!("</decl {name}>");
        }
    }
}

enum PatWrap<'a> {
    Pat(&'a Pat),
    Ident(&'a BindingIdent),
}

impl<'a> PatWrap<'a> {
    fn as_ident(&self) -> Option<&BindingIdent> {
        match self {
            PatWrap::Pat(p) => p.as_ident(),
            PatWrap::Ident(id) => Some(id),
        }
    }
}

fn var_name(pat: PatWrap) -> String {
    let res = pat
        .as_ident()
        .map(|id| id.sym.as_ref())
        .expect("can't make sense of this variable name");

    res.to_owned()
}

#[test]
fn test_if() -> anyhow::Result<()> {
    let source = r#"
    if (1 > 0) {
        x = 1
    } else {
        x = 2
    }
    "#;

    let _ = compile(Source::String(source), Flavor::VanillaJS)?;
    Ok(())
}
