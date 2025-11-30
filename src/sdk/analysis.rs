use std::fs;
use std::path::Path;
use syn::{ItemFn, Meta, visit::Visit};

pub struct OpVisitor {
    pub ops: Vec<OpInfo>,
    pub structs: Vec<StructInfo>,
}

#[derive(Debug, Clone)]
pub struct OpInfo {
    pub name: String,
    pub docs: Vec<String>,
    pub args: Vec<ArgInfo>,
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub docs: Vec<String>,
    pub fields: Vec<FieldInfo>,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub docs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ArgInfo {
    pub name: String,
    pub type_name: String,
}

impl<'ast> Visit<'ast> for OpVisitor {
    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        let name = node.ident.to_string();
        let mut docs = Vec::new();

        for attr in &node.attrs {
            if let Meta::NameValue(meta) = &attr.meta {
                if !meta.path.is_ident("doc") {
                    continue;
                }
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(lit_str),
                    ..
                }) = &meta.value
                {
                    docs.push(lit_str.value().trim().to_string());
                }
            }
        }

        let mut fields = Vec::new();

        if let syn::Fields::Named(named_fields) = &node.fields {
            for field in &named_fields.named {
                if let Some(ident) = &field.ident {
                    let field_name = ident.to_string();
                    let mut field_docs = Vec::new();

                    for attr in &field.attrs {
                        if let Meta::NameValue(meta) = &attr.meta {
                            if !meta.path.is_ident("doc") {
                                continue;
                            }
                            if let syn::Expr::Lit(syn::ExprLit {
                                lit: syn::Lit::Str(lit_str),
                                ..
                            }) = &meta.value
                            {
                                field_docs.push(lit_str.value().trim().to_string());
                            }
                        }
                    }

                    if !field_docs.is_empty() {
                        fields.push(FieldInfo {
                            name: field_name,
                            docs: field_docs,
                        });
                    }
                }
            }
        }

        if !docs.is_empty() || !fields.is_empty() {
            self.structs.push(StructInfo { name, docs, fields });
        }

        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        // Check if function has #[op2] attribute
        let is_op = node.attrs.iter().any(|attr| {
            if let Meta::List(meta) = &attr.meta {
                return meta.path.is_ident("op2");
            }
            false
        });

        if is_op {
            let name = node.sig.ident.to_string();
            let mut docs = Vec::new();

            // Extract doc comments
            for attr in &node.attrs {
                if let Meta::NameValue(meta) = &attr.meta {
                    if !meta.path.is_ident("doc") {
                        continue;
                    }
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit_str),
                        ..
                    }) = &meta.value
                    {
                        docs.push(lit_str.value().trim().to_string());
                    }
                }
            }

            let mut args = Vec::new();
            for input in &node.sig.inputs {
                let syn::FnArg::Typed(pat_type) = input else {
                    continue;
                };
                let syn::Pat::Ident(pat_ident) = &*pat_type.pat else {
                    continue;
                };

                let arg_name = pat_ident.ident.to_string();
                // Simple type extraction (this is a simplification)
                let type_name = quote::quote!(#pat_type.ty).to_string();
                args.push(ArgInfo {
                    name: arg_name,
                    type_name,
                });
            }

            self.ops.push(OpInfo { name, docs, args });
        }

        // Continue visiting children
        syn::visit::visit_item_fn(self, node);
    }
}

pub fn analyze_source(path: &Path) -> (Vec<OpInfo>, Vec<StructInfo>) {
    let content = fs::read_to_string(path).unwrap_or_default();
    let syntax = syn::parse_file(&content).expect("Unable to parse file");

    let mut visitor = OpVisitor {
        ops: Vec::new(),
        structs: Vec::new(),
    };
    visitor.visit_file(&syntax);

    (visitor.ops, visitor.structs)
}
