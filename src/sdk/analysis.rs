use std::fs;
use std::path::Path;
use syn::{ItemFn, Meta, visit::Visit};

pub struct OpVisitor {
    pub ops: Vec<OpInfo>,
}

#[derive(Debug, Clone)]
pub struct OpInfo {
    pub name: String,
    pub docs: Vec<String>,
    pub args: Vec<ArgInfo>,
}

#[derive(Debug, Clone)]
pub struct ArgInfo {
    pub name: String,
    pub type_name: String,
}

impl<'ast> Visit<'ast> for OpVisitor {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        // Check if function has #[op2] attribute
        let is_op = node.attrs.iter().any(|attr| {
            if let Meta::List(meta) = &attr.meta {
                if let Some(ident) = meta.path.get_ident() {
                    return ident == "op2";
                }
            }
            false
        });

        if is_op {
            let name = node.sig.ident.to_string();
            let mut docs = Vec::new();
            
            // Extract doc comments
            for attr in &node.attrs {
                if let Meta::NameValue(meta) = &attr.meta {
                    if meta.path.is_ident("doc") {
                        if let syn::Expr::Lit(expr_lit) = &meta.value {
                            if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                                docs.push(lit_str.value().trim().to_string());
                            }
                        }
                    }
                }
            }

            let mut args = Vec::new();
            for input in &node.sig.inputs {
                if let syn::FnArg::Typed(pat_type) = input {
                    if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                        let arg_name = pat_ident.ident.to_string();
                        // Simple type extraction (this is a simplification)
                        let type_name = quote::quote!(#pat_type.ty).to_string();
                        args.push(ArgInfo {
                            name: arg_name,
                            type_name,
                        });
                    }
                }
            }

            self.ops.push(OpInfo {
                name,
                docs,
                args,
            });
        }
        
        // Continue visiting children
        syn::visit::visit_item_fn(self, node);
    }
}

pub fn analyze_source(path: &Path) -> Vec<OpInfo> {
    let content = fs::read_to_string(path).unwrap_or_default();
    let syntax = syn::parse_file(&content).expect("Unable to parse file");
    
    let mut visitor = OpVisitor { ops: Vec::new() };
    visitor.visit_file(&syntax);
    
    visitor.ops
}
