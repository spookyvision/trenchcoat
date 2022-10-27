use std::{collections::HashMap, marker::PhantomData};

use swc_ecma_ast::*;
use swc_ecma_utils::ExprExt;
use swc_ecma_visit::Visit;

use super::{
    bytecode::{DefaultStack, FFIOps},
    runtime::CoreRuntime,
};
use crate::forth::bytecode::{Cell, CellData, FuncDef, Op, Stack, VM};

pub struct Vis0r<FFI, RT> {
    stack: DefaultStack<FFI>,
    func_defs: HashMap<String, FuncDef<FFI>>,
    ffi_defs: HashMap<String, FFI>,
    inside_assignment: bool,
    _rt: PhantomData<RT>,
}

impl<FFI, RT> Vis0r<FFI, RT>
where
    RT: CoreRuntime,
    FFI: FFIOps<RT> + Copy,
{
    pub fn new(ffi_defs: HashMap<String, FFI>) -> Self {
        Self {
            stack: heapless::Vec::new(),
            func_defs: HashMap::new(),
            ffi_defs,
            inside_assignment: false,
            _rt: PhantomData,
        }
    }
    fn eval_expr(&mut self, ex: &Expr) {
        match ex {
            Expr::This(_) => todo!(),
            Expr::Array(_) => todo!(),
            Expr::Object(_) => todo!(),
            Expr::Fn(_) => todo!(),
            Expr::Unary(_) => todo!(),
            Expr::Update(_) => todo!(),
            Expr::Bin(bin_expr) => {
                self.eval_expr(&bin_expr.left);
                self.eval_expr(&bin_expr.right);
                let _ = match bin_expr.op {
                    BinaryOp::EqEq => todo!(),
                    BinaryOp::NotEq => todo!(),
                    BinaryOp::EqEqEq => todo!(),
                    BinaryOp::NotEqEq => todo!(),
                    BinaryOp::Lt => todo!(),
                    BinaryOp::LtEq => todo!(),
                    BinaryOp::Gt => todo!(),
                    BinaryOp::GtEq => todo!(),
                    BinaryOp::LShift => todo!(),
                    BinaryOp::RShift => todo!(),
                    BinaryOp::ZeroFillRShift => todo!(),
                    BinaryOp::Add => self.stack.push(Cell::Op(Op::Add)),
                    BinaryOp::Sub => self.stack.push(Cell::Op(Op::Sub)),
                    BinaryOp::Mul => self.stack.push(Cell::Op(Op::Mul)),
                    BinaryOp::Div => self.stack.push(Cell::Op(Op::Div)),
                    BinaryOp::Mod => self.stack.push(Cell::Op(Op::Mod)),
                    BinaryOp::BitOr => self.stack.push(Cell::Op(Op::Or)),
                    BinaryOp::BitXor => todo!(),
                    BinaryOp::BitAnd => todo!(),
                    BinaryOp::LogicalOr => todo!(),
                    BinaryOp::LogicalAnd => todo!(),
                    BinaryOp::In => todo!(),
                    BinaryOp::InstanceOf => todo!(),
                    BinaryOp::Exp => todo!(),
                    BinaryOp::NullishCoalescing => todo!(),
                };
            }
            Expr::Assign(ass) => {
                let left = &ass.left;

                let name = left.as_pat().expect("wat is this {left:?}");
                let name = var_name(name);
                let right = &ass.right;
                println!("assign {name} = {:?}", right);

                self.inside_assignment = true;
                self.eval_expr(right);
                self.inside_assignment = false;
                self.stack.push(Cell::Op(Op::SetVar(name.into())));
            }
            Expr::Member(_) => todo!(),
            Expr::SuperProp(_) => todo!(),
            Expr::Cond(_) => todo!(),
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
                dbg!(&callee);
                match callee {
                    Callee::Super(_) => todo!(),
                    Callee::Import(_) => todo!(),
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
                                    *self.ffi_defs.get(&func).expect("function not found!"),
                                )));
                            }
                        }
                        Expr::Ident(func_name) => {
                            let func_name = func_name.sym.as_ref();

                            // TODO FFI funcs take precedence over local definitions, which is not optimal
                            // (but also not terrible .. can reverse precedence if needed)

                            match self.ffi_defs.get(func_name) {
                                Some(ffi_func) => {
                                    dbg!("add ffi call to", func_name);
                                    self.stack.push(Cell::Op(Op::FFI(*ffi_func)));
                                }
                                None => {
                                    dbg!("add call to", func_name);
                                    self.stack.push(Cell::Op(Op::Call(func_name.into())));
                                }
                            };
                        }
                        Expr::This(_) => todo!(),
                        Expr::Object(_) => todo!(),
                        Expr::Fn(f) => todo!(),
                        _ => todo!(),
                    },
                }
                // println!("call! {name:?}({args:?})");
            }
            Expr::New(_) => todo!(),
            Expr::Seq(s) => {
                dbg!("SEQ", s);
            }
            Expr::Ident(id) => {
                println!("ident! {:?}", id);
                self.stack
                    .push(Cell::Op(Op::GetVar(id.sym.as_ref().into())));
            }
            Expr::Lit(lit) => {
                println!("lit! {:?}", lit);
                match lit {
                    Lit::Str(s) => {
                        // TODO dedup <> `VM::push_str`
                        let s = &s.value;
                        let bytes = s.as_bytes();
                        let valid_bytes_len = bytes.len();

                        // TODO maybe we need chunks_exact
                        // let chonky_boytes = bytes.chunks_exact(4);
                        let chonky_boytes = bytes.chunks(4);

                        // let remainder = chonky_boytes.remainder();
                        let chonky_boytes = chonky_boytes.map(|boi| {
                            let val = CellData::from_le_bytes(<[u8; 4]>::try_from(boi).unwrap());
                            Cell::Val(val)
                        });
                        self.stack.extend(chonky_boytes);
                        self.stack
                            .push(Cell::Val(CellData::from_num(valid_bytes_len)))
                    }
                    Lit::Bool(b) => self
                        .stack
                        .push(Cell::Val(CellData::from_num(b.value as i32))),
                    Lit::Null(_) => todo!(),
                    Lit::Num(num) => self.stack.push(Cell::Val(CellData::from_num(num.value))),
                    Lit::BigInt(_) => todo!(),
                    Lit::Regex(_) => todo!(),
                    Lit::JSXText(_) => todo!(),
                };
            }
            Expr::Tpl(_) => todo!(),
            Expr::TaggedTpl(_) => todo!(),
            Expr::Arrow(_) => todo!(),
            Expr::Class(_) => todo!(),
            Expr::Yield(_) => todo!(),
            Expr::MetaProp(_) => todo!(),
            Expr::Await(_) => todo!(),
            Expr::Paren(_) => todo!(),
            Expr::JSXMember(_) => todo!(),
            Expr::JSXNamespacedName(_) => todo!(),
            Expr::JSXEmpty(_) => todo!(),
            Expr::JSXElement(_) => todo!(),
            Expr::JSXFragment(_) => todo!(),
            Expr::TsTypeAssertion(_) => todo!(),
            Expr::TsConstAssertion(_) => todo!(),
            Expr::TsNonNull(_) => todo!(),
            Expr::TsAs(_) => todo!(),
            Expr::TsInstantiation(_) => todo!(),
            Expr::TsSatisfaction(_) => todo!(),
            Expr::PrivateName(_) => todo!(),
            Expr::OptChain(_) => todo!(),
            Expr::Invalid(_) => todo!(),
        }
    }

    pub fn into_vm(mut self, rt: RT) -> VM<FFI, RT> {
        // TODO this is nonsense, maybe removing `vm` from the visitor wasn't such a smart idea after all
        // but what about the runtime param then...
        let mut vm = VM::new(self.stack, Default::default(), rt);
        for (name, func_def) in self.func_defs {
            vm.add_func(name, func_def.params(), func_def.stack());
        }
        vm
    }
}

