use proc_macro::{TokenStream};
use quote::{quote};
use syn::{self, Expr};
use syn::token::Comma;
use syn::parse::Parser;
use syn::punctuated::Punctuated;

// For `proc_macro_derive` 
// 1. it declares a procedure macro
// 2. it has to be annotated with public function, in which the
//    signature has to be `(TokenStream) -> TokenStream`
// 3. because of the signature, the input token stream will be
//    discarded because the function moves the ownership of the
//    stream, don't insert anything to the input stream.
// 4. the identity of the macro is specified in the parenthesis syntax
// 5. it is able to carry attributes (TODO)
 
#[proc_macro_derive(HiMakroPlusSth)]
pub fn whatever_name_unrelated (s_in: TokenStream) -> TokenStream
{ // let it panic and report compile error if the code failed to parse
    let syntaxtree = syn::parse(s_in).unwrap();
    detail_impl_haro(&syntaxtree)
}

fn  detail_impl_haro(ast: & syn::DeriveInput) -> TokenStream
{ // this example expects the top of syntax tree is the struct name
    let mname:&syn::Ident = &ast.ident;
    // the macro `quote!` let programmers define Rust code expression,
    // then convert it to xxx.
    // the macro below will convert `#mname` to the value in the variable `mname`
    let gen = quote! {
        impl AbsHaloMakro for #mname {
            fn shout_out() {
                println!("Hello, Macro! My name is {}!", stringify!(#mname));
            }
        }
    };
    gen.into()
}

#[proc_macro_attribute]
pub fn second_custom_macro(_attr:TokenStream, _in:TokenStream) -> TokenStream
{ // print-line functions are invoked at compile time
    // println!("_attr: {}", _attr.to_string());
    let orig_code:syn::DeriveInput = syn::parse_macro_input!(_in as syn::DeriveInput);
    // get the function pointer
    let punc_parser = Punctuated::<Expr, Comma>::parse_terminated;
    let attr_args = punc_parser.parse(_attr);
    let attr_args: Punctuated<Expr, Comma> = attr_args.unwrap();
    let mut attr_args = attr_args.iter();
    let fn_ident = attr_args.next().unwrap();
    let file_path = attr_args.next().unwrap();
    // println!("fn_ident: {:#?}", file_path);
    attr_chk_2nd_custom_macro(&fn_ident);
    attr_chk_2nd_custom_macro(&file_path);
    let topname:&syn::Ident = &orig_code.ident;
    let gen = quote! {
        impl #topname {
            fn #fn_ident () {
                let msg:&str = #file_path;
                println!("Hi, wave your hands, {} !", msg);
            }
        }
        #orig_code
    };
    TokenStream::from(gen)
} // end of second_custom_macro
// if you don't add any extra code in the macro, following functions
// can be applied
//     use quote::{ToTokens};
//     let _out:TokenStream = tree4code.to_token_stream().into();
//     _out

fn attr_chk_2nd_custom_macro(e:&syn::Expr)
{
    match e {
        Expr::Path(p) => {
            let _ = p.path.segments[0].ident;
        },
        Expr::Lit(li)  => {
            let _ = li.lit;
        },
        _others => {
            panic!("attribute error on : {:#?}", e);
        },
    };
}
