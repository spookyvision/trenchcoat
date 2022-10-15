use swc_common::sync::Lrc;
use swc_common::{
    errors::{ColorConfig, Handler},
    FileName, SourceMap,
};
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};
use swc_ecma_visit::Visit;
use vis0r::Vis0r;
mod vis0r {
    use swc_ecma_visit::Visit;

    use swc_ecma_ast::*;

    pub struct Vis0r {}

    impl Vis0r {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Visit for Vis0r {
        fn visit_ident(&mut self, n: &Ident) {
            let sym_str = n.sym.as_ref();
            println!("ID {sym_str}");
        }
        fn visit_if_stmt(&mut self, n: &IfStmt) {
            println!("if {:?}", n.test);
        }
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

    if let Ok(module) = parser.parse_module().map_err(|mut e| {
        // Unrecoverable fatal error occurred
        e.into_diagnostic(&handler).emit()
    }) {
        let mut v = Vis0r::new();
        v.visit_module(&module);
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
