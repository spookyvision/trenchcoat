use swc_common::{
    errors::{ColorConfig, Handler},
    sync::Lrc,
    FileName, SourceMap,
};
use swc_ecma_ast::Ident;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecma_visit::Visit;
use vis0r::Vis0r;
mod vis0r {
    use std::collections::HashMap;

    use phf::phf_map;
    use pixelblaze_rs::forth::bytecode::{Cell, CellData, Op, FFI, VM};
    use swc_ecma_ast::*;
    use swc_ecma_utils::ExprExt;
    use swc_ecma_visit::Visit;

    static FUNCS: phf::Map<&'static str, FFI> = phf_map! {
        "console_log" => FFI::ConsoleLog1
    };

    pub struct Vis0r {
        vm: VM,
    }

    impl Vis0r {
        pub fn new() -> Self {
            Self { vm: VM::new() }
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
                    match bin_expr.op {
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
                        BinaryOp::Add => self.vm.push(Cell::Op(Op::Add)),
                        BinaryOp::Sub => self.vm.push(Cell::Op(Op::Sub)),
                        BinaryOp::Mul => self.vm.push(Cell::Op(Op::Mul)),
                        BinaryOp::Div => self.vm.push(Cell::Op(Op::Div)),
                        BinaryOp::Mod => self.vm.push(Cell::Op(Op::Mod)),
                        BinaryOp::BitOr => self.vm.push(Cell::Op(Op::Or)),
                        BinaryOp::BitXor => todo!(),
                        BinaryOp::BitAnd => todo!(),
                        BinaryOp::LogicalOr => todo!(),
                        BinaryOp::LogicalAnd => todo!(),
                        BinaryOp::In => todo!(),
                        BinaryOp::InstanceOf => todo!(),
                        BinaryOp::Exp => todo!(),
                        BinaryOp::NullishCoalescing => todo!(),
                    }
                }
                Expr::Assign(ass) => {
                    let left = &ass.left;

                    let name = left.as_pat().expect("wat is this {left:?}");
                    let name = var_name(name);
                    let right = &ass.right;
                    println!("assign {name} = {:?}", right);

                    self.eval_expr(right);
                    self.vm.push(Cell::Op(Op::SetVar(name.into())));
                }
                Expr::Member(_) => todo!(),
                Expr::SuperProp(_) => todo!(),
                Expr::Cond(_) => todo!(),
                Expr::Call(call_expr) => {
                    // let args = vec![];
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
                                // TODO this always calls FFI funcs, SOME DAY we might want object support lol
                                if let (Some(obj), Some(prop)) =
                                    (me.obj.as_ident(), me.prop.as_ident())
                                {
                                    let obj = obj.sym.as_ref();
                                    let prop = prop.sym.as_ref();
                                    let func = format!("{obj}_{prop}");
                                    self.vm.push(Cell::Op(Op::FFI(
                                        *FUNCS.get(&func).expect("function not found!"),
                                    )));
                                }
                            }
                            Expr::Ident(func_name) => {
                                // TODO this never calls FFI funcs

                                self.vm.push(Cell::Op(Op::Nruter));
                                // TODO eval args

                                self.vm
                                    .push(Cell::Op(Op::Call(func_name.sym.as_ref().into())));
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
                    self.vm.push(Cell::Op(Op::GetVar(id.sym.as_ref().into())));
                }
                Expr::Lit(lit) => {
                    println!("lit! {:?}", lit);
                    match lit {
                        Lit::Str(s) => self.vm.push_str(&s.value),
                        Lit::Bool(b) => self.vm.push(Cell::Val(CellData::from_num(b.value as i32))),
                        Lit::Null(_) => todo!(),
                        Lit::Num(num) => self.vm.push(Cell::Val(CellData::from_num(num.value))),
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

        pub fn vm_mut(&mut self) -> &mut VM {
            &mut self.vm
        }
    }

    impl Visit for Vis0r {
        fn visit_fn_decl(&mut self, n: &FnDecl) {
            let name = n.ident.sym.as_ref();
            let mut child_visor = Vis0r::new();
            if let Some(body) = &n.function.body {
                for s in body.stmts.iter().rev() {
                    child_visor.visit_stmt(s);
                }
            }
            self.vm.add_func(name, child_visor.vm.stack());
        }
        fn visit_ident(&mut self, n: &Ident) {
            let sym_str = n.sym.as_ref();
            println!("ID {sym_str}");
        }
        fn visit_if_stmt(&mut self, n: &IfStmt) {
            println!("if {:?}", n.test);
        }
        fn visit_assign_expr(&mut self, n: &AssignExpr) {
            println!("ass ex {:?}", n);
        }
        fn visit_assign_op(&mut self, n: &AssignOp) {
            println!("ass op {:?}", n);
        }

        fn visit_assign_pat(&mut self, n: &AssignPat) {
            println!("ass pat {:?}", n);
        }
        fn visit_assign_pat_prop(&mut self, n: &AssignPatProp) {
            println!("ass pat prop {:?}", n);
        }
        fn visit_assign_prop(&mut self, n: &AssignProp) {
            println!("ass prop {:?}", n);
        }

        fn visit_expr(&mut self, ex: &Expr) {
            self.eval_expr(ex);
        }

        fn visit_return_stmt(&mut self, n: &ReturnStmt) {
            if let Some(arg) = &n.arg {
                self.eval_expr(arg.as_expr());
                self.vm.push(Cell::Op(Op::Return));
            }
        }

        fn visit_call_expr(&mut self, n: &CallExpr) {
            panic!("test");
            self.vm.push(Cell::Op(Op::Nruter));
            // TODO eval args

            let callee = &n
                .callee
                .as_expr()
                .expect("cannot call this")
                .as_ident()
                .expect("cannot identify this")
                .sym;
            self.vm.push(Cell::Op(Op::Call(callee.as_ref().into())));
        }
        fn visit_var_decl(&mut self, n: &VarDecl) {
            // TODO make this work for > 1 decl
            for decl in n.decls.iter() {
                let name = var_name(&decl.name);

                println!("\ndecl {name} = ");

                self.eval_expr(decl.init.as_deref().unwrap());
                self.vm.push(Cell::Op(Op::SetVar(name.into())));
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
}

fn main() -> anyhow::Result<()> {
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    // Real usage
    // let fm = cm
    //     .load_file(Path::new("test.js"))
    //     .expect("failed to load test.js");

    let js = "
    // An XOR in 2D/3D space based on block reflections

export function beforeRender(delta) {
  t2 = time(0.1) * PI2
  t1 = time(.1)
  t3 = time(.5)
  t4 = time(0.2) * PI2
}

export function render2D(index, x, y) {
  render3D(index, x, y, 0)
}

export function render3D(index, x, y, z) {
  h = sin(t2)
  m = (.3 + triangle(t1) * .2)
  h = h + (wave((5*(x-.5) ^ 5*(y-.5) ^ 5*(z-.5))/50  * ( triangle(t3) * 10 + 4 * sin(t4)) % m))
  s = 1;
  v = ((abs(h) + abs(m) + t1) % 1);
  v = triangle(v*v)
  h = triangle(h)/5 + (x + y + z)/3 + t1
  
  // test 
  if (1 > 2*h) {
    v = v * v * v
  }
  // test end

  v = v * v * v
  hsv(h, s, v)
}
    ";

    let js = "
    export function something() {
        var x = 10.1;
        return x;
    }

    export function main() {
        //console.log(\"js!!11!!twelve∆h\");
        var x = 10 * 5; // 50
        var y = 4.4 + x; // 54.4
        x = y - something(); // 54.4 - 10.1 = 44.3
    }";

    // SOON
    // let js = include_str!("../../res/rainbow melt.js");
    let fm = cm.new_source_file(FileName::Custom("test.js".into()), js.into());
    let lexer = Lexer::new(
        // We want to parse ecmascript
        Syntax::Es(Default::default()),
        // EsVersion defaults to es5
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    if let Ok(module) = parser.parse_module().map_err(|e| {
        // Unrecoverable fatal error occurred
        e.into_diagnostic(&handler).emit()
    }) {
        let mut v = Vis0r::new();
        dbg!(&module);
        v.visit_module(&module);

        println!("\n\n\n*** VM START ***\n");
        let vm = v.vm_mut();
        vm.dump_state();
        vm.call_fn("main");
        println!("\n*** DÖNE ***\n");
        vm.dump_state();
    }

    // if let Ok(module) = parser.parse_module().map_err(|mut e| {
    //     // Unrecoverable fatal error occurred
    //     e.into_diagnostic(&handler).emit()
    // }) {
    //     for item in module.body.iter() {
    //         match item {
    //             ModuleItem::ModuleDecl(decl) => {
    //                 println!("DECL {decl:?} \n\n");
    //                 if let ModuleDecl::ExportDecl(ExportDecl { span, decl }) = decl {
    //                     match decl {
    //                         Decl::Class(class) => println!("{class:?}"),
    //                         Decl::Fn(function) => println!("{function:?}"),
    //                         Decl::Var(var) => println!("{var:?}"),
    //                         Decl::TsInterface(_) => todo!(),
    //                         Decl::TsTypeAlias(_) => todo!(),
    //                         Decl::TsEnum(_) => todo!(),
    //                         Decl::TsModule(_) => todo!(),
    //                     }
    //                 }
    //                 // match decl {
    //                 //     ModuleDecl::Import(_) => todo!(),
    //                 //     ModuleDecl::ExportDecl(_) => todo!(),
    //                 //     ModuleDecl::ExportNamed(_) => todo!(),
    //                 //     ModuleDecl::ExportDefaultDecl(_) => todo!(),
    //                 //     ModuleDecl::ExportDefaultExpr(_) => todo!(),
    //                 //     ModuleDecl::ExportAll(_) => todo!(),
    //                 //     ModuleDecl::TsImportEquals(_) => todo!(),
    //                 //     ModuleDecl::TsExportAssignment(_) => todo!(),
    //                 //     ModuleDecl::TsNamespaceExport(_) => todo!(),
    //                 // }
    //             }
    //             ModuleItem::Stmt(st) => println!("ST {st:?}"),
    //         }
    //         println!("\n\n");
    //     }
    // }
    Ok(())
}

pub fn whats_this(js: &str) {}