impl<FFI, RT> Visit for Vis0r<FFI, RT>
where
    RT: CoreRuntime,
    FFI: FFIOps<RT> + Copy + Clone,
{
    fn visit_fn_decl(&mut self, n: &FnDecl) {
        let name = n.ident.sym.as_ref();
        let mut child_visor: Vis0r<_, RT> = Vis0r::new(self.ffi_defs.clone());
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
            .map(|p| var_name(&p.pat).to_owned())
            .collect();
        self.func_defs
            .insert(name.to_string(), FuncDef::new(&params, child_visor.stack));
    }
    // fn visit_ident(&mut self, n: &Ident) {
    //     let sym_str = n.sym.as_ref();
    //     println!("ID {sym_str}");
    // }
    // fn visit_if_stmt(&mut self, n: &IfStmt) {
    //     println!("if {:?}", n.test);
    // }
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
            let name = var_name(&decl.name);

            println!("\ndecl {name} = ");

            if let Some(init) = decl.init.as_deref() {
                self.inside_assignment = true;
                self.eval_expr(init);
                self.inside_assignment = false;
                self.stack.push(Op::SetVar(name.into()).into());
            }
            self.stack.push(Op::DeclVar(name.into()).into());

            println!("</decl {name}>");
        }
    }
}

fn var_name(pat: &Pat) -> &str {
    let res = pat
        .as_ident()
        .map(|id| id.sym.as_ref())
        .expect("can't make sense of this variable name");

    res
}
